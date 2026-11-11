use super::{action_count, macro_count, mode_label, text};
use crate::domain::types::Mode;

#[test]
fn english_labels_counts_and_interpolation() {
    assert_eq!(rust_i18n::available_locales!(), vec!["en"]);
    assert_eq!(text("editor.new"), "New macro");
    assert_eq!(mode_label(&Mode::Toggle), "Toggle");
    assert_eq!(macro_count(0), "0 macros");
    assert_eq!(macro_count(1), "1 macro");
    assert_eq!(macro_count(2), "2 macros");
    assert_eq!(action_count(1), "1 action");
    assert_eq!(action_count(2), "2 actions");
    assert_eq!(
        rust_i18n::t!("error.invalid_action_key", key = "left_ctrl"),
        "Invalid action key: left_ctrl"
    );
    assert_eq!(rust_i18n::t!("editor.new", locale = "unknown"), "New macro");
}

#[test]
fn literal_translation_keys_used_in_sources_exist_in_english_catalog() {
    fn check_dir(path: &std::path::Path) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                check_dir(&path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let source = std::fs::read_to_string(&path).unwrap();
                for prefix in ["text(\"", "rust_i18n::t!(\""] {
                    for suffix in source.split(prefix).skip(1) {
                        let key = suffix.split('"').next().unwrap();
                        assert_ne!(text(key), key, "Missing {key} in {}", path.display());
                    }
                }
            }
        }
    }
    check_dir(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"));
}
