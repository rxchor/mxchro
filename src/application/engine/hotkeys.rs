//! Project configured bindings into a collision-free, lightweight hook index.
//! Callers lock collections before macros. Only the final map swap takes the
//! input index lock; neither I/O nor thread joins happen under that lock.
use super::macros::{MacroInstance, ToggleTarget};
use crate::{
    application::{
        dto::{HotkeyIssueDTO, HotkeyIssueKind, HotkeyOwnerDTO},
        error::CommandError,
        ports::InputBackend,
    },
    domain::types::Collection,
};
use std::{collections::HashMap, sync::RwLock};

struct Binding {
    target: ToggleTarget,
    key: String,
    vk: Option<u16>,
    grouped: bool,
}

fn bindings(
    collections: &HashMap<String, Collection>,
    macros: &HashMap<u16, MacroInstance>,
    input: &dyn InputBackend,
) -> Vec<Binding> {
    let mut bindings = Vec::new();
    for collection in collections.values() {
        if let Some(key) = &collection.toggle_enabled_hotkey {
            bindings.push(Binding {
                target: ToggleTarget::Collection(collection.id.clone()),
                key: key.clone(),
                vk: input.string_to_vk(key),
                grouped: false,
            });
        }
    }
    for instance in macros.values() {
        if let Some(key) = &instance.toggle_hotkey_config {
            bindings.push(Binding {
                target: ToggleTarget::Macro(
                    input
                        .vk_to_string(&instance.trigger_key)
                        .unwrap_or_default()
                        .into(),
                ),
                key: key.clone(),
                vk: input.string_to_vk(key),
                grouped: instance.collection_id.is_some(),
            });
        }
    }
    bindings
}

pub(super) fn validate(
    key: Option<&str>,
    grouped: bool,
    excluded: Option<&ToggleTarget>,
    collections: &HashMap<String, Collection>,
    macros: &HashMap<u16, MacroInstance>,
    input: &dyn InputBackend,
) -> Result<(), CommandError> {
    let Some(key) = key else {
        return Ok(());
    };
    if grouped {
        return Err(CommandError::GroupedMacroHotkey);
    }
    let vk = input
        .string_to_vk(key)
        .ok_or(CommandError::InvalidToggleHotkey)?;
    if bindings(collections, macros, input).iter().any(|binding| {
        !binding.grouped && binding.vk == Some(vk) && Some(&binding.target) != excluded
    }) {
        return Err(CommandError::ToggleHotkeyInUse(key.into()));
    }
    Ok(())
}

fn project(
    collections: &HashMap<String, Collection>,
    macros: &HashMap<u16, MacroInstance>,
    input: &dyn InputBackend,
) -> (HashMap<u16, ToggleTarget>, Vec<HotkeyIssueDTO>) {
    let bindings = bindings(collections, macros, input);
    let mut counts = HashMap::<u16, usize>::new();
    for binding in &bindings {
        if !binding.grouped
            && let Some(vk) = binding.vk
        {
            *counts.entry(vk).or_default() += 1;
        }
    }
    let mut active = HashMap::new();
    let mut issues = Vec::new();
    for binding in bindings {
        let issue = if binding.grouped {
            Some(HotkeyIssueKind::GroupedMacro)
        } else if binding.vk.is_none() {
            Some(HotkeyIssueKind::InvalidKey)
        } else if counts[&binding.vk.unwrap()] > 1 {
            Some(HotkeyIssueKind::Duplicate)
        } else {
            None
        };
        if let Some(kind) = issue {
            issues.push(HotkeyIssueDTO {
                owner: match binding.target {
                    ToggleTarget::Macro(key) => HotkeyOwnerDTO::Macro(key),
                    ToggleTarget::Collection(id) => HotkeyOwnerDTO::Collection(id),
                },
                hotkey: binding.key,
                kind,
            });
        } else if let Some(vk) = binding.vk {
            active.insert(vk, binding.target);
        }
    }
    issues.sort();
    (active, issues)
}

pub(super) fn refresh(
    index: &RwLock<HashMap<u16, ToggleTarget>>,
    collections: &HashMap<String, Collection>,
    macros: &HashMap<u16, MacroInstance>,
    input: &dyn InputBackend,
) {
    let (active, _) = project(collections, macros, input);
    let old = std::mem::replace(&mut *index.write().unwrap(), active);
    drop(old);
}

pub(super) fn issues(
    collections: &HashMap<String, Collection>,
    macros: &HashMap<u16, MacroInstance>,
    input: &dyn InputBackend,
) -> Vec<HotkeyIssueDTO> {
    project(collections, macros, input).1
}
