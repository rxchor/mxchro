use super::*;

#[test]
fn all_available_keys_round_trip() {
    let mapper = KeyMapper::new();
    for key in mapper.strings_vec() {
        let vk = mapper.string_to_vk(&key).unwrap();
        assert_eq!(mapper.vk_to_string(&vk), Some(key.as_str()));
    }
}

#[test]
fn maps_keyboard_mouse_and_rejects_unknown_keys() {
    let mapper = KeyMapper::new();
    assert_eq!(mapper.string_to_vk("A"), Some(VK_A.0));
    assert_eq!(mapper.string_to_vk("mouse_left"), Some(VK_LBUTTON.0));
    assert_eq!(mapper.string_to_vk("enter"), Some(VK_RETURN.0));
    assert_eq!(mapper.string_to_vk("unknown"), None);
    for (key, vk) in [
        ("left_shift", VK_LSHIFT),
        ("right_shift", VK_RSHIFT),
        ("left_alt", VK_LMENU),
        ("right_alt", VK_RMENU),
    ] {
        assert_eq!(mapper.string_to_vk(key), Some(vk.0));
        assert_eq!(mapper.vk_to_string(&vk.0), Some(key));
    }
}
