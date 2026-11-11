//! Standalone preferences, unrelated to PoloDB and its schema/reset policy.
use crate::application::{
    dto::{AppSettingsDTO, LogoChoice, ThemeChoice},
    ports::{SettingsError, SettingsRepository},
};
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    fs,
    io::{ErrorKind, Write},
    path::PathBuf,
};

pub struct FileSettingsRepository {
    path: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct SettingsDocument {
    theme: String,
    logo: String,
}

impl FileSettingsRepository {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn configured_path() -> PathBuf {
        let configured = std::env::var_os("MXCHRO_SETTINGS_PATH");
        let appdata = std::env::var_os("APPDATA");
        resolve_path(configured.as_deref(), appdata.as_deref())
    }

    fn write(&self, settings: AppSettingsDTO) -> std::io::Result<()> {
        let parent = self
            .path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or(std::path::Path::new("."));
        fs::create_dir_all(parent)?;
        let temporary = parent.join(format!(".mxchro-settings-{}.tmp", uuid::Uuid::new_v4()));
        let document = SettingsDocument {
            theme: settings.theme.key().into(),
            logo: settings.logo.key().into(),
        };
        let encoded = toml::to_string_pretty(&document).map_err(std::io::Error::other)?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        let result = (|| {
            file.write_all(encoded.as_bytes())?;
            file.sync_all()?;
            drop(file);
            fs::rename(&temporary, &self.path)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }
}

/// Keep relative paths relative to the launch directory; never move existing files.
fn resolve_path(configured: Option<&OsStr>, appdata: Option<&OsStr>) -> PathBuf {
    if let Some(path) = configured.filter(|path| !path.is_empty()) {
        return PathBuf::from(path);
    }
    super::data_path::default_directory(appdata).join("settings.toml")
}

impl SettingsRepository for FileSettingsRepository {
    fn load(&self) -> Result<AppSettingsDTO, SettingsError> {
        let text = match fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return Ok(AppSettingsDTO::default());
            }
            Err(error) => {
                log::warn!("Could not read settings {}: {error}", self.path.display());
                return Err(SettingsError::Read);
            }
        };
        let document: SettingsDocument = toml::from_str(&text).map_err(|error| {
            log::warn!("Invalid settings {}: {error}", self.path.display());
            SettingsError::Invalid
        })?;
        Ok(AppSettingsDTO {
            theme: ThemeChoice::from_key(&document.theme).ok_or(SettingsError::Invalid)?,
            logo: LogoChoice::from_key(&document.logo).ok_or(SettingsError::Invalid)?,
        })
    }

    fn save(&self, settings: AppSettingsDTO) -> Result<(), SettingsError> {
        self.write(settings).map_err(|error| {
            log::warn!("Could not save settings {}: {error}", self.path.display());
            SettingsError::Write
        })
    }
}

#[cfg(test)]
#[path = "../../tests/unit/infrastructure/settings.rs"]
mod tests;
