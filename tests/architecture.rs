//! Source-level dependency checks for the single-crate modular monolith.
use std::{
    fs,
    path::{Path, PathBuf},
};
use syn::{UseTree, visit::Visit};

fn sources(directory: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files.extend(sources(&path));
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    files
}

#[derive(Default)]
struct Paths(Vec<Vec<String>>);
impl Paths {
    fn collect_use(&mut self, prefix: Vec<String>, tree: &UseTree) {
        match tree {
            UseTree::Path(path) => {
                let mut next = prefix;
                next.push(path.ident.to_string());
                self.collect_use(next, &path.tree);
            }
            UseTree::Group(group) => {
                for item in &group.items {
                    self.collect_use(prefix.clone(), item);
                }
            }
            UseTree::Name(name) => {
                let mut path = prefix;
                path.push(name.ident.to_string());
                self.0.push(path);
            }
            UseTree::Rename(rename) => {
                let mut path = prefix;
                path.push(rename.ident.to_string());
                self.0.push(path);
            }
            UseTree::Glob(_) => self.0.push(prefix),
        }
    }
}
impl<'ast> Visit<'ast> for Paths {
    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        self.collect_use(Vec::new(), &item.tree);
    }
    fn visit_path(&mut self, path: &'ast syn::Path) {
        self.0.push(
            path.segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect(),
        );
        syn::visit::visit_path(self, path);
    }
}

#[test]
fn dependencies_point_inward_and_views_do_not_import_domain_entities() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for file in sources(&root.join("src")) {
        let relative = file
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let source = fs::read_to_string(&file).unwrap();
        let ast = syn::parse_file(&source).unwrap();
        let mut paths = Paths::default();
        paths.visit_file(&ast);
        let banned: &[&str] = if relative.starts_with("src/domain/") {
            &[
                "application",
                "infrastructure",
                "view",
                "bootstrap",
                "gpui_kit",
                "polodb_core",
                "windows",
            ]
        } else if relative.starts_with("src/application/") {
            &[
                "infrastructure",
                "view",
                "bootstrap",
                "gpui_kit",
                "polodb_core",
                "windows",
            ]
        } else if relative.starts_with("src/infrastructure/") {
            &["view", "bootstrap", "gpui_kit"]
        } else if relative.starts_with("src/view/") {
            &["infrastructure", "bootstrap", "polodb_core", "windows"]
        } else {
            &[]
        };
        for path in paths.0 {
            let joined = path.join("::");
            assert!(
                !path
                    .iter()
                    .any(|segment| banned.contains(&segment.as_str())),
                "{relative} imports forbidden dependency {joined}"
            );
            assert!(
                !joined.starts_with("crate::core::") && !joined.starts_with("crate::adapters::"),
                "legacy layer in {relative}: {joined}"
            );
            if relative.starts_with("src/domain/") {
                assert!(
                    !["std::fs", "std::env", "std::thread", "std::sync::mpsc"]
                        .iter()
                        .any(|prefix| joined.starts_with(prefix)),
                    "domain I/O: {relative}: {joined}"
                );
            }
            if relative.starts_with("src/view/") && path.iter().any(|segment| segment == "domain") {
                assert!(
                    matches!(path.last().map(String::as_str), Some("Action" | "Mode")),
                    "UI may import only explicit domain values, not entities or wildcards: {relative}: {joined}"
                );
            }
            if relative.starts_with("src/view/ui/components/")
                || relative.starts_with("src/view/ui/pages/")
            {
                assert!(
                    !joined.contains("application::engine")
                        && !joined.contains("view::backend")
                        && !joined.contains("mxchro_app"),
                    "controlled UI imported its coordinator/backend: {relative}: {joined}"
                );
            }
        }
    }
}

#[test]
fn dto_ownership_and_summary_payload_are_explicit() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for file in sources(&root.join("src")) {
        let ast = syn::parse_file(&fs::read_to_string(&file).unwrap()).unwrap();
        for item in ast.items {
            if let syn::Item::Struct(item) = item {
                let name = item.ident.to_string();
                assert!(!matches!(name.as_str(), "MacroView" | "CollectionView"));
                if name.ends_with("DTO") {
                    assert!(
                        file.starts_with(root.join("src/application/dto")),
                        "DTO outside application: {}",
                        file.display()
                    );
                }
                if name == "MacroDTO" {
                    assert!(
                        !item
                            .fields
                            .iter()
                            .any(|field| field.ident.as_ref().is_some_and(|id| id == "actions")),
                        "summary must not carry the sequence"
                    );
                }
            }
        }
    }
    let snapshot = fs::read_to_string(root.join("src/application/queries.rs")).unwrap();
    assert!(snapshot.contains("Vec<MacroDTO>"));
    assert!(
        !snapshot.contains(".filter_map(|view|"),
        "snapshot must not reload every entity after projecting it"
    );
}
