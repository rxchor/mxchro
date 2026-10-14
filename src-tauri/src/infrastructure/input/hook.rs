use std::ptr::null_mut;
use std::sync::{Arc, OnceLock};

use tauri::{AppHandle, Emitter};
use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
    KBDLLHOOKSTRUCT, LLKHF_INJECTED, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN,
    WM_SYSKEYUP,
};

use crate::core::managers::macro_manager::MacroManager;

struct HookContext {
    manager: Arc<MacroManager>,
    app_handle: AppHandle,
}

static CONTEXT: OnceLock<HookContext> = OnceLock::new();

pub struct KeyboardHook;

impl KeyboardHook {
    pub fn install(manager: Arc<MacroManager>, app_handle: AppHandle) {
        if CONTEXT
            .set(HookContext {
                manager,
                app_handle,
            })
            .is_err()
        {
            log::warn!("KeyboardHook ya estaba instalado");
            return;
        }

        std::thread::spawn(Self::run_message_loop);
    }

    fn run_message_loop() {
        unsafe {
            let hook = SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(Self::keyboard_hook_proc),
                Some(HINSTANCE(null_mut())),
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
        }
    }

    unsafe extern "system" fn keyboard_hook_proc(
        n_code: i32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        if n_code >= 0 {
            let kb_struct = unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) };
            let vk = kb_struct.vkCode as u16;

            let is_key_down = w_param.0 as u32 == WM_KEYDOWN || w_param.0 as u32 == WM_SYSKEYDOWN;
            let is_key_up = w_param.0 as u32 == WM_KEYUP || w_param.0 as u32 == WM_SYSKEYUP;
            let is_injected = (kb_struct.flags.0 & LLKHF_INJECTED.0) != 0;

            log::trace!(
                "Key event: vk={}, down={}, up={}, injected={}",
                vk,
                is_key_down,
                is_key_up,
                is_injected
            );

            if let Some(ctx) = CONTEXT.get() {
                // Procesar todo en una sola operación para minimizar el tiempo con el lock
                let action_info =
                    ctx.manager
                        .process_key_event(vk, is_key_down, is_key_up, is_injected);

                // Emit Tauri event if a toggle occurred
                if let Some(toggle_event) = action_info.toggle_event {
                    let _ = ctx.app_handle.emit("toggle-event", toggle_event);
                }

                if action_info.should_block {
                    log::debug!("Blocking key: vk={}", vk);
                    return LRESULT(1);
                }
            }
        }

        unsafe { CallNextHookEx(None, n_code, w_param, l_param) }
    }
}
