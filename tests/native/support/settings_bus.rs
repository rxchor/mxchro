//! Unused settings channel for fixtures that exercise only macro operations.
use mxchro::view::backend::settings::SettingsBus;
use std::sync::mpsc;

pub fn disconnected() -> SettingsBus {
    let (requests, _) = mpsc::channel();
    let (_, updates) = mpsc::channel();
    SettingsBus {
        initial: Default::default(),
        load_error: None,
        requests,
        updates,
    }
}
