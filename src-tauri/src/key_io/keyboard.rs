use std::time::Duration;

use windows::Win32::UI::Input::KeyboardAndMouse::{
    keybd_event, MapVirtualKeyA, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, MAPVK_VK_TO_VSC, VIRTUAL_KEY
};

const KEY_UP: KEYBD_EVENT_FLAGS = KEYEVENTF_KEYUP;
const KEY_DOWN: KEYBD_EVENT_FLAGS = KEYBD_EVENT_FLAGS(0);

pub struct Keyboard;

impl Keyboard {
    
    pub fn press_key(virtual_key: VIRTUAL_KEY, duration: Option<Duration>) {
        let key: u8 = virtual_key.0 as u8;

        Self::send_input(key, KEY_DOWN);
        if let Some(dur) = duration {
            std::thread::sleep(dur);
        }
        Self::send_input(key, KEY_UP);
    }

    fn send_input(key: u8, flag: KEYBD_EVENT_FLAGS) {
        let bscan = Self::vk_to_scan_code(key);
        unsafe {
            keybd_event(key, bscan, flag, 0);
        }
    }

    fn vk_to_scan_code(vk: u8) -> u8 {
        return unsafe { 
            MapVirtualKeyA(vk as u32, MAPVK_VK_TO_VSC) as u8
        }
    }
}