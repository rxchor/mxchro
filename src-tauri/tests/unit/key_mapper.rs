use super::*;

#[test]
fn test_vk_to_string_basic_keys() {
    let mapper = KeyMapper::new();
    assert_eq!(mapper.vk_to_string(&VK_A.0), Some("A"));
    assert_eq!(mapper.vk_to_string(&VK_0.0), Some("0"));
    assert_eq!(mapper.vk_to_string(&VK_RETURN.0), Some("enter"));
    assert_eq!(mapper.vk_to_string(&VK_LBUTTON.0), Some("mouse_left"));
}

#[test]
fn test_string_to_vk_basic_keys() {
    let mapper = KeyMapper::new();
    assert_eq!(mapper.string_to_vk("A"), Some(VK_A.0));
    assert_eq!(mapper.string_to_vk("0"), Some(VK_0.0));
    assert_eq!(mapper.string_to_vk("enter"), Some(VK_RETURN.0));
    assert_eq!(mapper.string_to_vk("mouse_left"), Some(VK_LBUTTON.0));
}

#[test]
fn test_vk_to_string_not_found() {
    let mapper = KeyMapper::new();
    assert_eq!(mapper.vk_to_string(&9999), None);
}

#[test]
fn test_string_to_vk_not_found() {
    let mapper = KeyMapper::new();
    assert_eq!(mapper.string_to_vk("not_a_key"), None);
}

#[test]
fn test_vk_to_string_and_back() {
    let mapper = KeyMapper::new();
    let keys = vec!["A", "enter", "mouse_left", "F12", "numpad5"];
    for key in keys {
        let vk = mapper.string_to_vk(key).unwrap();
        let key_back = mapper.vk_to_string(&vk).unwrap();
        assert_eq!(key, key_back);
    }
}
