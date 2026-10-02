use std::time::Duration;

use windows::Win32::UI::Input::KeyboardAndMouse::{
    MapVirtualKeyA, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
    KEYEVENTF_SCANCODE, MAPVK_VK_TO_VSC, VIRTUAL_KEY,
};

pub struct Keyboard;

impl Keyboard {
    pub fn press_key(virtual_key: u16, duration: Option<Duration>) {
        let scan_code = Self::vk_to_scan_code(virtual_key as u32);

        Self::send_keybd_input(scan_code, KEYEVENTF_SCANCODE, 0);
        if let Some(dur) = duration {
            std::thread::sleep(dur);
        }
        Self::send_keybd_input(scan_code, KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP, 0);
    }

    fn send_keybd_input(
        scan_code: u16,
        flags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS,
        extra_info: usize,
    ) {
        let input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: scan_code,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: extra_info,
                },
            },
        };

        unsafe {
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }
    }

    fn vk_to_scan_code(vk: u32) -> u16 {
        unsafe { MapVirtualKeyA(vk, MAPVK_VK_TO_VSC) as u16 }
    }
}
