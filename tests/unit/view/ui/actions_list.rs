use super::action_presentation;
use crate::domain::types::Action;
use gpui_kit::assets::IconName;

#[test]
fn labels_distinguish_press_release_and_wait() {
    let press = action_presentation(&Action::Key {
        key: "left_ctrl".into(),
        press: true,
    });
    assert_eq!(press.icon, IconName::ArrowDownToLine);
    assert_eq!(press.label, "PRESS");
    assert_eq!(press.detail, "Left ctrl");
    assert!(press.is_key);
    let release = action_presentation(&Action::Key {
        key: "mouse_left".into(),
        press: false,
    });
    assert_eq!(release.icon, IconName::ArrowUpFromLine);
    assert_eq!(release.label, "RELEASE");
    assert_eq!(release.detail, "Mouse left");
    assert!(release.is_key);
    let wait = action_presentation(&Action::Wait { duration_ms: 100 });
    assert_eq!(wait.icon, IconName::Timer);
    assert_eq!(wait.label, "WAIT");
    assert_eq!(wait.detail, "100 ms");
    assert!(!wait.is_key);
}
