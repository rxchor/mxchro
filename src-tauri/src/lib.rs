mod adapters;
mod core;
mod infrastructure;

use adapters::commands::Commands;
use core::macro_manager::MacroManager;
use core::types::{MacroConfig, MacroSummary};
use infrastructure::keyboard::hook::KeyboardHook;
use once_cell::sync::Lazy;

static MACRO_MANAGER: Lazy<MacroManager> = Lazy::new(|| MacroManager::new());

#[tauri::command]
fn get_vk_string_values() -> Vec<String> {
    Commands::new(&MACRO_MANAGER).get_available_keys()
}

#[tauri::command]
fn add_macro(dto: MacroConfig) -> bool {
    Commands::new(&MACRO_MANAGER).add_macro(dto)
}

#[tauri::command]
fn delete_macro(trigger_key: String) -> bool {
    Commands::new(&MACRO_MANAGER).delete_macro(trigger_key)
}

#[tauri::command]
fn enable_macro(trigger_key: String) -> bool {
    Commands::new(&MACRO_MANAGER).enable_macro(trigger_key)
}

#[tauri::command]
fn disable_macro(trigger_key: String) -> bool {
    Commands::new(&MACRO_MANAGER).disable_macro(trigger_key)
}

#[tauri::command]
fn list_macros() -> Vec<MacroSummary> {
    Commands::new(&MACRO_MANAGER).list_macros()
}

#[tauri::command]
fn get_macro(trigger_key: String) -> Option<MacroConfig> {
    Commands::new(&MACRO_MANAGER).get_macro(trigger_key)
}

#[tauri::command]
fn update_macro(old_trigger_key: String, dto: MacroConfig) -> bool {
    Commands::new(&MACRO_MANAGER).update_macro(old_trigger_key, dto)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    log::info!("Starting mxchro application");

    KeyboardHook::install(&MACRO_MANAGER);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_vk_string_values,
            add_macro,
            get_macro,
            update_macro,
            delete_macro,
            enable_macro,
            disable_macro,
            list_macros,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
