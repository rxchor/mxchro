use std::{ffi::OsStr, path::PathBuf};

/// Shared default for settings and database; overrides are resolved separately.
/// Resolving paths never opens, migrates or deletes existing data.
pub(super) fn default_directory(appdata: Option<&OsStr>) -> PathBuf {
    appdata
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("mxchro-native")
}
