mod key_io;
mod model;

use std::{thread, time::Duration};

use windows::Win32::UI::Input::KeyboardAndMouse;

use crate::key_io::{keyboard::Keyboard, key_mapper::KeyMapper};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn macro_test() {
    thread::sleep(Duration::from_secs(2));

    for _ in 0..10 {
        Keyboard::press_key(KeyboardAndMouse::VK_H, Some(Duration::from_millis(10)));
        Keyboard::press_key(KeyboardAndMouse::VK_O,  Some(Duration::from_millis(10)));
        Keyboard::press_key(KeyboardAndMouse::VK_L,  Some(Duration::from_millis(10)));
        Keyboard::press_key(KeyboardAndMouse::VK_A,  Some(Duration::from_millis(10)));
        Keyboard::press_key(KeyboardAndMouse::VK_RETURN,  Some(Duration::from_millis(10)));
        thread::sleep(Duration::from_millis(10));
    }
}   

#[tauri::command]
fn get_vk_string_values() -> Vec<String> {
    return KeyMapper::new().strings_vec();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![macro_test, get_vk_string_values])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
