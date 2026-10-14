use std::sync::{Arc, LazyLock};

use tauri::{command, AppHandle};

use crate::{core::managers::Managers, infrastructure::input::hook::KeyboardHook};

pub mod collection_commands;
pub mod macro_commands;

static MANAGERS: LazyLock<Managers> = LazyLock::new(|| Managers::new());

// ── Utility ─────────────────────────────────────────────────

#[command]
pub fn wipe_data() -> Result<(), String> {
    MANAGERS.wipe_data()?;
    Ok(())
}

pub fn install_hook(app_handle: AppHandle) {
    KeyboardHook::install(Arc::clone(&MANAGERS.macro_manager), app_handle);
}
