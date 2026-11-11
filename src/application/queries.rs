use super::{
    dto::{CollectionDTO, HotkeyIssueDTO, MacroDTO, MacroEditDTO},
    engine::{collections::CollectionManager, macros::MacroManager},
    error::CommandError,
};

pub enum Query {
    EditMacro(String),
}

#[derive(Default)]
pub struct Snapshot {
    pub hotkey_issues: Vec<HotkeyIssueDTO>,
    pub macros: Vec<MacroDTO>,
    pub collections: Vec<CollectionDTO>,
    pub keys: Vec<String>,
}

pub fn snapshot(macros: &MacroManager, collections: &CollectionManager) -> Snapshot {
    let mut configs = macros.list_macros();
    configs.sort_by(|a, b| a.trigger_key.cmp(&b.trigger_key));
    Snapshot {
        hotkey_issues: macros.hotkey_issues(),
        macros: configs,
        collections: collections.list_collections(),
        keys: macros.get_available_keys(),
    }
}

pub fn macro_for_editing(macros: &MacroManager, key: &str) -> Result<MacroEditDTO, CommandError> {
    macros
        .get_macro(key)
        .map(Into::into)
        .ok_or(CommandError::MacroNotFound)
}
