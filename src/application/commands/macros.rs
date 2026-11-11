use crate::application::error::{CommandError, operation_result};
use crate::application::{
    dto::SaveMacroDTO,
    engine::{collections::CollectionManager, macros::MacroManager},
};
use crate::domain::types::Macro;
use crate::domain::validation::validate_macro;

pub fn validate(config: &SaveMacroDTO, keys: &[String]) -> Result<(), CommandError> {
    validate_macro(&config.clone().into(), keys).map_err(Into::into)
}

pub fn save(
    macros: &MacroManager,
    collections: &CollectionManager,
    old_key: Option<String>,
    config: SaveMacroDTO,
) -> Result<(), CommandError> {
    let config: Macro = config.into();
    validate_macro(&config, &macros.get_available_keys()).map_err(CommandError::from)?;
    if config
        .collection_id
        .as_ref()
        .is_some_and(|id| !collections.list_collections().iter().any(|c| &c.id == id))
    {
        return Err(CommandError::CollectionNotFound);
    }
    macros.validate_toggle_hotkey(
        old_key.as_deref(),
        config.toggle_enabled_hotkey.as_deref(),
        config.collection_id.is_some(),
    )?;
    operation_result(if let Some(old) = old_key {
        macros.update_macro(&old, config)
    } else {
        macros.add_macro(config)
    })
}

pub fn delete(macros: &MacroManager, key: &str) -> Result<(), CommandError> {
    operation_result(macros.delete_macro(key))
}

pub fn set_enabled(macros: &MacroManager, key: &str, enabled: bool) -> Result<(), CommandError> {
    operation_result(if enabled {
        macros.enable_macro(key)
    } else {
        macros.disable_macro(key)
    })
}
