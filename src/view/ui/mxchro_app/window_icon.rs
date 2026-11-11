use super::*;
use crate::{
    application::ports::WindowIconError,
    view::assets::{AppAssets, logo_path},
};

impl MxchroApp {
    pub(super) fn sync_window_icon(&mut self) {
        let path = logo_path(self.settings.logo, self.settings.theme);
        if self.window_icon_path == Some(path) {
            return;
        }
        let Some(backend) = &mut self.window_icon else {
            return;
        };
        let result = match AppAssets.load(path) {
            Ok(Some(svg)) => backend.set_icon(&svg),
            _ => Err(WindowIconError::InvalidSvg),
        };
        match result {
            Ok(()) => self.window_icon_path = Some(path),
            Err(error) => {
                log::warn!("Could not apply window icon {path}: {error:?}");
                self.notification_errors.push(text("error.window_icon"));
            }
        }
    }
}
