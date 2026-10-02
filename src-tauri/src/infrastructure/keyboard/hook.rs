use std::ptr::null_mut;
use std::sync::Once;

use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
    KBDLLHOOKSTRUCT, LLKHF_INJECTED, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN,
    WM_SYSKEYUP,
};

use crate::core::macro_manager::MacroManager;

static HOOK_INIT: Once = Once::new();
static mut MACRO_MANAGER: Option<&'static MacroManager> = None;

pub struct KeyboardHook;

impl KeyboardHook {
    pub fn install(manager: &'static MacroManager) {
        unsafe {
            MACRO_MANAGER = Some(manager);
        }

        HOOK_INIT.call_once(|| {
            std::thread::spawn(move || unsafe {
                let h_instance = HINSTANCE(null_mut());

                let hook = SetWindowsHookExW(
                    WH_KEYBOARD_LL,
                    Some(Self::keyboard_hook_proc),
                    Some(h_instance),
                    0,
                );

                if !hook.is_ok() {
                    return;
                }

                let mut msg = MSG::default();
                while GetMessageW(&mut msg, None, 0, 0).into() {
                    let _ = TranslateMessage(&msg);
                    let _ = DispatchMessageW(&msg);
                }
            });
        });
    }

    unsafe extern "system" fn keyboard_hook_proc(
        n_code: i32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        if n_code >= 0 {
            let kb_struct = unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) };
            let vk = kb_struct.vkCode as u16;

            let is_key_down =
                w_param.0 as u32 == WM_KEYDOWN || w_param.0 as u32 == WM_SYSKEYDOWN;
            let is_key_up = w_param.0 as u32 == WM_KEYUP || w_param.0 as u32 == WM_SYSKEYUP;
            let is_injected = (kb_struct.flags.0 & LLKHF_INJECTED.0) != 0;

            if let Some(manager) = unsafe { MACRO_MANAGER } {
                if is_key_down {
                    manager.on_key_down(vk, is_injected);
                } else if is_key_up {
                    manager.on_key_up(vk);
                }

                if manager.should_block_key(vk) {
                    return LRESULT(1);
                }
            }
        }

        unsafe { CallNextHookEx(None, n_code, w_param, l_param) }
    }
}
