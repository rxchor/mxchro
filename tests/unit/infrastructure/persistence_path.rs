use super::*;

#[test]
fn database_and_settings_defaults_share_the_same_appdata_directory() {
    let appdata = OsStr::new(r"C:\Users\example\AppData\Roaming");
    let directory = crate::infrastructure::data_path::default_directory(Some(appdata));
    assert_eq!(directory, PathBuf::from(appdata).join("mxchro-native"));
    let database = resolve_path(None, Some(appdata));
    assert_eq!(database, directory.join("macros.polodb"));
    assert_eq!(database.parent(), directory.join("settings.toml").parent());
}

#[test]
fn missing_or_empty_appdata_uses_the_shared_local_directory() {
    for appdata in [None, Some(OsStr::new(""))] {
        assert_eq!(
            resolve_path(None, appdata),
            PathBuf::from(".")
                .join("mxchro-native")
                .join("macros.polodb")
        );
    }
}

#[test]
fn explicit_database_paths_preserve_the_existing_nested_layout() {
    for configured in [
        r"D:\Mxchro data\custom.db",
        "./mxchro-native-dev.polodb",
        "./portable/custom",
        ".",
        "",
    ] {
        let expected = PathBuf::from(configured)
            .with_extension("polodb")
            .join("macros.polodb");
        for appdata in [None, Some(OsStr::new(r"C:\Users\example\AppData\Roaming"))] {
            assert_eq!(
                resolve_path(Some(OsStr::new(configured)), appdata),
                expected
            );
        }
    }
}

#[test]
fn explicit_dot_keeps_the_old_working_directory_database() {
    assert_eq!(
        resolve_path(Some(OsStr::new(".")), Some(OsStr::new(r"C:\AppData"))),
        PathBuf::from(".").join("macros.polodb")
    );
}

#[test]
fn uninstaller_only_offers_to_remove_the_default_appdata_folder() {
    let config: toml::Value = toml::from_str(include_str!("../../../Cargo.toml")).unwrap();
    let paths = config["package"]["metadata"]["packager"]["nsis"]["appdata-paths"]
        .as_array()
        .unwrap();
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].as_str(), Some("$APPDATA/mxchro-native"));
}
