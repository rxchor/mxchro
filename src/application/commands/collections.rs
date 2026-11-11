use crate::application::engine::collections::CollectionManager;
use crate::application::engine::macros::MacroManager;
use crate::application::error::{CommandError, operation_result};

pub fn set_hotkey(
    macros: &MacroManager,
    collections: &CollectionManager,
    id: &str,
    hotkey: Option<String>,
) -> Result<(), CommandError> {
    if !collections
        .list_collections()
        .iter()
        .any(|collection| collection.id == id)
    {
        return Err(CommandError::CollectionNotFound);
    }
    if let Some(key) = &hotkey
        && !macros.get_available_keys().contains(key)
    {
        return Err(CommandError::InvalidToggleHotkey);
    }
    collections.set_collection_toggle_hotkey(id, hotkey)
}

pub fn create(collections: &CollectionManager, name: &str) -> Result<(), CommandError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(CommandError::CollectionNameRequired);
    }
    operation_result(collections.create_collection(name.into()).is_some())
}

pub fn rename(collections: &CollectionManager, id: &str, name: &str) -> Result<(), CommandError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(CommandError::EmptyCollectionName);
    }
    operation_result(collections.rename_collection(id, name.into()))
}

pub fn delete(collections: &CollectionManager, id: &str) -> Result<(), CommandError> {
    operation_result(collections.delete_collection(id))
}

pub fn set_enabled(
    collections: &CollectionManager,
    id: &str,
    enabled: bool,
) -> Result<(), CommandError> {
    operation_result(if enabled {
        collections.enable_collection(id)
    } else {
        collections.disable_collection(id)
    })
}
