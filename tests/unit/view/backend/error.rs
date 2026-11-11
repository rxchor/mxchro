use super::*;

#[test]
fn translates_application_errors_without_changing_key_names() {
    assert_eq!(message(CommandError::NoActions), "Add at least one action.");
    assert_eq!(
        message(CommandError::InvalidActionKey("left_ctrl".into())),
        "Invalid action key: left_ctrl"
    );
    assert_eq!(
        message(CommandError::ToggleHotkeyInUse("left_ctrl".into())),
        "The toggle hotkey left_ctrl is already assigned to another independent macro or collection."
    );
    assert_eq!(
        message(CommandError::GroupedMacroHotkey),
        "A macro in a collection cannot have its own toggle hotkey. Clear the individual hotkey or remove the collection assignment."
    );
}
