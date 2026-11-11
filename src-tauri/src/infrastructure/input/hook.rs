use std::ptr::null_mut;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::{Arc, OnceLock};

use tauri::{AppHandle, Emitter};
use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VK_LBUTTON, VK_MBUTTON, VK_RBUTTON, VK_XBUTTON1, VK_XBUTTON2,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
    KBDLLHOOKSTRUCT, LLKHF_INJECTED, LLMHF_INJECTED, MSG, MSLLHOOKSTRUCT, WH_KEYBOARD_LL,
    WH_MOUSE_LL, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP,
    WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN, WM_SYSKEYUP, WM_XBUTTONDOWN,
    WM_XBUTTONUP,
};

use crate::core::managers::macro_manager::{MacroManager, ToggleTarget};

struct HookContext {
    manager: Arc<MacroManager>,
    toggle_sender: SyncSender<ToggleTarget>,
}

static CONTEXT: OnceLock<HookContext> = OnceLock::new();

pub struct Hook;

impl Hook {
    pub fn install(manager: Arc<MacroManager>, app_handle: AppHandle) {
        let (toggle_sender, toggle_receiver) = sync_channel(64);
        if CONTEXT
            .set(HookContext {
                manager: Arc::clone(&manager),
                toggle_sender,
            })
            .is_err()
        {
            log::warn!("KeyboardHook ya estaba instalado");
            return;
        }

        // Un único worker ordenado: nunca crear hilos por pulsación ni esperar
        // joins/escrituras desde los callbacks de Windows.
        std::thread::spawn(move || {
            while let Ok(target) = toggle_receiver.recv() {
                if let Some(event) = manager.apply_toggle(target) {
                    let _ = app_handle.emit("toggle-event", event);
                }
            }
        });
        std::thread::spawn(Self::run_message_loop);
    }

    fn process_input(vk: u16, is_down: bool, is_up: bool, is_injected: bool) -> bool {
        let Some(ctx) = CONTEXT.get() else {
            return false;
        };
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

            // Mapear mensaje -> (botón virtual, down/up)
            let (vk, is_down, is_up): (Option<u16>, bool, bool) = match msg {
                WM_LBUTTONDOWN => (Some(VK_LBUTTON.0), true, false),
                WM_LBUTTONUP => (Some(VK_LBUTTON.0), false, true),
                WM_RBUTTONDOWN => (Some(VK_RBUTTON.0), true, false),
                WM_RBUTTONUP => (Some(VK_RBUTTON.0), false, true),
                WM_MBUTTONDOWN => (Some(VK_MBUTTON.0), true, false),
                WM_MBUTTONUP => (Some(VK_MBUTTON.0), false, true),
                WM_XBUTTONDOWN | WM_XBUTTONUP => {
                    // El botón X se indica en el HIWORD de mouseData
                    let xbtn = ((ms.mouseData >> 16) & 0xFFFF) as u16;
                    let vk = if xbtn == 1 {
                        VK_XBUTTON1.0
                    } else {
                        VK_XBUTTON2.0
                    };
                    (Some(vk), msg == WM_XBUTTONDOWN, msg == WM_XBUTTONUP)
                }
                WM_MOUSEWHEEL => {
                    // delta con signo en el HIWORD de mouseData
                    let _delta = (ms.mouseData >> 16) as i16;
                    (None, false, false)
                }
                _ => (None, false, false), // WM_MOUSEMOVE, etc. se ignoran
            };

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
