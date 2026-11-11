use super::*;

#[test]
fn right_control_and_alt_keep_the_extended_flag_for_both_press_and_release() {
    for vk in [0xA3, 0xA5] {
        assert_eq!(
            modifier_flags(vk, false),
            KEYEVENTF_SCANCODE | KEYEVENTF_EXTENDEDKEY
        );
        assert_eq!(
            modifier_flags(vk, true),
            KEYEVENTF_SCANCODE | KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP
        );
    }
    for vk in [0x41, 0xA0, 0xA1, 0xA2, 0xA4] {
        assert_eq!(modifier_flags(vk, false), KEYEVENTF_SCANCODE);
        assert_eq!(
            modifier_flags(vk, true),
            KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP
        );
    }
}
