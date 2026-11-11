use std::ptr::null_mut;
use std::sync::mpsc::{SyncSender, sync_channel};
use std::sync::{Arc, OnceLock};

use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VK_LBUTTON, VK_MBUTTON, VK_RBUTTON, VK_XBUTTON1, VK_XBUTTON2,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, KBDLLHOOKSTRUCT, LLKHF_INJECTED, LLMHF_INJECTED,
    MSG, MSLLHOOKSTRUCT, SetWindowsHookExW, TranslateMessage, WH_KEYBOARD_LL, WH_MOUSE_LL,
    WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP,
    WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN, WM_SYSKEYUP, WM_XBUTTONDOWN, WM_XBUTTONUP,
};

use super::capture::WindowsKeyCapture;
use crate::application::engine::macros::{MacroManager, ToggleTarget};

struct HookContext {
    manager: Arc<MacroManager>,
    toggle_sender: SyncSender<ToggleTarget>,
    capture: Arc<WindowsKeyCapture>,
}

static CONTEXT: OnceLock<HookContext> = OnceLock::new();

pub struct Hook;

impl Hook {
    pub fn install(
        manager: Arc<MacroManager>,
        capture: Arc<WindowsKeyCapture>,
        on_toggle: impl Fn() + Send + 'static,
    ) {
        let (toggle_sender, toggle_receiver) = sync_channel(64);
        if CONTEXT
            .set(HookContext {
                manager: Arc::clone(&manager),
                toggle_sender,
                capture,
            })
            .is_err()
        {
            log::warn!("KeyboardHook ya estaba instalado");
            return;
        }

        std::thread::spawn(move || {
            while let Ok(target) = toggle_receiver.recv() {
                if manager.apply_toggle(target).is_some() {
                    on_toggle();
                }
            }
        });
        std::thread::spawn(Self::run_message_loop);
    }

    fn process_input(vk: u16, is_down: bool, is_up: bool, is_injected: bool) -> bool {
        let Some(ctx) = CONTEXT.get() else {
            return false;
        };
        if ctx.capture.intercept(vk, is_down, is_up, is_injected) {
            return true;
        }
        let result = ctx
            .manager
            .process_key_event(vk, is_down, is_up, is_injected);
        if let Some(target) = result.toggle_request {
            if let Err(e) = ctx.toggle_sender.try_send(target) {
                log::warn!("Couldn't enqueue toggle hotkey: {}", e);
            }
        }
        result.should_block
    }

    fn run_message_loop() {
        unsafe {
            let keyboard_hook = SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(Self::keyboard_hook_proc),
                Some(HINSTANCE(null_mut())),
                0,
            );

            if keyboard_hook.is_err() {
                log::error!("Couldn't install keyboard hook");
                return;
            }

            let mouse_hook = SetWindowsHookExW(
                WH_MOUSE_LL,
                Some(Self::mouse_hook_proc),
                Some(HINSTANCE(null_mut())),
                0,
            );

            if mouse_hook.is_err() {
                log::error!("Couldn't install mouse hook");
            }
            if let Some(ctx) = CONTEXT.get() {
                ctx.capture.set_available(mouse_hook.is_ok());
            }

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).into() {
                let _ = TranslateMessage(&msg);
                let _ = DispatchMessageW(&msg);
            }
            if let Some(ctx) = CONTEXT.get() {
                ctx.capture.set_available(false);
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

            if Self::process_input(vk, is_key_down, is_key_up, is_injected) {
                log::debug!("Blocking key: vk={}", vk);
                return LRESULT(1);
            }
        }

        unsafe { CallNextHookEx(None, n_code, w_param, l_param) }
    }

    unsafe extern "system" fn mouse_hook_proc(
        n_code: i32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        if n_code >= 0 {
            let ms = unsafe { *(l_param.0 as *const MSLLHOOKSTRUCT) };
            let msg = w_param.0 as u32;
            let is_injected = (ms.flags & LLMHF_INJECTED) != 0;

            // Map message -> (virtual mouse button, down/up).
            let (vk, is_down, is_up) = mouse_button_event(msg, ms.mouseData)
                .map_or((None, false, false), |(vk, down, up)| (Some(vk), down, up));

            log::trace!(
                "Mouse event: msg=0x{:X}, vk={:?}, down={}, up={}, injected={}, x={}, y={}",
                msg,
                vk,
                is_down,
                is_up,
                is_injected,
                ms.pt.x,
                ms.pt.y
            );

            if let Some(vk) = vk {
                if Self::process_input(vk, is_down, is_up, is_injected) {
                    return LRESULT(1);
                }
            }
        }

        unsafe { CallNextHookEx(None, n_code, w_param, l_param) }
    }
}

fn mouse_button_event(message: u32, data: u32) -> Option<(u16, bool, bool)> {
    Some(match message {
        WM_LBUTTONDOWN => (VK_LBUTTON.0, true, false),
        WM_LBUTTONUP => (VK_LBUTTON.0, false, true),
        WM_RBUTTONDOWN => (VK_RBUTTON.0, true, false),
        WM_RBUTTONUP => (VK_RBUTTON.0, false, true),
        WM_MBUTTONDOWN => (VK_MBUTTON.0, true, false),
        WM_MBUTTONUP => (VK_MBUTTON.0, false, true),
        WM_XBUTTONDOWN | WM_XBUTTONUP => {
            let vk = match (data >> 16) & 0xFFFF {
                1 => VK_XBUTTON1.0,
                2 => VK_XBUTTON2.0,
                _ => return None,
            };
            (vk, message == WM_XBUTTONDOWN, message == WM_XBUTTONUP)
        }
        // Wheel/movement have no supported single-key Action representation.
        _ => return None,
    })
}

#[cfg(test)]
#[path = "../../../tests/unit/infrastructure/hook.rs"]
mod tests;
