use super::*;

fn down(state: &CaptureState, vk: u16) -> bool {
    state.process(vk, true, false, false, true)
}
fn up(state: &CaptureState, vk: u16) -> bool {
    state.process(vk, false, true, false, true)
}

#[test]
fn capture_swallows_press_repeats_and_release_and_commits_only_after_release() {
    let state = CaptureState::default();
    let session = state.start().unwrap();
    assert!(down(&state, 0x41));
    assert!(down(&state, 0x41));
    assert_eq!(state.poll(session), None);
    assert!(up(&state, 0x41));
    assert_eq!(state.poll(session), Some(Ok(0x41)));
    assert_eq!(state.poll(session), None);
    assert!(!down(&state, 0x41));
    assert!(!up(&state, 0x41));
}

#[test]
fn all_mouse_buttons_are_captured_without_treating_movement_as_input() {
    for vk in [1, 2, 4, 5, 6] {
        let state = CaptureState::default();
        let session = state.start().unwrap();
        assert!(!state.process(0, false, false, false, true));
        assert_eq!(state.poll(session), None);
        assert!(down(&state, vk));
        assert!(up(&state, vk));
        assert_eq!(state.poll(session), Some(Ok(vk)));
    }
}

#[test]
fn activation_click_and_already_held_keys_do_not_become_the_capture() {
    let state = CaptureState::default();
    assert!(!down(&state, 1));
    assert!(!down(&state, 0x20));
    let session = state.start().unwrap();
    assert!(!up(&state, 1));
    assert!(!down(&state, 0x20), "held-key autorepeat was captured");
    assert!(
        !up(&state, 0x20),
        "pre-existing Hold release must reach the engine"
    );
    assert_eq!(state.poll(session), None);
    assert!(down(&state, 0x42));
    assert!(up(&state, 0x42));
    assert_eq!(state.poll(session), Some(Ok(0x42)));
}

#[test]
fn injected_events_neither_complete_capture_nor_modify_physical_release_tracking() {
    let state = CaptureState::default();
    let session = state.start().unwrap();
    assert!(!state.process(0x41, true, false, true, true));
    assert!(!state.process(0x41, false, true, true, true));
    assert!(down(&state, 0x41));
    assert!(!state.process(0x41, false, true, true, true));
    assert_eq!(state.poll(session), None);
    assert!(up(&state, 0x41));
    assert_eq!(state.poll(session), Some(Ok(0x41)));
}

#[test]
fn only_the_first_key_is_selected_and_other_gesture_keys_do_not_escape() {
    let state = CaptureState::default();
    let session = state.start().unwrap();
    assert!(down(&state, 0xA0));
    assert!(down(&state, 0x41));
    assert!(up(&state, 0xA0));
    assert_eq!(state.poll(session), Some(Ok(0xA0)));
    assert!(
        up(&state, 0x41),
        "an unmatched release escaped after capture completion"
    );
    assert!(!down(&state, 0x41));
}

#[test]
fn cancellation_is_owner_safe_and_preserves_swallowed_releases() {
    let state = CaptureState::default();
    let first = state.start().unwrap();
    assert_eq!(state.start(), Err(CaptureError::Busy));
    assert!(down(&state, 0x41));
    state.cancel(first);
    let second = state.start().unwrap();
    state.cancel(first);
    assert_eq!(state.poll(first), None);
    assert!(up(&state, 0x41));
    assert!(down(&state, 0x42));
    assert!(up(&state, 0x42));
    assert_eq!(state.poll(second), Some(Ok(0x42)));
}

#[test]
fn escape_cancels_and_losing_focus_never_swallows_the_other_apps_new_input() {
    let state = CaptureState::default();
    let session = state.start().unwrap();
    assert!(down(&state, ESCAPE));
    assert_eq!(state.poll(session), Some(Err(())));
    assert!(up(&state, ESCAPE));
    let session = state.start().unwrap();
    assert!(!state.process(0x41, true, false, false, false));
    assert_eq!(state.poll(session), Some(Err(())));
}

#[test]
fn a_focus_change_cancels_partial_capture_but_keeps_its_release_suppressed() {
    let state = CaptureState::default();
    let session = state.start().unwrap();
    assert!(down(&state, 0x41));
    assert!(!state.process(0x42, true, false, false, false));
    assert_eq!(state.poll(session), Some(Err(())));
    assert!(state.process(0x41, false, true, false, false));
    assert!(!state.process(0x42, false, true, false, false));
}

#[test]
fn polling_maps_known_keys_but_preserves_an_unknown_vk_for_ui_feedback() {
    let capture = WindowsKeyCapture::new(Arc::new(super::super::mapper::KeyMapper::new()));
    // Start the isolated state directly: this test must never install hooks or
    // require focus/privileges on the desktop.
    for (vk, expected) in [
        (0x41, Some("A")),
        (5, Some("mouse_x1")),
        (0xA0, Some("left_shift")),
        (0xFF, None),
    ] {
        let session = capture.state.start().unwrap();
        assert!(down(&capture.state, vk));
        assert!(up(&capture.state, vk));
        assert_eq!(
            capture.poll(session),
            Some(CaptureOutcome::Key {
                vk,
                key: expected.map(str::to_owned)
            })
        );
    }
    assert_eq!(capture.start(), Err(CaptureError::Unavailable));
}
