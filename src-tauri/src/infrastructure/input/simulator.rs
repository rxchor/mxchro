use windows::Win32::UI::Input::KeyboardAndMouse::{
    MapVirtualKeyA, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT,
    KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, MAPVK_VK_TO_VSC, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP, MOUSEINPUT, VIRTUAL_KEY,
};

pub struct InputSimulator;

impl InputSimulator {
    pub fn key_down(virtual_key: u16) {
        if Self::is_mouse_button(virtual_key) {
            Self::send_mouse_input(virtual_key, true);
        } else {
            let scan_code = Self::vk_to_scan_code(virtual_key as u32);
            Self::send_keybd_input(scan_code, KEYEVENTF_SCANCODE, 0);
        }
    }

    pub fn key_up(virtual_key: u16) {
        if Self::is_mouse_button(virtual_key) {
            Self::send_mouse_input(virtual_key, false);
        } else {
            let scan_code = Self::vk_to_scan_code(virtual_key as u32);
            Self::send_keybd_input(scan_code, KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP, 0);
        }
    }

    fn is_mouse_button(vk: u16) -> bool {
        matches!(
            vk,
            vk if vk == 0x01 // VK_LBUTTON
                || vk == 0x02 // VK_RBUTTON
                || vk == 0x04 // VK_MBUTTON
                || vk == 0x05 // VK_XBUTTON1
                || vk == 0x06 // VK_XBUTTON2
        )
    }

    fn send_mouse_input(virtual_key: u16, down: bool) {
        let (flags, mouse_data) = match virtual_key {
            0x01 => {
                // VK_LBUTTON
                if down {
                    (MOUSEEVENTF_LEFTDOWN, 0)
                } else {
                    (MOUSEEVENTF_LEFTUP, 0)
                }
            }
            0x02 => {
                // VK_RBUTTON
                if down {
                    (MOUSEEVENTF_RIGHTDOWN, 0)
                } else {
                    (MOUSEEVENTF_RIGHTUP, 0)
                }
            }
            0x04 => {
                // VK_MBUTTON
                if down {
                    (MOUSEEVENTF_MIDDLEDOWN, 0)
                } else {
                    (MOUSEEVENTF_MIDDLEUP, 0)
                }
            }
            0x05 => {
                // VK_XBUTTON1
                if down {
                    (MOUSEEVENTF_XDOWN, 0x0001) // XBUTTON1
                } else {
                    (MOUSEEVENTF_XUP, 0x0001) // XBUTTON1
                }
            }
            0x06 => {
                // VK_XBUTTON2
                if down {
                    (MOUSEEVENTF_XDOWN, 0x0002) // XBUTTON2
                } else {
                    (MOUSEEVENTF_XUP, 0x0002) // XBUTTON2
                }
            }
            _ => return,
        };

        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: mouse_data,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        unsafe {
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }
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
