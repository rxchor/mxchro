use super::types::{Action, Macro};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MacroValidationError {
    InvalidTriggerKey,
    NoActions,
    InvalidToggleHotkey,
    GroupedMacroHotkey,
    InvalidActionKey(String),
    ZeroWait,
}

pub fn validate_macro(config: &Macro, keys: &[String]) -> Result<(), MacroValidationError> {
    use MacroValidationError::*;
    let valid_key = |key: &str| keys.iter().any(|candidate| candidate == key);
    if !valid_key(&config.trigger_key) {
        return Err(InvalidTriggerKey);
    }
    if config.actions.is_empty() {
        return Err(NoActions);
    }
    if config.collection_id.is_some() && config.toggle_enabled_hotkey.is_some() {
        return Err(GroupedMacroHotkey);
    }
    if config
        .toggle_enabled_hotkey
        .as_deref()
        .is_some_and(|key| !valid_key(key))
    {
        return Err(InvalidToggleHotkey);
    }
    for action in &config.actions {
        match action {
            Action::Key { key, .. } if !valid_key(key) => {
                return Err(InvalidActionKey(key.clone()));
            }
            Action::Wait { duration_ms: 0 } => return Err(ZeroWait),
            _ => {}
        }
    }
    Ok(())
}
