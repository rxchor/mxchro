mod adapters;
mod core;
mod infrastructure;

use adapters::commands::Commands;
use core::macro_manager::MacroManager;
use core::types::{CollectionView, Macro, MacroView};
use infrastructure::input::hook::KeyboardHook;
use once_cell::sync::Lazy;

static MACRO_MANAGER: Lazy<MacroManager> = Lazy::new(|| MacroManager::new());

#[tauri::command]
fn get_vk_string_values() -> Vec<String> {
    Commands::new(&MACRO_MANAGER).get_available_keys()
}

#[tauri::command]
fn add_macro(dto: Macro) -> bool {
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
fn list_macros() -> Vec<MacroView> {
    Commands::new(&MACRO_MANAGER).list_macros()
}

#[tauri::command]
fn get_macro(trigger_key: String) -> Option<Macro> {
    Commands::new(&MACRO_MANAGER).get_macro(trigger_key)
}

#[tauri::command]
fn update_macro(old_trigger_key: String, dto: Macro) -> bool {
    Commands::new(&MACRO_MANAGER).update_macro(old_trigger_key, dto)
}

#[tauri::command]
fn wipe_data() -> Result<(), String> {
    Commands::new(&MACRO_MANAGER).wipe_data()
}

// ── Collections ─────────────────────────────────────────────

#[tauri::command]
fn create_collection(name: String) -> Option<String> {
    Commands::new(&MACRO_MANAGER).create_collection(name)
}

#[tauri::command]
fn delete_collection(collection_id: String) -> bool {
    Commands::new(&MACRO_MANAGER).delete_collection(collection_id)
}

#[tauri::command]
fn enable_collection(collection_id: String) -> bool {
    Commands::new(&MACRO_MANAGER).enable_collection(collection_id)
}

#[tauri::command]
fn disable_collection(collection_id: String) -> bool {
    Commands::new(&MACRO_MANAGER).disable_collection(collection_id)
}

#[tauri::command]
fn list_collections() -> Vec<CollectionView> {
    Commands::new(&MACRO_MANAGER).list_collections()
}

#[tauri::command]
fn rename_collection(collection_id: String, name: String) -> bool {
    Commands::new(&MACRO_MANAGER).rename_collection(collection_id, name)
}

#[tauri::command]
fn reorder_collections(ordered_ids: Vec<String>) -> bool {
    Commands::new(&MACRO_MANAGER).reorder_collections(ordered_ids)
}

#[tauri::command]
fn set_macro_toggle_hotkey(trigger_key: String, hotkey: Option<String>) -> bool {
    Commands::new(&MACRO_MANAGER).set_macro_toggle_hotkey(trigger_key, hotkey)
}

#[tauri::command]
fn set_collection_toggle_hotkey(collection_id: String, hotkey: Option<String>) -> bool {
    Commands::new(&MACRO_MANAGER).set_collection_toggle_hotkey(collection_id, hotkey)
}

#[tauri::command]
fn move_macro_to_collection(trigger_key: String, collection_id: Option<String>) -> bool {
    Commands::new(&MACRO_MANAGER).move_macro_to_collection(trigger_key, collection_id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenv::dotenv().ok();
    env_logger::init();
    log::info!("Starting mxchro application");

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
            wipe_data,
            create_collection,
            delete_collection,
            enable_collection,
            disable_collection,
            list_collections,
            rename_collection,
            reorder_collections,
            set_macro_toggle_hotkey,
            set_collection_toggle_hotkey,
            move_macro_to_collection,
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            KeyboardHook::install(&MACRO_MANAGER, app_handle);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
