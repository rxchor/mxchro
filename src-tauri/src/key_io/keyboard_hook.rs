use std::{collections::HashMap, ptr::null_mut, sync::{RwLock}, thread};

use once_cell::sync::Lazy;
use windows::Win32::{Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM}, UI::{Input::KeyboardAndMouse::VIRTUAL_KEY, WindowsAndMessaging::{CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage, KBDLLHOOKSTRUCT, LLKHF_INJECTED, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP}}};

use crate::model::{key_macro::Macro, key_mode::Mode};

#[allow(dead_code)]
static MACRO_MAP: Lazy<RwLock<HashMap<u16, Macro>>> = Lazy::new(|| {
    return RwLock::new(HashMap::new());
});

pub struct KeyboardHook;

impl KeyboardHook {

    #[allow(dead_code)]
    pub fn handle() {
        // Thread with the hook
        thread::spawn(move || {
            unsafe {
                let h_instance = HINSTANCE(null_mut());
    
                let hook = SetWindowsHookExW(
                    WH_KEYBOARD_LL, 
                    Some(Self::keyboard_hook_proc), 
                    Some(h_instance), 
                    0
                );

                if !hook.is_ok() {
                    return;
                }

                let mut msg = MSG::default();

                while GetMessageW(&mut msg, None, 0, 0).into() {
                    let _ = TranslateMessage( &msg);
                    let _ = DispatchMessageW(&msg);
                }
            }
        });

        // TODO: Spawn a thread that containts the loop that iterates holdable and toggleable macros
        thread::spawn(move || {
            let macros_read = MACRO_MAP.read().unwrap();
            loop {
                for macro_struct in macros_read.values() {
                    if macro_struct.repeat {
                        (macro_struct.action)();
                    }                    
                }
            }
        });
    }

    // HOOK FUNCTION
    unsafe extern "system" fn keyboard_hook_proc(
        n_code: i32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        if n_code >= 0 {
            let kb_struct = unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) };
            let macros_read = MACRO_MAP.read().unwrap();

            for macro_struct in macros_read.values() {
                if kb_struct.vkCode == macro_struct.trigger_key.0 as u32 {
                    
                    if (kb_struct.flags.0 & LLKHF_INJECTED.0) != 0 && macro_struct.ignore_injected_keys {
                        return unsafe { CallNextHookEx(None, n_code, w_param, l_param) };
                    }

                    // Pressed
                    if w_param.0 as u32 == WM_KEYDOWN || w_param.0 as u32 == WM_SYSKEYDOWN {
                        match macro_struct.mode {
                            Mode::HOLD => {
                                let mut macros_write = MACRO_MAP.write().unwrap();
                                let mw = macros_write.get_mut(&macro_struct.trigger_key.0).unwrap();
                                mw.repeat = true;
                                drop(macros_write);
                            },
                            Mode::TOGGLE => {
                                let mut macros_write = MACRO_MAP.write().unwrap();
                                let mw = macros_write.get_mut(&macro_struct.trigger_key.0).unwrap();
                                mw.repeat = !mw.repeat;
                                drop(macros_write);
                            },
                            Mode::ONCE => {
                                let mr = macros_read.get(&macro_struct.trigger_key.0).unwrap();
                                let action = mr.action.clone();
                                thread::spawn(move || {
                                    (action)();
                                });
                                
                            }
                        }
                    }

                    // Released
                    if w_param.0 as u32 == WM_KEYUP || w_param.0 as u32 == WM_SYSKEYUP { 
                        if macro_struct.mode == Mode::HOLD {
                            let mut macros_write = MACRO_MAP.write().unwrap();
                            let mw = macros_write.get_mut(&macro_struct.trigger_key.0).unwrap();
                            mw.repeat = false;
                            drop(macros_write);
                        }
                    }

                    if macro_struct.block_original_event {
                        return LRESULT(1);
                    }
                }
            }
        }

        return unsafe { CallNextHookEx(None, n_code, w_param, l_param) };
    }

    // map related functions
    #[allow(dead_code)]
    pub fn add_macro(key_macro: Macro) -> bool {
        let macros_read = MACRO_MAP.read().unwrap();
        let mut saved = false;
        if !macros_read.contains_key(&key_macro.trigger_key.0) {
            let mut macros_write = MACRO_MAP.write().unwrap();
            macros_write.insert(key_macro.trigger_key.0, key_macro);
            drop(macros_write);
            saved = true;
        }
        return saved;
    }

    #[allow(dead_code)]
    pub fn delete_macro(vk: &VIRTUAL_KEY) -> bool {
        let mut macros_write = MACRO_MAP.write().unwrap();
        return macros_write.remove(&vk.0).is_some();
    }
}


