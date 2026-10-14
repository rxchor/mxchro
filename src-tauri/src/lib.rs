mod adapters;
mod core;
mod infrastructure;

use crate::adapters::commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenv::dotenv().ok();
    env_logger::init();
    log::info!("Starting mxchro application");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::macro_commands::get_vk_string_values,
            commands::macro_commands::add_macro,
            commands::macro_commands::get_macro,
            commands::macro_commands::update_macro,
            commands::macro_commands::delete_macro,
            commands::macro_commands::enable_macro,
            commands::macro_commands::disable_macro,
            commands::macro_commands::list_macros,
            commands::macro_commands::set_macro_toggle_hotkey,
            commands::macro_commands::move_macro_to_collection,
            commands::collection_commands::create_collection,
            commands::collection_commands::delete_collection,
            commands::collection_commands::enable_collection,
            commands::collection_commands::disable_collection,
            commands::collection_commands::list_collections,
            commands::collection_commands::rename_collection,
            commands::collection_commands::reorder_collections,
            commands::collection_commands::set_collection_toggle_hotkey,
            commands::wipe_data,
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            commands::install_hook(app_handle);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
