use super::*;
use crate::core::persistence::{CollectionRepository, MacroRepository};
use crate::core::types::{Action, Macro, Mode};
use polodb_core::Database;
use std::path::PathBuf;
use std::sync::{mpsc, Mutex};
use std::time::Duration;

struct TestDirectory(PathBuf);

impl Drop for TestDirectory {
    fn drop(&mut self) {
        // Solo la carpeta exclusiva creada por este test, nunca la BD del usuario.
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

struct Fixture {
    managers: Managers,
    macros: Arc<RwLock<HashMap<u16, MacroInstance>>>,
    // Los repositorios deben cerrarse antes de borrar el directorio en Windows.
    _directory: TestDirectory,
}

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir()
            .join("opencode")
            .join(format!("mxchro-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&path).unwrap();
        let db = Database::open_path(&path).unwrap();
        let persistence = Arc::new(Persistence {
            macro_repo: MacroRepository::new(db.clone()),
            collection_repo: CollectionRepository::new(db),
        });
        let collections = Arc::new(RwLock::new(HashMap::new()));
        let macros = Arc::new(RwLock::new(HashMap::new()));
        let hotkeys = Arc::new(RwLock::new(HashMap::new()));
        let mapper = Arc::new(KeyMapper::new());
        let collection_manager = Arc::new(CollectionManager::new(
            collections,
            Arc::clone(&macros),
            Arc::clone(&hotkeys),
            Arc::clone(&mapper),
            Arc::clone(&persistence),
        ));
        let macro_manager = Arc::new(MacroManager::new(
            Arc::clone(&macros),
            Arc::clone(&collection_manager),
            hotkeys,
            mapper,
            Arc::clone(&persistence),
        ));
        Self {
            managers: Managers {
                persistence,
                collection_manager,
                macro_manager,
            },
            macros,
            _directory: TestDirectory(path),
        }
    }

    fn add_macro(&self, key: &str, collection_id: Option<&str>, enabled: bool) {
        assert!(self.managers.macro_manager.add_macro(Macro {
            trigger_key: key.into(),
            mode: Mode::Once,
            actions: vec![Action::Wait { duration_ms: 1 }],
            block_original_event: true,
            ignore_injected_keys: true,
            enabled,
            collection_id: collection_id.map(str::to_owned),
            toggle_enabled_hotkey: None,
        }));
    }

    fn assert_enabled(&self, key: &str, enabled: bool) {
        assert_eq!(
            self.managers.macro_manager.get_macro(key).unwrap().enabled,
            enabled
        );
        let stored = self.managers.persistence.macro_repo.load_all();
        assert_eq!(
            stored
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
fn collection_state_is_persisted_for_members_only() {
    let fixture = Fixture::new();
    let manager = &fixture.managers.collection_manager;
    let id = manager.create_collection("Test".into()).unwrap();
    fixture.add_macro("A", Some(&id), true);
    fixture.add_macro("B", None, true);
    assert!(manager.disable_collection(&id));
    fixture.assert_enabled("A", false);
    fixture.assert_enabled("B", true);
    assert!(!fixture.managers.persistence.collection_repo.load_all()[0].enabled);

    assert!(manager.enable_collection(&id));
    fixture.assert_enabled("A", true);
    assert!(fixture.managers.persistence.collection_repo.load_all()[0].enabled);

    assert_eq!(manager.toggle_collection_enabled(&id), Some(false));
    fixture.assert_enabled("A", false);
    assert_eq!(manager.toggle_collection_enabled(&id), Some(true));
    fixture.assert_enabled("A", true);
    assert!(!manager.enable_collection("missing"));
    assert!(!manager.disable_collection("missing"));
    assert_eq!(manager.toggle_collection_enabled("missing"), None);
}

#[test]
fn hook_defers_toggles_and_never_waits_for_the_macro_map() {
    let fixture = Fixture::new();
    fixture.add_macro("A", None, true);
    let manager = Arc::clone(&fixture.managers.macro_manager);
    assert!(manager.set_macro_toggle_hotkey("A", Some("F1".into())));
    let mapper = KeyMapper::new();
    let a = mapper.string_to_vk("A").unwrap();
    let f1 = mapper.string_to_vk("F1").unwrap();

    let locked = fixture.macros.write().unwrap();
    let (sender, receiver) = mpsc::channel();
    let hook = std::thread::spawn(move || {
        let trigger = manager.process_key_event(a, true, false, false);
        let injected = manager.process_key_event(a, true, false, true);
        let request = manager.process_key_event(f1, true, false, false);
        let key_up = manager.process_key_event(f1, false, true, false);
        sender.send((trigger, injected, request, key_up)).unwrap();
    });
    let result = receiver.recv_timeout(Duration::from_secs(1));
    drop(locked); // Liberar incluso si la comprobación falla.
    hook.join().unwrap();
    let (trigger, injected, request, key_up) = result.expect("el hook esperó el lock de macros");
    assert!(trigger.should_block);
    assert!(!injected.should_block);
    assert!(key_up.toggle_request.is_none());
    assert!(request.should_block);
    fixture.assert_enabled("A", true); // El hook no ejecutó el toggle.
    let event = fixture
        .managers
        .macro_manager
        .apply_toggle(request.toggle_request.unwrap())
        .unwrap();
    assert!(!event.enabled);
    fixture.assert_enabled("A", false);
    assert!(
        !fixture
            .managers
            .macro_manager
            .process_key_event(a, true, false, false)
            .should_block
    );
}

#[test]
fn collection_hotkey_uses_the_same_persisted_transition() {
    let fixture = Fixture::new();
    let id = fixture
        .managers
        .collection_manager
        .create_collection("Test".into())
        .unwrap();
    fixture.add_macro("A", Some(&id), true);
    assert!(fixture
        .managers
        .collection_manager
        .set_collection_toggle_hotkey(&id, Some("F2".into())));
    let vk = KeyMapper::new().string_to_vk("F2").unwrap();
    let manager = &fixture.managers.macro_manager;
    assert!(manager
        .process_key_event(vk, true, false, true)
        .toggle_request
        .is_none());
    let request = manager
        .process_key_event(vk, true, false, false)
        .toggle_request
        .unwrap();
    assert!(!manager.apply_toggle(request).unwrap().enabled);
    fixture.assert_enabled("A", false);
    assert!(fixture.managers.collection_manager.delete_collection(&id));
    let result = manager.process_key_event(vk, true, false, false);
    assert!(!result.should_block);
    assert!(result.toggle_request.is_none());
}

#[test]
fn hook_remains_responsive_while_toggle_joins_a_running_macro() {
    let fixture = Fixture::new();
    fixture.add_macro("A", None, false);
    let vk = KeyMapper::new().string_to_vk("A").unwrap();
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Mutex::new(release_rx);
    fixture.macros.write().unwrap().get_mut(&vk).unwrap().action = Arc::new(move || {
        started_tx.send(()).unwrap();
        let _ = release_rx.lock().unwrap().recv();
    });
    let manager = Arc::clone(&fixture.managers.macro_manager);
    assert!(manager.enable_macro("A"));
    manager.process_key_event(vk, true, false, false);
    started_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    let toggler = Arc::clone(&manager);
    let (done_tx, done_rx) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        done_tx
            .send(
                toggler
                    .apply_toggle(ToggleTarget::Macro("A".into()))
                    .unwrap()
                    .enabled,
            )
            .unwrap();
    });
    // Esperar a que disable publique el estado de entrada, sin acceder al mapa
    // que el worker mantiene bloqueado mientras espera el callback.
    let (hook_tx, hook_rx) = mpsc::channel();
    let hook = std::thread::spawn(move || {
        while manager
            .process_key_event(vk, false, true, false)
            .should_block
        {
            std::thread::yield_now();
        }
        hook_tx.send(()).unwrap();
    });
    let responsive = hook_rx.recv_timeout(Duration::from_secs(1));
    assert!(done_rx.try_recv().is_err());
    release_tx.send(()).unwrap();
    hook.join().unwrap();
    worker.join().unwrap();
    responsive.expect("el hook quedó bloqueado por el join");
    assert!(!done_rx.recv().unwrap());
}

#[test]
fn input_index_preserves_hold_and_toggle_self_trigger_protection() {
    let fixture = Fixture::new();
    let manager = &fixture.managers.macro_manager;
    for (key, mode) in [("A", Mode::Hold), ("B", Mode::Toggle)] {
        fixture.add_macro(key, None, false);
        let mut config = manager.get_macro(key).unwrap();
        config.mode = mode.clone();
        config.ignore_injected_keys = false;
        config.enabled = true;
        assert!(manager.update_macro(key, config));
        let vk = KeyMapper::new().string_to_vk(key).unwrap();
        let state = Arc::clone(
            &fixture
                .macros
                .read()
                .unwrap()
                .get(&vk)
                .unwrap()
                .repeat_state,
        );
        assert!(
            manager
                .process_key_event(vk, true, false, false)
                .should_block
        );
        assert!(state.0.lock().unwrap().active);

        let injected_down = mode == Mode::Toggle;
        let result = manager.process_key_event(vk, injected_down, !injected_down, true);
        assert!(!result.should_block);
        assert!(state.0.lock().unwrap().active);
        manager.process_key_event(vk, injected_down, !injected_down, false);
        assert!(!state.0.lock().unwrap().active);
    }
}

#[test]
fn editing_and_deleting_macros_refresh_the_hook_index() {
    let fixture = Fixture::new();
    fixture.add_macro("A", None, true);
    let manager = &fixture.managers.macro_manager;
    let mapper = KeyMapper::new();
    let a = mapper.string_to_vk("A").unwrap();
    let b = mapper.string_to_vk("B").unwrap();
    let mut config = manager.get_macro("A").unwrap();
    config.trigger_key = "B".into();
    config.toggle_enabled_hotkey = Some("F1".into());
    assert!(manager.update_macro("A", config));
    assert!(
        !manager
            .process_key_event(a, false, true, false)
            .should_block
    );
    assert!(
        manager
            .process_key_event(b, false, true, false)
            .should_block
    );
    assert!(manager.disable_macro("B"));
    assert!(
        !manager
            .process_key_event(b, false, true, false)
            .should_block
    );
    assert!(manager.enable_macro("B"));
    assert!(
        manager
            .process_key_event(b, false, true, false)
            .should_block
    );
    assert!(manager.delete_macro("B"));
    assert!(
        !manager
            .process_key_event(b, false, true, false)
            .should_block
    );
    let f1 = mapper.string_to_vk("F1").unwrap();
    assert!(manager
        .process_key_event(f1, true, false, false)
        .toggle_request
        .is_none());
}

#[test]
fn wiping_data_clears_trigger_and_hotkey_indices() {
    let fixture = Fixture::new();
    let id = fixture
        .managers
        .collection_manager
        .create_collection("Test".into())
        .unwrap();
    fixture.add_macro("A", Some(&id), true);
    assert!(fixture
        .managers
        .macro_manager
        .set_macro_toggle_hotkey("A", Some("F1".into())));
    assert!(fixture
        .managers
        .collection_manager
        .set_collection_toggle_hotkey(&id, Some("F2".into())));
    fixture.managers.wipe_data().unwrap();
    for key in ["A", "F1", "F2"] {
        let vk = KeyMapper::new().string_to_vk(key).unwrap();
        let result = fixture
            .managers
            .macro_manager
            .process_key_event(vk, true, false, false);
        assert!(!result.should_block);
        assert!(result.toggle_request.is_none());
    }
}

#[test]
fn listing_and_changing_collections_complete_concurrently() {
    let fixture = Fixture::new();
    let manager = Arc::clone(&fixture.managers.collection_manager);
    let id = manager.create_collection("Test".into()).unwrap();
    fixture.add_macro("A", Some(&id), true);
    let reader = Arc::clone(&manager);
    let (sender, receiver) = mpsc::channel();
    let listing = std::thread::spawn(move || {
        for _ in 0..100 {
            assert_eq!(reader.list_collections().len(), 1);
        }
        sender.send(()).unwrap();
    });
    for _ in 0..20 {
        assert!(manager.disable_collection(&id));
        assert!(manager.enable_collection(&id));
    }
    receiver.recv_timeout(Duration::from_secs(5)).unwrap();
    listing.join().unwrap();
}
