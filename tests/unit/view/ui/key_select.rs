use super::*;

#[test]
fn labels_preserve_exact_values_and_catalog_order() {
    let keys = ["A", "F6", "left_ctrl", "mouse_left", "numpad0", "'"].map(str::to_owned);
    let options = key_options(&keys);
    assert_eq!(
        options.iter().map(|o| o.value.clone()).collect::<Vec<_>>(),
        keys
    );
    assert_eq!(options[0].label.as_ref(), "A");
    assert_eq!(options[1].label.as_ref(), "F6");
    assert_eq!(options[2].label.as_ref(), "Left ctrl");
    assert_eq!(options[3].label.as_ref(), "Mouse left");
    assert!(key_options(&[]).is_empty());
}

#[test]
fn legacy_hotkey_options_preserve_unknown_values_without_duplicating_known_keys() {
    let keys = ["A".into(), "F6".into()];
    assert!(hotkey_options(&keys, None) == key_options(&keys));
    assert_eq!(hotkey_options(&keys, Some("A")).len(), 2);
    let legacy = hotkey_options(&keys, Some("unknown-key"));
    assert_eq!(legacy.len(), 3);
    assert_eq!(legacy[2].value, "unknown-key");
    assert_eq!(legacy[2].label.as_ref(), "unknown-key");
}
