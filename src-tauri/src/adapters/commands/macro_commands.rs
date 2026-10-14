use tauri::command;

use crate::adapters::commands::MANAGERS;
use crate::core::types::{Macro, MacroView};

// ── Macros ─────────────────────────────────────────────────

#[command]
pub fn get_vk_string_values() -> Vec<String> {
    MANAGERS.macro_manager.get_available_keys()
}

#[command]
pub fn add_macro(dto: Macro) -> bool {
    MANAGERS.macro_manager.add_macro(dto)
}

#[command]
pub fn delete_macro(trigger_key: String) -> bool {
    MANAGERS.macro_manager.delete_macro(&trigger_key)
}

#[command]
pub fn enable_macro(trigger_key: String) -> bool {
    MANAGERS.macro_manager.enable_macro(&trigger_key)
}

#[command]
pub fn disable_macro(trigger_key: String) -> bool {
    MANAGERS.macro_manager.disable_macro(&trigger_key)
}

#[command]
pub fn list_macros() -> Vec<MacroView> {
    MANAGERS.macro_manager.list_macros()
}

#[command]
pub fn get_macro(trigger_key: String) -> Option<Macro> {
    MANAGERS.macro_manager.get_macro(&trigger_key)
}

#[command]
pub fn update_macro(old_trigger_key: String, dto: Macro) -> bool {
    MANAGERS.macro_manager.update_macro(&old_trigger_key, dto)
}

#[command]
pub fn set_macro_toggle_hotkey(trigger_key: String, hotkey: Option<String>) -> bool {
    MANAGERS
        .macro_manager
        .set_macro_toggle_hotkey(&trigger_key, hotkey)
}

#[command]
pub fn move_macro_to_collection(trigger_key: String, collection_id: Option<String>) -> bool {
    MANAGERS
        .macro_manager
        .move_macro_to_collection(&trigger_key, collection_id)
}
