mod key_io;
mod model;

use crate::key_io::key_mapper::KeyMapper;

#[tauri::command]
fn get_vk_string_values() -> Vec<String> {
    return KeyMapper::new().strings_vec();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_vk_string_values])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
