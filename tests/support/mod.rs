//! Fake input and exclusive temporary persistence, shared by test suites only.
// A suite may use only a subset of the fixture; this never applies to production code.
#![allow(dead_code)]

use super::app::{
    application::{
        commands::{self, Command, CommandOutcome},
        error::CommandError,
    },
    application::{
        dto::SaveMacroDTO,
        engine::{collections::CollectionManager, macros::MacroManager},
        ports::InputBackend,
    },
    domain::types::{Action, Mode},
    infrastructure::persistence::{CollectionRepository, MacroRepository, Persistence},
};
use polodb_core::Database;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

struct FakeInput;
impl InputBackend for FakeInput {
    fn string_to_vk(&self, key: &str) -> Option<u16> {
        match key {
            "A" => Some(65),
            "B" => Some(66),
            "F6" => Some(117),
            _ => None,
        }
    }
    fn vk_to_string(&self, vk: &u16) -> Option<&str> {
        match vk {
            65 => Some("A"),
            66 => Some("B"),
            117 => Some("F6"),
            _ => None,
        }
    }
    fn strings_vec(&self) -> Vec<String> {
        vec!["A".into(), "B".into(), "F6".into()]
    }
    fn key_down(&self, _: u16) {}
    fn key_up(&self, _: u16) {}
}

struct Directory(PathBuf);
impl Drop for Directory {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

pub(crate) struct Fixture {
    pub macros: Arc<MacroManager>,
    pub collections: Arc<CollectionManager>,
    pub persistence: Arc<Persistence>,
    _directory: Directory,
}
impl Fixture {
    pub fn new() -> Self {
        let path = std::env::temp_dir()
            .join("opencode")
            .join(format!("mxchro-commands-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&path).unwrap();
        let db = Database::open_path(&path).unwrap();
        let persistence = Arc::new(Persistence {
            macro_repo: MacroRepository::new(db.clone()),
            collection_repo: CollectionRepository::new(db),
        });
        let instances = Arc::new(RwLock::new(HashMap::new()));
        let repositories = persistence.repositories();
        let hotkeys = Arc::new(RwLock::new(HashMap::new()));
        let input: Arc<dyn InputBackend> = Arc::new(FakeInput);
        let collections = Arc::new(CollectionManager::new(
            Arc::new(RwLock::new(HashMap::new())),
            Arc::clone(&instances),
            Arc::clone(&hotkeys),
            Arc::clone(&input),
            Arc::clone(&repositories),
        ));
        let macros = Arc::new(MacroManager::new(
            instances,
            Arc::clone(&collections),
            hotkeys,
            input,
            Arc::clone(&repositories),
        ));
        Self {
            macros,
            collections,
            persistence,
            _directory: Directory(path),
        }
    }

    pub fn execute(&self, command: Command) -> Result<CommandOutcome, CommandError> {
        commands::execute(&self.macros, &self.collections, command)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.macros.clear_macros();
    }
}

pub(crate) fn config() -> SaveMacroDTO {
    SaveMacroDTO {
        trigger_key: "F6".into(),
        mode: Mode::Once,
        actions: vec![
            Action::Key {
                key: "A".into(),
                press: true,
            },
            Action::Key {
                key: "A".into(),
                press: false,
            },
        ],
        block_original_event: false,
        ignore_injected_keys: true,
        enabled: false,
        collection_id: None,
        toggle_enabled_hotkey: None,
    }
}
