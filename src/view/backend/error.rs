use crate::{application::error::CommandError, view::i18n::text};

pub(crate) fn message(error: CommandError) -> String {
    match error {
        CommandError::InvalidTriggerKey => text("error.invalid_trigger"),
        CommandError::NoActions => text("error.empty_actions"),
        CommandError::InvalidToggleHotkey => text("error.invalid_hotkey"),
        CommandError::GroupedMacroHotkey => text("error.grouped_macro_hotkey"),
        CommandError::ToggleHotkeyInUse(key) => {
            rust_i18n::t!("error.hotkey_in_use", key = key).into_owned()
        }
        CommandError::InvalidActionKey(key) => {
            rust_i18n::t!("error.invalid_action_key", key = key).into_owned()
        }
        CommandError::ZeroWait => text("error.zero_wait"),
        CommandError::CollectionNotFound => text("error.missing_collection"),
        CommandError::MacroNotFound => text("error.missing_macro"),
        CommandError::CollectionNameRequired => text("error.collection_name"),
        CommandError::EmptyCollectionName => text("error.empty_name"),
        CommandError::OperationFailed => text("error.operation_failed"),
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/view/backend/error.rs"]
mod tests;
