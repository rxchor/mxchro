mod key_io;
mod model;

use key_io::key_mapper::KeyMapper;
use key_io::keyboard_hook::KeyboardHook;
use model::dto::{MacroDto, MacroSummaryDto};
use model::key_macro::Macro;

#[tauri::command]
fn get_vk_string_values() -> Vec<String> {
    return KeyMapper::instance().strings_vec();
}

#[tauri::command]
fn add_macro(dto: MacroDto) -> bool {
    if let Some(macro_struct) = Macro::from_dto(dto) {
        return KeyboardHook::add_macro(macro_struct);
    }
    false
}

#[tauri::command]
fn delete_macro(trigger_key: String) -> bool {
    let mapper = KeyMapper::instance();
    if let Some(vk) = mapper.string_to_vk(&trigger_key) {
        let virtual_key = windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(vk);
        return KeyboardHook::delete_macro(&virtual_key);
    }
    false
}

#[tauri::command]
fn enable_macro(trigger_key: String) -> bool {
    let mapper = KeyMapper::instance();
    if let Some(vk) = mapper.string_to_vk(&trigger_key) {
        let virtual_key = windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(vk);
        return KeyboardHook::enable_macro(&virtual_key);
    }
    false
}

#[tauri::command]
fn disable_macro(trigger_key: String) -> bool {
    let mapper = KeyMapper::instance();
    if let Some(vk) = mapper.string_to_vk(&trigger_key) {
        let virtual_key = windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(vk);
        return KeyboardHook::disable_macro(&virtual_key);
    }
    false
}

#[tauri::command]
fn list_macros() -> Vec<MacroSummaryDto> {
    return KeyboardHook::list_macros();
}

#[tauri::command]
fn start_hook() {
    KeyboardHook::handle();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    KeyboardHook::handle();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_vk_string_values,
            add_macro,
            delete_macro,
            enable_macro,
            disable_macro,
            list_macros,
            start_hook,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
