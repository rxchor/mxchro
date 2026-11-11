//! Engine regression tests with in-memory repository ports and fake input.
use super::*;
use crate as app;
use crate::{
    application::ports::InputBackend,
    domain::types::{Action, Macro, Mode},
};
#[path = "../../support/memory.rs"]
mod memory;
use std::{sync::mpsc, time::Duration};

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

struct Fixture {
    managers: Managers,
    macros: Arc<RwLock<HashMap<u16, MacroInstance>>>,
}
impl Fixture {
    fn new() -> Self {
        let repositories = memory::repositories();
        let macros = Arc::new(RwLock::new(HashMap::new()));
        let hotkeys = Arc::new(RwLock::new(HashMap::new()));
        let input: Arc<dyn InputBackend> = Arc::new(FakeInput);
        let collection_manager = Arc::new(CollectionManager::new(
            Arc::new(RwLock::new(HashMap::new())),
            Arc::clone(&macros),
            Arc::clone(&hotkeys),
            Arc::clone(&input),
            Arc::clone(&repositories),
        ));
        let macro_manager = Arc::new(MacroManager::new(
            Arc::clone(&macros),
            Arc::clone(&collection_manager),
            hotkeys,
            input,
            Arc::clone(&repositories),
        ));
        Self {
            managers: Managers {
                repositories,
                collection_manager,
                macro_manager,
            },
            macros,
        }
    }
    fn add(&self, key: &str, collection_id: Option<String>, mode: Mode) {
        assert!(self.managers.macro_manager.add_macro(Macro {
            trigger_key: key.into(),
            mode,
            actions: vec![Action::Wait { duration_ms: 1 }],
            block_original_event: true,
            ignore_injected_keys: false,
            enabled: true,
            collection_id,
            toggle_enabled_hotkey: None,
        }));
    }
    fn assert_enabled(&self, key: &str, enabled: bool) {
        assert_eq!(
            self.managers.macro_manager.get_macro(key).unwrap().enabled,
            enabled
        );
        assert_eq!(
            self.managers
                .repositories
                .macro_repo
                .load_all()
                .iter()
                .find(|m| m.trigger_key == key)
                .unwrap()
                .enabled,
            enabled
        );
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.managers.macro_manager.clear_macros();
    }
}

#[test]
fn collection_toggle_and_delete_are_persisted() {
    let f = Fixture::new();
    let collection = &f.managers.collection_manager;
    let id = collection.create_collection("Trabajo".into()).unwrap();
    f.add("A", Some(id.clone()), Mode::Once);
    f.add("B", None, Mode::Once);
    assert!(collection.disable_collection(&id));
    f.assert_enabled("A", false);
    f.assert_enabled("B", true);
    assert_eq!(collection.toggle_collection_enabled(&id), Some(true));
    f.assert_enabled("A", true);
    assert!(collection.delete_collection(&id));
    assert!(
        f.managers
            .macro_manager
            .get_macro("A")
            .unwrap()
            .collection_id
            .is_none()
    );
    assert!(
        f.managers
            .repositories
            .macro_repo
            .load_all()
            .iter()
            .all(|m| m.collection_id.is_none())
    );
}

#[test]
fn hook_does_not_wait_for_crud_lock_and_defers_toggles() {
    let f = Fixture::new();
    f.add("A", None, Mode::Once);
    let manager = Arc::clone(&f.managers.macro_manager);
    assert!(manager.set_macro_toggle_hotkey("A", Some("F6".into())));
    let lock = f.macros.write().unwrap();
    let (tx, rx) = mpsc::channel();
    let thread = std::thread::spawn(move || {
        let physical = manager.process_key_event(65, true, false, false);
        let injected = manager.process_key_event(65, true, false, true);
        let toggle = manager.process_key_event(117, true, false, false);
        tx.send((physical, injected, toggle)).unwrap();
    });
    let result = rx.recv_timeout(Duration::from_secs(2));
    drop(lock);
    thread.join().unwrap();
    let (physical, injected, toggle) = result.expect("hook blocked behind CRUD");
    assert!(physical.should_block);
    assert!(!injected.should_block);
    f.assert_enabled("A", true);
    assert!(
        !f.managers
            .macro_manager
            .apply_toggle(toggle.toggle_request.unwrap())
            .unwrap()
            .enabled
    );
    f.assert_enabled("A", false);
}

#[test]
fn hold_and_toggle_ignore_their_own_injected_events() {
    let f = Fixture::new();
    for (key, vk, mode) in [("A", 65, Mode::Hold), ("B", 66, Mode::Toggle)] {
        f.add(key, None, mode.clone());
        let state = Arc::clone(&f.macros.read().unwrap().get(&vk).unwrap().repeat_state);
        let manager = &f.managers.macro_manager;
        manager.process_key_event(vk, true, false, false);
        assert!(state.0.lock().unwrap().active);
        let down = mode == Mode::Toggle;
        assert!(
            !manager
                .process_key_event(vk, down, !down, true)
                .should_block
        );
        assert!(state.0.lock().unwrap().active);
        manager.process_key_event(vk, down, !down, false);
        assert!(!state.0.lock().unwrap().active);
    }
}

#[test]
fn editing_deleting_and_wiping_refresh_input_indices() {
    let f = Fixture::new();
    f.add("A", None, Mode::Once);
    let manager = &f.managers.macro_manager;
    let mut config = manager.get_macro("A").unwrap();
    config.trigger_key = "B".into();
    config.toggle_enabled_hotkey = Some("F6".into());
    assert!(manager.update_macro("A", config));
    assert!(
        !manager
            .process_key_event(65, false, true, false)
            .should_block
    );
    assert!(
        manager
            .process_key_event(66, false, true, false)
            .should_block
    );
    assert!(manager.delete_macro("B"));
    assert!(
        manager
            .process_key_event(117, true, false, false)
            .toggle_request
            .is_none()
    );
    f.add("A", None, Mode::Once);
    f.managers.wipe_data().unwrap();
    assert!(
        !manager
            .process_key_event(65, false, true, false)
            .should_block
    );
    assert!(manager.list_macros().is_empty());
}

fn legacy_macro(key: &str, hotkey: &str, collection: Option<&str>) -> Macro {
    Macro {
        trigger_key: key.into(),
        mode: Mode::Once,
        actions: vec![Action::Wait { duration_ms: 1 }],
        block_original_event: false,
        ignore_injected_keys: true,
        enabled: false,
        collection_id: collection.map(str::to_owned),
        toggle_enabled_hotkey: Some(hotkey.into()),
    }
}

fn legacy_collection(id: &str, hotkey: Option<&str>) -> crate::domain::types::Collection {
    crate::domain::types::Collection {
        id: id.into(),
        name: id.into(),
        enabled: false,
        order: 0,
        toggle_enabled_hotkey: hotkey.map(str::to_owned),
    }
}

#[test]
fn grouped_legacy_hotkeys_are_preserved_but_only_the_collection_can_toggle() {
    use crate::application::dto::HotkeyIssueKind;
    let repositories = memory::repositories();
    repositories
        .collection_repo
        .insert(&legacy_collection("work", Some("B")))
        .unwrap();
    repositories
        .macro_repo
        .insert(&legacy_macro("A", "F6", Some("work")))
        .unwrap();
    let managers = Managers::new(Arc::new(FakeInput), repositories.clone());
    let macros = &managers.macro_manager;
    assert_eq!(
        macros
            .get_macro("A")
            .unwrap()
            .toggle_enabled_hotkey
            .as_deref(),
        Some("F6")
    );
    assert_eq!(
        repositories.macro_repo.load_all()[0]
            .toggle_enabled_hotkey
            .as_deref(),
        Some("F6")
    );
    assert_eq!(
        macros.hotkey_issues()[0].kind,
        HotkeyIssueKind::GroupedMacro
    );
    assert!(
        macros
            .process_key_event(117, true, false, false)
            .toggle_request
            .is_none()
    );
    assert!(
        macros
            .apply_toggle(ToggleTarget::Macro("A".into()))
            .is_none()
    );
    let request = macros
        .process_key_event(66, true, false, false)
        .toggle_request
        .unwrap();
    assert_eq!(request, ToggleTarget::Collection("work".into()));
    macros.apply_toggle(request).unwrap();
    assert!(macros.get_macro("A").unwrap().enabled);
    assert!(!macros.set_macro_toggle_hotkey("A", Some("A".into())));
    assert!(macros.set_macro_toggle_hotkey("A", None));
    assert!(macros.hotkey_issues().is_empty());
    assert!(
        repositories.macro_repo.load_all()[0]
            .toggle_enabled_hotkey
            .is_none()
    );
    macros.clear_macros();
}

#[test]
fn every_legacy_duplicate_is_inactive_and_resolution_restores_only_the_remaining_owner() {
    use crate::application::dto::HotkeyIssueKind;
    let repositories = memory::repositories();
    repositories
        .collection_repo
        .insert(&legacy_collection("work", Some("F6")))
        .unwrap();
    for key in ["A", "B"] {
        repositories
            .macro_repo
            .insert(&legacy_macro(key, "F6", None))
            .unwrap();
    }
    let managers = Managers::new(Arc::new(FakeInput), repositories.clone());
    let macros = &managers.macro_manager;
    assert!(
        macros
            .process_key_event(117, true, false, false)
            .toggle_request
            .is_none()
    );
    let issues = macros.hotkey_issues();
    assert_eq!(issues.len(), 3);
    assert!(
        issues
            .iter()
            .all(|issue| issue.kind == HotkeyIssueKind::Duplicate)
    );
    assert!(
        repositories
            .macro_repo
            .load_all()
            .iter()
            .all(|config| config.toggle_enabled_hotkey.as_deref() == Some("F6"))
    );
    assert!(macros.set_macro_toggle_hotkey("B", None));
    assert!(
        macros
            .process_key_event(117, true, false, false)
            .toggle_request
            .is_none()
    );
    assert!(macros.delete_macro("A"));
    assert!(macros.hotkey_issues().is_empty());
    assert_eq!(
        macros
            .process_key_event(117, true, false, false)
            .toggle_request,
        Some(ToggleTarget::Collection("work".into()))
    );
    assert!(
        macros
            .apply_toggle(ToggleTarget::Macro("A".into()))
            .is_none()
    );
    managers.collection_manager.delete_collection("work");
    assert!(
        macros
            .process_key_event(117, true, false, false)
            .toggle_request
            .is_none()
    );
    macros.clear_macros();
}

#[test]
fn removing_a_grouped_legacy_binding_does_not_remove_another_owners_active_index() {
    let repositories = memory::repositories();
    for id in ["one", "two"] {
        repositories
            .collection_repo
            .insert(&legacy_collection(id, Some("F6")))
            .unwrap();
    }
    repositories
        .macro_repo
        .insert(&legacy_macro("A", "F6", Some("one")))
        .unwrap();
    let managers = Managers::new(Arc::new(FakeInput), repositories);
    managers
        .collection_manager
        .set_collection_toggle_hotkey("one", None)
        .unwrap();
    let macros = &managers.macro_manager;
    assert_eq!(
        macros
            .process_key_event(117, true, false, false)
            .toggle_request,
        Some(ToggleTarget::Collection("two".into()))
    );
    assert!(macros.delete_macro("A"));
    assert_eq!(
        macros
            .process_key_event(117, true, false, false)
            .toggle_request,
        Some(ToggleTarget::Collection("two".into()))
    );
    macros.clear_macros();
}

#[test]
fn unknown_legacy_hotkey_is_reported_without_rewriting_the_configuration() {
    use crate::application::dto::HotkeyIssueKind;
    let repositories = memory::repositories();
    repositories
        .macro_repo
        .insert(&legacy_macro("A", "unknown", None))
        .unwrap();
    let managers = Managers::new(Arc::new(FakeInput), repositories.clone());
    assert_eq!(
        managers
            .macro_manager
            .get_macro("A")
            .unwrap()
            .toggle_enabled_hotkey
            .as_deref(),
        Some("unknown")
    );
    assert_eq!(
        managers.macro_manager.hotkey_issues()[0].kind,
        HotkeyIssueKind::InvalidKey
    );
    assert_eq!(
        repositories.macro_repo.load_all()[0]
            .toggle_enabled_hotkey
            .as_deref(),
        Some("unknown")
    );
    managers.macro_manager.clear_macros();
}

#[test]
fn concurrent_macro_and_collection_assignment_cannot_claim_the_same_hotkey() {
    let f = Fixture::new();
    let id = f
        .managers
        .collection_manager
        .create_collection("Work".into())
        .unwrap();
    let macros = f.managers.macro_manager.clone();
    let collections = f.managers.collection_manager.clone();
    let gate = Arc::new(std::sync::Barrier::new(2));
    let first_gate = gate.clone();
    let first = std::thread::spawn(move || {
        first_gate.wait();
        macros.add_macro(legacy_macro("A", "F6", None))
    });
    let second = std::thread::spawn(move || {
        gate.wait();
        collections
            .set_collection_toggle_hotkey(&id, Some("F6".into()))
            .is_ok()
    });
    assert_ne!(first.join().unwrap(), second.join().unwrap());
    assert!(f.managers.macro_manager.hotkey_issues().is_empty());
    assert!(
        f.managers
            .macro_manager
            .process_key_event(117, true, false, false)
            .toggle_request
            .is_some()
    );
}

#[test]
fn deleting_a_legacy_collection_reprojects_child_bindings_without_picking_a_winner() {
    let repositories = memory::repositories();
    repositories
        .collection_repo
        .insert(&legacy_collection("work", Some("B")))
        .unwrap();
    for (key, collection) in [("A", Some("work")), ("B", None)] {
        repositories
            .macro_repo
            .insert(&legacy_macro(key, "F6", collection))
            .unwrap();
    }
    let managers = Managers::new(Arc::new(FakeInput), repositories.clone());
    let macros = &managers.macro_manager;
    assert_eq!(
        macros
            .process_key_event(117, true, false, false)
            .toggle_request,
        Some(ToggleTarget::Macro("B".into()))
    );
    assert!(managers.collection_manager.delete_collection("work"));
    assert!(macros.get_macro("A").unwrap().collection_id.is_none());
    assert!(
        repositories
            .macro_repo
            .load_all()
            .iter()
            .all(|value| value.collection_id.is_none())
    );
    assert_eq!(macros.hotkey_issues().len(), 2);
    assert!(
        macros
            .process_key_event(117, true, false, false)
            .toggle_request
            .is_none()
    );
    assert!(
        macros
            .apply_toggle(ToggleTarget::Macro("B".into()))
            .is_none()
    );
    assert!(macros.set_macro_toggle_hotkey("B", None));
    assert_eq!(
        macros
            .process_key_event(117, true, false, false)
            .toggle_request,
        Some(ToggleTarget::Macro("A".into()))
    );
    assert!(macros.hotkey_issues().is_empty());
    macros.clear_macros();
}

#[test]
fn unassigning_a_legacy_macro_preserves_its_hotkey_and_rejects_new_collisions() {
    let repositories = memory::repositories();
    repositories
        .collection_repo
        .insert(&legacy_collection("work", Some("F6")))
        .unwrap();
    repositories
        .macro_repo
        .insert(&legacy_macro("A", "F6", Some("work")))
        .unwrap();
    let managers = Managers::new(Arc::new(FakeInput), repositories.clone());
    let macros = &managers.macro_manager;
    assert!(!macros.move_macro_to_collection("A", None));
    assert_eq!(
        macros.get_macro("A").unwrap().collection_id.as_deref(),
        Some("work")
    );
    managers
        .collection_manager
        .set_collection_toggle_hotkey("work", None)
        .unwrap();
    assert!(macros.move_macro_to_collection("A", None));
    assert_eq!(
        macros
            .process_key_event(117, true, false, false)
            .toggle_request,
        Some(ToggleTarget::Macro("A".into()))
    );
    assert_eq!(
        repositories.macro_repo.load_all()[0]
            .toggle_enabled_hotkey
            .as_deref(),
        Some("F6")
    );
    assert!(macros.hotkey_issues().is_empty());
    macros.clear_macros();
}

#[test]
fn failed_hotkey_writes_keep_previous_assignments_and_the_active_projection() {
    use std::sync::atomic::{AtomicBool, Ordering};
    let fail = Arc::new(AtomicBool::new(false));
    let repositories = memory::repositories_with_hotkey_failures(fail.clone());
    repositories
        .collection_repo
        .insert(&legacy_collection("work", Some("B")))
        .unwrap();
    repositories
        .macro_repo
        .insert(&legacy_macro("A", "F6", None))
        .unwrap();
    let managers = Managers::new(Arc::new(FakeInput), repositories.clone());
    let macros = &managers.macro_manager;
    fail.store(true, Ordering::Release);
    assert!(!macros.set_macro_toggle_hotkey("A", None));
    assert_eq!(
        managers
            .collection_manager
            .set_collection_toggle_hotkey("work", None),
        Err(crate::application::error::CommandError::OperationFailed)
    );
    assert_eq!(
        macros
            .get_macro("A")
            .unwrap()
            .toggle_enabled_hotkey
            .as_deref(),
        Some("F6")
    );
    assert_eq!(
        repositories.macro_repo.load_all()[0]
            .toggle_enabled_hotkey
            .as_deref(),
        Some("F6")
    );
    assert_eq!(
        repositories.collection_repo.load_all()[0]
            .toggle_enabled_hotkey
            .as_deref(),
        Some("B")
    );
    assert_eq!(
        macros
            .process_key_event(117, true, false, false)
            .toggle_request,
        Some(ToggleTarget::Macro("A".into()))
    );
    assert_eq!(
        macros
            .process_key_event(66, true, false, false)
            .toggle_request,
        Some(ToggleTarget::Collection("work".into()))
    );
    fail.store(false, Ordering::Release);
    assert!(macros.set_macro_toggle_hotkey("A", None));
    assert!(
        macros
            .process_key_event(117, true, false, false)
            .toggle_request
            .is_none()
    );
    macros.clear_macros();
}
