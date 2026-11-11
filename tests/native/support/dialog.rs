//! Keyboard interaction with the actual GPUI-KIT dialog layer.
#![allow(dead_code)]
use gpui_kit::{component::WindowExt, *};
use std::time::Duration;

pub fn key(handle: AnyWindowHandle, name: &'static str, cx: &mut AsyncApp) {
    cx.update_window(handle, |_, window, cx| {
        let keystroke = Keystroke::parse(name).unwrap();
        window.dispatch_event(
            KeyDownEvent {
                keystroke: keystroke.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(KeyUpEvent { keystroke }.to_platform_input(), cx);
        window.draw(cx).clear(cx);
    })
    .unwrap();
}

pub fn active(handle: AnyWindowHandle, cx: &mut AsyncApp) -> bool {
    cx.update_window(handle, |_, window, cx| window.has_active_dialog(cx))
        .unwrap()
}

pub async fn accept(handle: AnyWindowHandle, cx: &mut AsyncApp) {
    assert!(active(handle, cx), "confirmation dialog was not opened");
    key(handle, "tab", cx); // Initial focus is Cancel; move to the destructive button.
    key(handle, "enter", cx);
    cx.background_executor()
        .timer(Duration::from_millis(600))
        .await;
}
