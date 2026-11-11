use super::{label, toggle_hotkey_tag};

#[test]
fn absent_assignments_have_no_tag() {
    assert!(toggle_hotkey_tag(None, false).is_none());
    assert!(toggle_hotkey_tag(None, true).is_none());
}

#[test]
fn labels_distinguish_activation_from_enable_hotkeys_and_inactive_assignments() {
    assert_eq!(label("F6", false), "Enable/disable: F6");
    assert_eq!(label("mouse_x1", false), "Enable/disable: Mouse x1");
    assert_eq!(
        label("legacy_key", true),
        "Enable/disable: Legacy key (inactive)"
    );
    assert!(toggle_hotkey_tag(Some("F6"), false).is_some());
    assert!(toggle_hotkey_tag(Some("legacy_key"), true).is_some());
}
