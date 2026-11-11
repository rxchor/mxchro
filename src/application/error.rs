/// Expected failures at the application boundary. Presentation belongs to adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    InvalidTriggerKey,
    NoActions,
    InvalidToggleHotkey,
    GroupedMacroHotkey,
    ToggleHotkeyInUse(String),
    InvalidActionKey(String),
    ZeroWait,
    CollectionNotFound,
    MacroNotFound,
    CollectionNameRequired,
    EmptyCollectionName,
    OperationFailed,
}

impl From<crate::domain::validation::MacroValidationError> for CommandError {
    fn from(error: crate::domain::validation::MacroValidationError) -> Self {
        use crate::domain::validation::MacroValidationError as E;
        match error {
            E::InvalidTriggerKey => Self::InvalidTriggerKey,
            E::NoActions => Self::NoActions,
            E::InvalidToggleHotkey => Self::InvalidToggleHotkey,
            E::GroupedMacroHotkey => Self::GroupedMacroHotkey,
            E::InvalidActionKey(key) => Self::InvalidActionKey(key),
            E::ZeroWait => Self::ZeroWait,
        }
    }
}

pub(super) fn operation_result(success: bool) -> Result<(), CommandError> {
    success.then_some(()).ok_or(CommandError::OperationFailed)
}
