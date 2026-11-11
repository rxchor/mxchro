//! Ordered preference writes on a dedicated worker, never on the GPUI thread.
use crate::application::{
    dto::AppSettingsDTO,
    ports::{SettingsError, SettingsRepository},
};
use std::sync::mpsc::{Receiver, Sender};

pub struct SettingsBus {
    pub initial: AppSettingsDTO,
    pub load_error: Option<SettingsError>,
    pub requests: Sender<AppSettingsDTO>,
    pub updates: Receiver<Result<(), SettingsError>>,
}

pub fn run(
    repository: &dyn SettingsRepository,
    requests: Receiver<AppSettingsDTO>,
    updates: Sender<Result<(), SettingsError>>,
) {
    for settings in requests {
        if updates.send(repository.save(settings)).is_err() {
            break;
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/view/backend/settings.rs"]
mod tests;
