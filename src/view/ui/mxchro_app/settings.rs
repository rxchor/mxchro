use super::*;
use crate::{
    application::ports::SettingsError,
    view::{theme, ui::pages::settings_page::SettingsEvent},
};

pub(super) fn settings_error(error: SettingsError) -> String {
    text(match error {
        SettingsError::Read => "error.settings_read",
        SettingsError::Invalid => "error.settings_invalid",
        SettingsError::Write => "error.settings_write",
    })
}

impl MxchroApp {
    pub(super) fn handle_settings(&mut self, event: &SettingsEvent, cx: &mut Context<Self>) {
        if matches!(event, SettingsEvent::Back) {
            self.settings_open = false;
            cx.notify();
            return;
        }
        if self.settings_pending {
            return;
        }
        match *event {
            SettingsEvent::Theme(theme) => self.settings.theme = theme,
            SettingsEvent::Logo(logo) => self.settings.logo = logo,
            SettingsEvent::Retry => {}
            SettingsEvent::Back => return,
        }
        theme::apply(self.settings.theme, cx);
        self.sync_window_icon();
        self.settings_error = None;
        self.settings_pending = self.settings_bus.requests.send(self.settings).is_ok();
        if !self.settings_pending {
            let message = settings_error(SettingsError::Write);
            self.settings_error = Some(message.clone());
            self.notification_errors.push(message);
        }
        cx.notify();
    }

    pub(super) fn poll_settings(&mut self, cx: &mut Context<Self>) {
        while self.settings_pending {
            match self.settings_bus.updates.try_recv() {
                Ok(result) => {
                    self.settings_pending = false;
                    if let Err(error) = result {
                        let message = settings_error(error);
                        self.settings_error = Some(message.clone());
                        self.notification_errors.push(message);
                    }
                    cx.notify();
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.settings_pending = false;
                    let message = settings_error(SettingsError::Write);
                    self.settings_error = Some(message.clone());
                    self.notification_errors.push(message);
                    cx.notify();
                }
            }
        }
    }
}
