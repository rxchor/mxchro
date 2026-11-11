//! Typed requests dispatched to use cases. Execute these off the UI/hook thread.
pub mod collections;
pub mod macros;

use super::error::CommandError;
use crate::application::{
    dto::SaveMacroDTO,
    engine::{collections::CollectionManager, macros::MacroManager},
};

pub enum Command {
    SaveMacro {
        old_key: Option<String>,
        config: SaveMacroDTO,
    },
    DeleteMacro(String),
    SetMacroEnabled(String, bool),
    CreateCollection(String),
    RenameCollection(String, String),
    DeleteCollection(String),
    SetCollectionEnabled(String, bool),
    SetCollectionHotkey(String, Option<String>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum CommandOutcome {
    Changed,
    MacroSaved,
}

pub fn execute(
    macros: &MacroManager,
    collections: &CollectionManager,
    command: Command,
) -> Result<CommandOutcome, CommandError> {
    match command {
        Command::SaveMacro { old_key, config } => {
            macros::save(macros, collections, old_key, config)?;
            return Ok(CommandOutcome::MacroSaved);
        }
        Command::DeleteMacro(key) => macros::delete(macros, &key)?,
        Command::SetMacroEnabled(key, enabled) => macros::set_enabled(macros, &key, enabled)?,
        Command::CreateCollection(name) => collections::create(collections, &name)?,
        Command::RenameCollection(id, name) => collections::rename(collections, &id, &name)?,
        Command::DeleteCollection(id) => collections::delete(collections, &id)?,
        Command::SetCollectionEnabled(id, enabled) => {
            collections::set_enabled(collections, &id, enabled)?
        }
        Command::SetCollectionHotkey(id, hotkey) => {
            collections::set_hotkey(macros, collections, &id, hotkey)?
        }
    }
    Ok(CommandOutcome::Changed)
}
