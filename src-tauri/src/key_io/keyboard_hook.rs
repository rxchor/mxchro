use std::{collections::HashMap, ptr::null_mut, sync::{Once, RwLock}, thread};

use once_cell::sync::Lazy;
use windows::Win32::{Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM}, UI::{Input::KeyboardAndMouse::VIRTUAL_KEY, WindowsAndMessaging::{CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage, KBDLLHOOKSTRUCT, LLKHF_INJECTED, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP}}};

use crate::model::{dto::MacroSummaryDto, key_macro::Macro, key_mode::Mode};
use crate::key_io::key_mapper::KeyMapper;

static MACRO_MAP: Lazy<RwLock<HashMap<u16, Macro>>> = Lazy::new(|| {
    return RwLock::new(HashMap::new());
});

static HOOK_INIT: Once = Once::new();

pub struct KeyboardHook;

impl KeyboardHook {

    pub fn handle() {
        HOOK_INIT.call_once(|| {
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
            
            let action_to_run = {
                let macros_read = MACRO_MAP.read().unwrap();
                if let Some(macro_struct) = macros_read.get(&vk) {
                    if !macro_struct.enabled {
                        return unsafe { CallNextHookEx(None, n_code, w_param, l_param) };
                    }
                    
                    if (kb_struct.flags.0 & LLKHF_INJECTED.0) != 0 && macro_struct.ignore_injected_keys {
                        return unsafe { CallNextHookEx(None, n_code, w_param, l_param) };
                    }
                    
                    if w_param.0 as u32 == WM_KEYDOWN || w_param.0 as u32 == WM_SYSKEYDOWN {
                        if macro_struct.mode == Mode::ONCE {
                            Some(macro_struct.action.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            };
            
            if let Some(action) = action_to_run {
                thread::spawn(move || {
                    (action)();
                });
            }
            
            let is_key_down = w_param.0 as u32 == WM_KEYDOWN || w_param.0 as u32 == WM_SYSKEYDOWN;
            let is_key_up = w_param.0 as u32 == WM_KEYUP || w_param.0 as u32 == WM_SYSKEYUP;
            
            if is_key_down || is_key_up {
                let repeat_state_clone = {
                    let macros_read = MACRO_MAP.read().unwrap();
                    if let Some(m) = macros_read.get(&vk) {
                        if !m.enabled {
                            return unsafe { CallNextHookEx(None, n_code, w_param, l_param) };
                        }
                        
                        if is_key_down && m.mode == Mode::ONCE {
                            None
                        } else {
                            Some((m.mode.clone(), m.repeat_state.clone()))
                        }
                    } else {
                        None
                    }
                };
                
                if let Some((mode, repeat_state)) = repeat_state_clone {
                    if is_key_down {
                        match mode {
                            Mode::HOLD => {
                                let mut state = repeat_state.0.lock().unwrap();
                                state.active = true;
                                repeat_state.1.notify_one();
                            },
                            Mode::TOGGLE => {
                                let mut state = repeat_state.0.lock().unwrap();
                                state.active = !state.active;
                                if state.active {
                                    repeat_state.1.notify_one();
                                }
                            },
                            Mode::ONCE => {}
                        }
                    } else if is_key_up && mode == Mode::HOLD {
                        let mut state = repeat_state.0.lock().unwrap();
                        state.active = false;
                    }
                }
            }
            
            let should_block = {
                let macros_read = MACRO_MAP.read().unwrap();
                macros_read.get(&vk).map_or(false, |m| m.enabled && m.block_original_event)
            };
            
            if should_block {
                return LRESULT(1);
            }
        }

        return unsafe { CallNextHookEx(None, n_code, w_param, l_param) };
    }

    pub fn add_macro(key_macro: Macro) -> bool {
        let mut macros_write = MACRO_MAP.write().unwrap();
        if !macros_write.contains_key(&key_macro.trigger_key.0) {
            let mut macro_clone = key_macro;
            if macro_clone.enabled {
                macro_clone.spawn_repeat_thread();
            }
            macros_write.insert(macro_clone.trigger_key.0, macro_clone);
            return true;
        }
        false
    }

    pub fn delete_macro(vk: &VIRTUAL_KEY) -> bool {
        let mut macros_write = MACRO_MAP.write().unwrap();
        if let Some(mut m) = macros_write.remove(&vk.0) {
            m.stop_repeat_thread();
            true
        } else {
            false
        }
    }

    pub fn enable_macro(vk: &VIRTUAL_KEY) -> bool {
        let mut macros_write = MACRO_MAP.write().unwrap();
        if let Some(m) = macros_write.get_mut(&vk.0) {
            m.enable();
            true
        } else {
            false
        }
    }

    pub fn disable_macro(vk: &VIRTUAL_KEY) -> bool {
        let mut macros_write = MACRO_MAP.write().unwrap();
        if let Some(m) = macros_write.get_mut(&vk.0) {
            m.disable();
            true
        } else {
            false
        }
    }

    pub fn list_macros() -> Vec<MacroSummaryDto> {
        let macros_read = MACRO_MAP.read().unwrap();
        let mapper = KeyMapper::instance();
        macros_read.values()
            .filter_map(|m| m.to_summary(&mapper))
            .collect()
    }
}


