use super::{
    AppSettingsDTO, FileSettingsRepository, LogoChoice, SettingsError, SettingsRepository,
    ThemeChoice,
};
use std::{ffi::OsStr, fs, path::PathBuf};

#[test]
fn configured_path_overrides_appdata_and_preserves_absolute_and_relative_paths() {
    let appdata = Some(OsStr::new(r"C:\Users\example\AppData\Roaming"));
    for configured in [
        r"D:\Mxchro preferences\settings.toml",
        "./portable/settings.toml",
        "custom.conf",
    ] {
        assert_eq!(
            super::resolve_path(Some(OsStr::new(configured)), appdata),
            PathBuf::from(configured)
        );
    }
}

#[test]
fn absent_or_empty_overrides_keep_the_existing_default_path() {
    let appdata = OsStr::new(r"C:\Users\example\AppData\Roaming");
    let expected = PathBuf::from(appdata)
        .join("mxchro-native")
        .join("settings.toml");
    assert_eq!(super::resolve_path(None, Some(appdata)), expected);
    assert_eq!(
        super::resolve_path(Some(OsStr::new("")), Some(appdata)),
        expected
    );
    for appdata in [None, Some(OsStr::new(""))] {
        assert_eq!(
            super::resolve_path(None, appdata),
            PathBuf::from(".")
                .join("mxchro-native")
                .join("settings.toml")
        );
    }
}

struct Fixture {
    directory: PathBuf,
    repository: FileSettingsRepository,
}
impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir()
            .join("opencode")
            .join(format!("mxchro-settings-test-{}", uuid::Uuid::new_v4()));
        Self {
            repository: FileSettingsRepository::new(directory.join("settings.toml")),
            directory,
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if self.directory.exists() {
            let _ = fs::remove_dir_all(&self.directory);
        }
    }
}

#[test]
fn missing_preferences_use_defaults_without_creating_files() {
    let f = Fixture::new();
    assert_eq!(f.repository.load(), Ok(AppSettingsDTO::default()));
    assert!(!f.directory.exists());
}

#[test]
fn all_choices_round_trip_and_existing_file_is_replaced() {
    let f = Fixture::new();
    for theme in ThemeChoice::ALL {
        for logo in LogoChoice::ALL {
            let settings = AppSettingsDTO { theme, logo };
            f.repository.save(settings).unwrap();
            assert_eq!(f.repository.load(), Ok(settings));
            assert_eq!(
                fs::read_dir(&f.directory).unwrap().count(),
                1,
                "temporary file leaked"
            );
        }
    }
}

#[test]
fn malformed_and_unknown_preferences_are_not_reset_or_rewritten() {
    let f = Fixture::new();
    fs::create_dir_all(&f.directory).unwrap();
    for invalid in [
        "not a valid TOML [",
        "theme = 'unknown'\nlogo = 'monochrome'",
        "theme = 'monochrome'\nlogo = 'unknown'",
    ] {
        fs::write(&f.repository.path, invalid).unwrap();
        assert_eq!(f.repository.load(), Err(SettingsError::Invalid));
        assert_eq!(fs::read_to_string(&f.repository.path).unwrap(), invalid);
    }
}

#[test]
fn write_failures_are_propagated_without_deleting_existing_data() {
    let f = Fixture::new();
    fs::create_dir_all(&f.directory).unwrap();
    let blocker = f.directory.join("blocker");
    fs::write(&blocker, "keep").unwrap();
    let repository = FileSettingsRepository::new(blocker.join("settings.toml"));
    assert_eq!(
        repository.save(AppSettingsDTO::default()),
        Err(SettingsError::Write)
    );
    assert_eq!(fs::read_to_string(blocker).unwrap(), "keep");
}

#[test]
fn failed_replacement_preserves_the_target_and_removes_only_its_temporary_file() {
    let f = Fixture::new();
    fs::create_dir_all(&f.repository.path).unwrap();
    let sentinel = f.repository.path.join("keep.txt");
    fs::write(&sentinel, "keep").unwrap();
    assert_eq!(
        f.repository.save(AppSettingsDTO::default()),
        Err(SettingsError::Write)
    );
    assert_eq!(fs::read_to_string(sentinel).unwrap(), "keep");
    assert_eq!(fs::read_dir(&f.directory).unwrap().count(), 1);
}
