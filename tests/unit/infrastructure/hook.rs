use super::*;
use windows::Win32::UI::WindowsAndMessaging::{WM_MOUSEMOVE, WM_MOUSEWHEEL};

#[test]
fn mouse_hook_messages_map_to_the_same_vk_catalog_used_for_capture_and_execution() {
    for (down, up, data, vk) in [
        (WM_LBUTTONDOWN, WM_LBUTTONUP, 0, VK_LBUTTON.0),
        (WM_RBUTTONDOWN, WM_RBUTTONUP, 0, VK_RBUTTON.0),
        (WM_MBUTTONDOWN, WM_MBUTTONUP, 0, VK_MBUTTON.0),
        (WM_XBUTTONDOWN, WM_XBUTTONUP, 1 << 16, VK_XBUTTON1.0),
        (WM_XBUTTONDOWN, WM_XBUTTONUP, 2 << 16, VK_XBUTTON2.0),
    ] {
        assert_eq!(mouse_button_event(down, data), Some((vk, true, false)));
        assert_eq!(mouse_button_event(up, data), Some((vk, false, true)));
    }
    assert_eq!(mouse_button_event(WM_XBUTTONDOWN, 3 << 16), None);
    assert_eq!(mouse_button_event(WM_MOUSEMOVE, 0), None);
    assert_eq!(mouse_button_event(WM_MOUSEWHEEL, 120 << 16), None);
}
