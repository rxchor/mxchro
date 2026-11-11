use std::collections::HashMap;
mod execution;
mod input;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread::JoinHandle;
use std::time::Duration;

use super::hotkeys;
use crate::application::engine::collections::CollectionManager;
use crate::application::{
    dto::MacroDTO,
    ports::{InputBackend, Repositories},
};
use crate::domain::types::{Action, Macro, Mode};
use crate::{
    application::{dto::HotkeyIssueDTO, error::CommandError},
    domain::types::Collection,
};
type ActionCallback = Arc<dyn Fn() + Send + Sync + 'static>;

pub struct RepeatState {
    pub active: bool,
    pub should_stop: bool,
    pub is_executing: bool, // For Mode::Once: whether the action is executing.
}

pub struct MacroInstance {
    pub trigger_key: u16,
    pub mode: Mode,
    pub action: ActionCallback,
    pub actions_config: Vec<Action>, // Original actions retained for editing.
    pub block_original_event: bool,
    pub ignore_injected_keys: bool,
    pub enabled: bool,
    input_enabled: Arc<AtomicBool>,
    pub collection_id: Option<String>,
    pub(super) toggle_hotkey_config: Option<String>,
    pub actions_count: usize,
    pub repeat_state: Arc<(Mutex<RepeatState>, Condvar)>,
    pub thread_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

/// Result of processing a key event.
pub struct KeyEventResult {
    pub should_block: bool,
    pub toggle_request: Option<ToggleTarget>,
}

// Lightweight hook index: never held locked during I/O or thread joins.
#[derive(Clone)]
struct MacroInputState {
    mode: Mode,
    block_original_event: bool,
    ignore_injected_keys: bool,
    enabled: Arc<AtomicBool>,
    repeat_state: Arc<(Mutex<RepeatState>, Condvar)>,
}

/// Toggle event produced while processing a key.
#[derive(Clone, serde::Serialize)]
pub struct ToggleEvent {
    pub target: ToggleTarget,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "type", content = "value")]
pub enum ToggleTarget {
    Macro(String),      // trigger_key
    Collection(String), // collection_id
}

pub struct MacroManager {
    macros: Arc<RwLock<HashMap<u16, MacroInstance>>>,
    collections: Arc<RwLock<HashMap<String, Collection>>>,
    input_states: RwLock<HashMap<u16, MacroInputState>>,
    collection_manager: Arc<CollectionManager>,
    toggle_hotkeys: Arc<RwLock<HashMap<u16, ToggleTarget>>>,
    key_mapper: Arc<dyn InputBackend>,
    repositories: Arc<Repositories>,
}

impl MacroManager {
    pub fn new(
        macros: Arc<RwLock<HashMap<u16, MacroInstance>>>,
        collection_manager: Arc<CollectionManager>,
        toggle_hotkeys: Arc<RwLock<HashMap<u16, ToggleTarget>>>,
        key_mapper: Arc<dyn InputBackend>,
        repositories: Arc<Repositories>,
    ) -> Self {
        let manager = MacroManager {
            collections: Arc::clone(&collection_manager.collections),
            macros,
            input_states: RwLock::new(HashMap::new()),
            collection_manager,
            toggle_hotkeys,
            key_mapper,
            repositories,
        };

        manager.load_macros();

        manager
    }

    fn trigger_key_str(&self, vk: &u16) -> String {
        self.key_mapper
            .vk_to_string(vk)
            .unwrap_or_default()
            .to_string()
    }

    fn build_config(&self, instance: &MacroInstance) -> Macro {
        Macro {
            trigger_key: self.trigger_key_str(&instance.trigger_key),
            mode: instance.mode.clone(),
            actions: instance.actions_config.clone(),
            block_original_event: instance.block_original_event,
            ignore_injected_keys: instance.ignore_injected_keys,
            enabled: instance.enabled,
            collection_id: instance.collection_id.clone(),
            toggle_enabled_hotkey: instance.toggle_hotkey_config.clone(),
        }
    }

    pub fn move_macro_to_collection(
        &self,
        trigger_key: &str,
        collection_id: Option<String>,
    ) -> bool {
        let vk = match self.key_mapper.string_to_vk(trigger_key) {
            Some(vk) => vk,
            None => {
                log::warn!(
                    "move_macro_to_collection: trigger key '{}' not recognized",
                    trigger_key
                );
                return false;
            }
        };

        let collections = self.collections.read().unwrap();
        let mut macros = self.macros.write().unwrap();
        if collection_id
            .as_ref()
            .is_some_and(|id| !collections.contains_key(id))
        {
            return false;
        }
        let Some(instance) = macros.get(&vk) else {
            return false;
        };
        if hotkeys::validate(
            instance.toggle_hotkey_config.as_deref(),
            collection_id.is_some(),
            Some(&ToggleTarget::Macro(trigger_key.into())),
            &collections,
            &macros,
            self.key_mapper.as_ref(),
        )
        .is_err()
        {
            return false;
        }
        if let Some(instance) = macros.get_mut(&vk) {
            let mut config = self.build_config(instance);
            config.collection_id = collection_id.clone();
            if let Err(e) = self.repositories.macro_repo.replace(trigger_key, &config) {
                log::error!("Failed to persist macro move: {}", e);
                return false;
            }
            instance.collection_id = collection_id.clone();
            hotkeys::refresh(
                &self.toggle_hotkeys,
                &collections,
                &macros,
                self.key_mapper.as_ref(),
            );

            log::info!(
                "Macro '{}' moved to collection {:?}",
                trigger_key,
                collection_id
            );
            true
        } else {
            log::warn!(
                "move_macro_to_collection: macro with trigger key '{}' not found",
                trigger_key
            );
            false
        }
    }

    fn create_instance(&self, macro_entity: Macro) -> MacroInstance {
        let vk = self
            .key_mapper
            .string_to_vk(&macro_entity.trigger_key)
            .unwrap();
        let actions = self.build_actions(&macro_entity.actions);
        let actions_count = actions.len();
        let combined_action = self.combine_actions(actions);

        let repeat_state = Arc::new((
            Mutex::new(RepeatState {
                active: false,
                should_stop: false,
                is_executing: false,
            }),
            Condvar::new(),
        ));

        MacroInstance {
            trigger_key: vk,
            mode: macro_entity.mode.clone(),
            action: combined_action,
            actions_config: macro_entity.actions.clone(),
            block_original_event: macro_entity.block_original_event,
            ignore_injected_keys: macro_entity.ignore_injected_keys,
            enabled: macro_entity.enabled,
            input_enabled: Arc::new(AtomicBool::new(macro_entity.enabled)),
            collection_id: macro_entity.collection_id.clone(),
            toggle_hotkey_config: macro_entity.toggle_enabled_hotkey.clone(),
            actions_count,
            repeat_state,
            thread_handle: Arc::new(Mutex::new(None)),
        }
    }

    pub fn add_macro(&self, macro_entity: Macro) -> bool {
        let vk = match self.key_mapper.string_to_vk(&macro_entity.trigger_key) {
            Some(vk) => vk,
            None => {
                log::warn!(
                    "add_macro: trigger key '{}' not recognized",
                    macro_entity.trigger_key
                );
                return false;
            }
        };

        let collections = self.collections.read().unwrap();
        let mut macros = self.macros.write().unwrap();
        if macro_entity
            .collection_id
            .as_ref()
            .is_some_and(|id| !collections.contains_key(id))
        {
            return false;
        }
        if hotkeys::validate(
            macro_entity.toggle_enabled_hotkey.as_deref(),
            macro_entity.collection_id.is_some(),
            None,
            &collections,
            &macros,
            self.key_mapper.as_ref(),
        )
        .is_err()
        {
            return false;
        }
        if macros.contains_key(&vk) {
            log::warn!(
                "add_macro: trigger key '{}' (vk={}) already exists",
                macro_entity.trigger_key,
                vk
            );
            return false;
        }

        let mut instance = self.create_instance(macro_entity.clone());
        if instance.enabled {
            instance.spawn_repeat_thread();
        }

        let config = self.build_config(&instance);
        log::info!("Macro added: trigger={} (vk={})", instance.trigger_key, vk);
        self.input_states
            .write()
            .unwrap()
            .insert(vk, instance.input_state());
        macros.insert(vk, instance);
        hotkeys::refresh(
            &self.toggle_hotkeys,
            &collections,
            &macros,
            self.key_mapper.as_ref(),
        );
        drop(macros);

        if let Err(e) = self.repositories.macro_repo.insert(&config) {
            log::error!("Failed to persist new macro: {}", e);
        }
        true
    }

    pub fn get_macro(&self, trigger_key: &str) -> Option<Macro> {
        let vk = match self.key_mapper.string_to_vk(trigger_key) {
            Some(vk) => vk,
            None => return None,
        };

        let macros = self.macros.read().unwrap();
        macros.get(&vk).map(|instance| Macro {
            trigger_key: self
                .key_mapper
                .vk_to_string(&instance.trigger_key)
                .unwrap_or(trigger_key)
                .to_string(),
            mode: instance.mode.clone(),
            actions: instance.actions_config.clone(),
            block_original_event: instance.block_original_event,
            ignore_injected_keys: instance.ignore_injected_keys,
            enabled: instance.enabled,
            collection_id: instance.collection_id.clone(),
            toggle_enabled_hotkey: instance.toggle_hotkey_config.clone(),
        })
    }

    pub fn update_macro(&self, old_trigger_key: &str, macro_entity: Macro) -> bool {
        let old_vk = match self.key_mapper.string_to_vk(old_trigger_key) {
            Some(vk) => vk,
            None => {
                log::warn!(
                    "update_macro: old trigger key '{}' not recognized",
                    old_trigger_key
                );
                return false;
            }
        };

        let new_vk = match self.key_mapper.string_to_vk(&macro_entity.trigger_key) {
            Some(vk) => vk,
            None => {
                log::warn!(
                    "update_macro: new trigger key '{}' not recognized",
                    macro_entity.trigger_key
                );
                return false;
            }
        };

        let collections = self.collections.read().unwrap();
        let mut macros = self.macros.write().unwrap();
        if macro_entity
            .collection_id
            .as_ref()
            .is_some_and(|id| !collections.contains_key(id))
        {
            return false;
        }
        if hotkeys::validate(
            macro_entity.toggle_enabled_hotkey.as_deref(),
            macro_entity.collection_id.is_some(),
            Some(&ToggleTarget::Macro(old_trigger_key.into())),
            &collections,
            &macros,
            self.key_mapper.as_ref(),
        )
        .is_err()
        {
            return false;
        }

        // If the trigger key changes, check that the new one is not already used.
        if old_vk != new_vk && macros.contains_key(&new_vk) {
            log::warn!(
                "update_macro: new trigger key '{}' (vk={}) already exists",
                macro_entity.trigger_key,
                new_vk
            );
            return false;
        }

        // Remove the old macro.
        if let Some(mut old_instance) = macros.remove(&old_vk) {
            self.input_states.write().unwrap().remove(&old_vk);
            old_instance.input_enabled.store(false, Ordering::Release);
            old_instance.stop_repeat_thread();
            log::info!(
                "Macro updated: old trigger={} (vk={})",
                old_trigger_key,
                old_vk
            );
        } else {
            log::warn!(
                "update_macro: macro with trigger key '{}' (vk={}) not found",
                old_trigger_key,
                old_vk
            );
            return false;
        }

        let mut instance = self.create_instance(macro_entity.clone());
        let actions_count = instance.actions_count;

        if instance.enabled {
            instance.spawn_repeat_thread();
        }

        log::info!(
            "Macro updated: new trigger={} (vk={}), mode={:?}, actions={}, enabled={}",
            macro_entity.trigger_key,
            new_vk,
            instance.mode,
            actions_count,
            instance.enabled
        );

        self.input_states
            .write()
            .unwrap()
            .insert(new_vk, instance.input_state());
        macros.insert(new_vk, instance);
        hotkeys::refresh(
            &self.toggle_hotkeys,
            &collections,
            &macros,
            self.key_mapper.as_ref(),
        );
        drop(macros);

        if let Err(e) = self
            .repositories
            .macro_repo
            .replace(old_trigger_key, &macro_entity)
        {
            log::error!("Failed to persist macro update: {}", e);
        }
        true
    }

    pub fn delete_macro(&self, trigger_key: &str) -> bool {
        let vk = match self.key_mapper.string_to_vk(trigger_key) {
            Some(vk) => vk,
            None => {
                log::warn!("delete_macro: trigger key '{}' not recognized", trigger_key);
                return false;
            }
        };

        let collections = self.collections.read().unwrap();
        let mut macros = self.macros.write().unwrap();
        if let Some(mut instance) = macros.remove(&vk) {
            self.input_states.write().unwrap().remove(&vk);
            instance.input_enabled.store(false, Ordering::Release);
            hotkeys::refresh(
                &self.toggle_hotkeys,
                &collections,
                &macros,
                self.key_mapper.as_ref(),
            );

            log::info!("Macro deleted: trigger={} (vk={})", trigger_key, vk);
            instance.stop_repeat_thread();
            drop(macros);

            if let Err(e) = self.repositories.macro_repo.delete(trigger_key) {
                log::error!("Failed to persist macro deletion: {}", e);
            }
            true
        } else {
            log::warn!(
                "delete_macro: trigger key '{}' (vk={}) not found",
                trigger_key,
                vk
            );
            false
        }
    }

    pub fn enable_macro(&self, trigger_key: &str) -> bool {
        let vk = match self.key_mapper.string_to_vk(trigger_key) {
            Some(vk) => vk,
            None => {
                log::warn!("enable_macro: trigger key '{}' not recognized", trigger_key);
                return false;
            }
        };

        let mut macros = self.macros.write().unwrap();
        if let Some(instance) = macros.get_mut(&vk) {
            log::info!("Macro enabled: trigger={} (vk={})", trigger_key, vk);
            instance.enable();
            drop(macros);

            if let Err(e) = self.repositories.macro_repo.set_enabled(trigger_key, true) {
                log::error!("Failed to persist enable for '{}': {}", trigger_key, e);
            }
            true
        } else {
            log::warn!(
                "enable_macro: trigger key '{}' (vk={}) not found",
                trigger_key,
                vk
            );
            false
        }
    }

    pub fn disable_macro(&self, trigger_key: &str) -> bool {
        let vk = match self.key_mapper.string_to_vk(trigger_key) {
            Some(vk) => vk,
            None => {
                log::warn!(
                    "disable_macro: trigger key '{}' not recognized",
                    trigger_key
                );
                return false;
            }
        };

        let mut macros = self.macros.write().unwrap();
        if let Some(instance) = macros.get_mut(&vk) {
            log::info!("Macro disabled: trigger={} (vk={})", trigger_key, vk);
            instance.disable();
            drop(macros);

            if let Err(e) = self.repositories.macro_repo.set_enabled(trigger_key, false) {
                log::error!("Failed to persist disable for '{}': {}", trigger_key, e);
            }
            true
        } else {
            log::warn!(
                "disable_macro: trigger key '{}' (vk={}) not found",
                trigger_key,
                vk
            );
            false
        }
    }

    pub fn list_macros(&self) -> Vec<MacroDTO> {
        let macros = self.macros.read().unwrap();
        macros
            .values()
            .map(|m| MacroDTO {
                trigger_key: self
                    .key_mapper
                    .vk_to_string(&m.trigger_key)
                    .unwrap_or_default()
                    .to_string(),
                mode: m.mode.clone(),
                actions_count: m.actions_config.len(),
                block_original_event: m.block_original_event,
                ignore_injected_keys: m.ignore_injected_keys,
                enabled: m.enabled,
                collection_id: m.collection_id.clone(),
                toggle_enabled_hotkey: m.toggle_hotkey_config.clone(),
            })
            .collect()
    }

    pub fn set_macro_toggle_hotkey(&self, trigger_key: &str, hotkey: Option<String>) -> bool {
        let vk = match self.key_mapper.string_to_vk(trigger_key) {
            Some(vk) => vk,
            None => {
                log::warn!(
                    "set_macro_toggle_hotkey: trigger key '{}' not recognized",
                    trigger_key
                );
                return false;
            }
        };

        let collections = self.collections.read().unwrap();
        let mut macros = self.macros.write().unwrap();
        let Some(instance) = macros.get(&vk) else {
            return false;
        };
        if hotkeys::validate(
            hotkey.as_deref(),
            instance.collection_id.is_some(),
            Some(&ToggleTarget::Macro(trigger_key.into())),
            &collections,
            &macros,
            self.key_mapper.as_ref(),
        )
        .is_err()
        {
            return false;
        }
        if let Some(instance) = macros.get_mut(&vk) {
            if let Err(e) = self
                .repositories
                .macro_repo
                .set_toggle_hotkey(trigger_key, hotkey.clone())
            {
                log::error!(
                    "Failed to persist toggle hotkey for '{}': {}",
                    trigger_key,
                    e
                );
                return false;
            }
            instance.toggle_hotkey_config = hotkey.clone();
            hotkeys::refresh(
                &self.toggle_hotkeys,
                &collections,
                &macros,
                self.key_mapper.as_ref(),
            );

            log::info!("Macro '{}' toggle hotkey set to {:?}", trigger_key, hotkey);
            true
        } else {
            log::warn!(
                "set_macro_toggle_hotkey: macro with trigger key '{}' not found",
                trigger_key
            );
            false
        }
    }

    /// Toggle a macro's enabled state (called from its toggle hotkey).
    /// Return the new state on success, or None if the macro was not found.
    fn toggle_macro_enabled(&self, trigger_key: &str) -> Option<bool> {
        let vk = match self.key_mapper.string_to_vk(trigger_key) {
            Some(vk) => vk,
            None => return None,
        };

        let mut macros = self.macros.write().unwrap();
        if let Some(instance) = macros.get_mut(&vk) {
            if instance.collection_id.is_some() {
                return None;
            }
            let new_enabled = !instance.enabled;
            if new_enabled {
                instance.enable();
            } else {
                instance.disable();
            }
            instance.enabled = new_enabled;

            drop(macros);

            if let Err(e) = self
                .repositories
                .macro_repo
                .set_enabled(trigger_key, new_enabled)
            {
                log::error!(
                    "Failed to persist toggle for macro '{}': {}",
                    trigger_key,
                    e
                );
            }

            log::info!("Macro '{}' toggled to enabled={}", trigger_key, new_enabled);
            Some(new_enabled)
        } else {
            None
        }
    }

    pub fn get_available_keys(&self) -> Vec<String> {
        self.key_mapper.strings_vec()
    }

    pub fn validate_toggle_hotkey(
        &self,
        old_key: Option<&str>,
        hotkey: Option<&str>,
        grouped: bool,
    ) -> Result<(), CommandError> {
        let collections = self.collections.read().unwrap();
        let macros = self.macros.read().unwrap();
        hotkeys::validate(
            hotkey,
            grouped,
            old_key.map(|key| ToggleTarget::Macro(key.into())).as_ref(),
            &collections,
            &macros,
            self.key_mapper.as_ref(),
        )
    }

    pub fn hotkey_issues(&self) -> Vec<HotkeyIssueDTO> {
        let collections = self.collections.read().unwrap();
        let macros = self.macros.read().unwrap();
        hotkeys::issues(&collections, &macros, self.key_mapper.as_ref())
    }

    /// Execute a toggle off the hook thread (may wait for threads and disk).
    pub fn apply_toggle(&self, target: ToggleTarget) -> Option<ToggleEvent> {
        // Ignore requests that were queued before their binding was invalidated.
        if !self
            .toggle_hotkeys
            .read()
            .unwrap()
            .values()
            .any(|active| active == &target)
        {
            return None;
        }
        let enabled = match &target {
            ToggleTarget::Macro(key) => self.toggle_macro_enabled(key),
            ToggleTarget::Collection(id) => self.collection_manager.toggle_collection_enabled(id),
        }?;
        Some(ToggleEvent { target, enabled })
    }

    fn build_actions(&self, events: &[Action]) -> Vec<ActionCallback> {
        events
            .iter()
            .filter_map(|event| match event {
                Action::Wait { duration_ms } => {
                    let duration = *duration_ms;
                    Some(Arc::new(move || {
                        std::thread::sleep(Duration::from_millis(duration));
                    }) as ActionCallback)
                }
                Action::Key { key, press } => {
                    let vk = self.key_mapper.string_to_vk(key)?;
                    let press = *press;
                    let input = Arc::clone(&self.key_mapper);

                    Some(Arc::new(move || {
                        if press {
                            input.key_down(vk);
                        } else {
                            input.key_up(vk);
                        }
                    }) as ActionCallback)
                }
            })
            .collect()
    }

    fn combine_actions(&self, actions: Vec<ActionCallback>) -> ActionCallback {
        Arc::new(move || {
            for action in &actions {
                action();
            }
        })
    }

    fn load_macros(&self) {
        let configs = self.repositories.macro_repo.load_all();
        let collections = self.collections.read().unwrap();
        let mut macros = self.macros.write().unwrap();

        for config in configs {
            if let Some(vk) = self.key_mapper.string_to_vk(&config.trigger_key) {
                if !macros.contains_key(&vk) {
                    let mut instance = self.create_instance(config);
                    if instance.enabled {
                        instance.spawn_repeat_thread();
                    }
                    self.input_states
                        .write()
                        .unwrap()
                        .insert(vk, instance.input_state());
                    macros.insert(vk, instance);
                }
            }
        }

        log::info!("Loaded {} macros from persistence", macros.len());
        hotkeys::refresh(
            &self.toggle_hotkeys,
            &collections,
            &macros,
            self.key_mapper.as_ref(),
        );
    }

    pub fn clear_macros(&self) {
        // Stop all repeat threads.
        let collections = self.collections.read().unwrap();
        let mut macros = self.macros.write().unwrap();
        self.input_states.write().unwrap().clear();
        for (_, mut instance) in macros.drain() {
            instance.input_enabled.store(false, Ordering::Release);
            instance.stop_repeat_thread();
        }
        hotkeys::refresh(
            &self.toggle_hotkeys,
            &collections,
            &macros,
            self.key_mapper.as_ref(),
        );
    }
}
