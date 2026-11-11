use app::application::{
    commands::{Command, CommandOutcome, macros::validate},
    error::CommandError,
    queries,
};
use app::domain::types::Action;
use mxchro as app;
mod support;
use support::{Fixture, config};

#[test]
fn accepts_press_release_sequence() {
    assert_eq!(validate(&config(), &["F6".into(), "A".into()]), Ok(()));
}

#[test]
fn grouped_macro_hotkeys_are_rejected_without_mutating_existing_data() {
    let f = Fixture::new();
    f.execute(Command::CreateCollection("Work".into())).unwrap();
    let id = f.collections.list_collections()[0].id.clone();
    let mut value = config();
    value.toggle_enabled_hotkey = Some("A".into());
    f.execute(Command::SaveMacro {
        old_key: None,
        config: value.clone(),
    })
    .unwrap();
    value.collection_id = Some(id.clone());
    assert_eq!(
        f.execute(Command::SaveMacro {
            old_key: Some("F6".into()),
            config: value.clone()
        }),
        Err(CommandError::GroupedMacroHotkey)
    );
    assert!(f.macros.get_macro("F6").unwrap().collection_id.is_none());
    assert_eq!(
        f.persistence.macro_repo.load_all()[0]
            .toggle_enabled_hotkey
            .as_deref(),
        Some("A")
    );
    assert!(!f.macros.move_macro_to_collection("F6", Some(id)));
    value.toggle_enabled_hotkey = None;
    f.execute(Command::SaveMacro {
        old_key: Some("F6".into()),
        config: value,
    })
    .unwrap();
    assert!(!f.macros.set_macro_toggle_hotkey("F6", Some("A".into())));
    assert!(
        f.macros
            .process_key_event(65, true, false, false)
            .toggle_request
            .is_none()
    );
}

#[test]
fn duplicate_toggle_hotkeys_are_rejected_across_all_owner_types_and_self_edits_are_allowed() {
    let f = Fixture::new();
    let mut value = config();
    value.toggle_enabled_hotkey = Some("A".into());
    f.execute(Command::SaveMacro {
        old_key: None,
        config: value.clone(),
    })
    .unwrap();
    let mut duplicate = value.clone();
    duplicate.trigger_key = "B".into();
    assert_eq!(
        f.execute(Command::SaveMacro {
            old_key: None,
            config: duplicate.clone()
        }),
        Err(CommandError::ToggleHotkeyInUse("A".into()))
    );
    assert_eq!(f.macros.list_macros().len(), 1);
    f.execute(Command::CreateCollection("Work".into())).unwrap();
    let work = f.collections.list_collections()[0].id.clone();
    f.execute(Command::SetCollectionHotkey(work.clone(), Some("B".into())))
        .unwrap();
    assert_eq!(
        f.execute(Command::SetCollectionHotkey(work.clone(), Some("A".into()))),
        Err(CommandError::ToggleHotkeyInUse("A".into()))
    );
    assert_eq!(
        f.persistence.collection_repo.load_all()[0]
            .toggle_enabled_hotkey
            .as_deref(),
        Some("B")
    );
    f.execute(Command::SetCollectionHotkey(work.clone(), Some("B".into())))
        .unwrap();
    duplicate.toggle_enabled_hotkey = Some("B".into());
    assert_eq!(
        f.execute(Command::SaveMacro {
            old_key: None,
            config: duplicate
        }),
        Err(CommandError::ToggleHotkeyInUse("B".into()))
    );
    f.execute(Command::CreateCollection("Other".into()))
        .unwrap();
    let other = f
        .collections
        .list_collections()
        .into_iter()
        .find(|collection| collection.id != work)
        .unwrap()
        .id;
    assert_eq!(
        f.execute(Command::SetCollectionHotkey(other, Some("B".into()))),
        Err(CommandError::ToggleHotkeyInUse("B".into()))
    );
    value.trigger_key = "B".into();
    f.execute(Command::SaveMacro {
        old_key: Some("F6".into()),
        config: value.clone(),
    })
    .unwrap();
    f.execute(Command::SaveMacro {
        old_key: Some("B".into()),
        config: value,
    })
    .unwrap();
    assert!(
        queries::snapshot(&f.macros, &f.collections)
            .hotkey_issues
            .is_empty()
    );
}

#[test]
fn rejects_unknown_trigger_action_and_hotkey_with_typed_errors() {
    let keys = vec!["F6".into(), "A".into()];
    let mut value = config();
    value.trigger_key = "unknown".into();
    assert_eq!(
        validate(&value, &keys),
        Err(CommandError::InvalidTriggerKey)
    );
    value = config();
    value.actions.push(Action::Key {
        key: "unknown".into(),
        press: true,
    });
    assert_eq!(
        validate(&value, &keys),
        Err(CommandError::InvalidActionKey("unknown".into()))
    );
    value = config();
    value.toggle_enabled_hotkey = Some("unknown".into());
    assert_eq!(
        validate(&value, &keys),
        Err(CommandError::InvalidToggleHotkey)
    );
}

#[test]
fn rejects_empty_actions_and_zero_wait() {
    let keys = vec!["F6".into(), "A".into()];
    let mut value = config();
    value.actions.clear();
    assert_eq!(validate(&value, &keys), Err(CommandError::NoActions));
    value.actions.push(Action::Wait { duration_ms: 0 });
    assert_eq!(validate(&value, &keys), Err(CommandError::ZeroWait));
}

#[test]
fn saves_edits_and_deletes_macros_through_use_cases() {
    let f = Fixture::new();
    assert_eq!(
        f.execute(Command::SaveMacro {
            old_key: None,
            config: config()
        }),
        Ok(CommandOutcome::MacroSaved)
    );
    assert!(!f.macros.get_macro("F6").unwrap().enabled);
    assert_eq!(
        f.execute(Command::SaveMacro {
            old_key: None,
            config: config()
        }),
        Err(CommandError::OperationFailed)
    );
    let mut edited = config();
    edited.trigger_key = "B".into();
    assert_eq!(
        f.execute(Command::SaveMacro {
            old_key: Some("F6".into()),
            config: edited
        }),
        Ok(CommandOutcome::MacroSaved)
    );
    assert!(f.macros.get_macro("F6").is_none());
    assert!(matches!(
        &f.macros.get_macro("B").unwrap().actions[..],
        [
            Action::Key { press: true, .. },
            Action::Key { press: false, .. }
        ]
    ));
    let stored = f.persistence.macro_repo.load_all();
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].trigger_key, "B");
    assert_eq!(
        f.execute(Command::SetMacroEnabled("B".into(), true)),
        Ok(CommandOutcome::Changed)
    );
    assert!(f.persistence.macro_repo.load_all()[0].enabled);
    f.execute(Command::SetMacroEnabled("B".into(), false))
        .unwrap();
    f.execute(Command::DeleteMacro("B".into())).unwrap();
    assert!(f.persistence.macro_repo.load_all().is_empty());
}

#[test]
fn validates_collection_names_and_missing_collection_before_mutating() {
    let f = Fixture::new();
    assert_eq!(
        f.execute(Command::CreateCollection("  ".into())),
        Err(CommandError::CollectionNameRequired)
    );
    assert_eq!(
        f.execute(Command::RenameCollection("missing".into(), "  ".into())),
        Err(CommandError::EmptyCollectionName)
    );
    let mut value = config();
    value.collection_id = Some("missing".into());
    assert_eq!(
        f.execute(Command::SaveMacro {
            old_key: None,
            config: value
        }),
        Err(CommandError::CollectionNotFound)
    );
    assert!(f.macros.list_macros().is_empty());
    assert!(f.persistence.macro_repo.load_all().is_empty());
}

#[test]
fn collection_use_cases_preserve_associated_macros_and_persistence() {
    let f = Fixture::new();
    f.execute(Command::CreateCollection("  Work  ".into()))
        .unwrap();
    let collection = f.collections.list_collections().remove(0);
    assert_eq!(collection.name, "Work");
    let mut value = config();
    value.collection_id = Some(collection.id.clone());
    f.execute(Command::SaveMacro {
        old_key: None,
        config: value,
    })
    .unwrap();
    f.execute(Command::RenameCollection(
        collection.id.clone(),
        "  Games  ".into(),
    ))
    .unwrap();
    assert_eq!(f.collections.list_collections()[0].name, "Games");
    f.execute(Command::SetCollectionEnabled(collection.id.clone(), true))
        .unwrap();
    assert!(f.persistence.macro_repo.load_all()[0].enabled);
    f.execute(Command::SetCollectionEnabled(collection.id.clone(), false))
        .unwrap();
    assert!(!f.persistence.macro_repo.load_all()[0].enabled);
    f.execute(Command::DeleteCollection(collection.id)).unwrap();
    assert!(f.collections.list_collections().is_empty());
    assert!(f.persistence.collection_repo.load_all().is_empty());
    assert!(f.macros.get_macro("F6").unwrap().collection_id.is_none());
    assert!(
        f.persistence.macro_repo.load_all()[0]
            .collection_id
            .is_none()
    );
}

#[test]
fn collection_hotkey_assign_replace_clear_and_delete_clean_the_input_index() {
    let f = Fixture::new();
    f.execute(Command::CreateCollection("Work".into())).unwrap();
    let id = f.collections.list_collections()[0].id.clone();
    let mut value = config();
    value.collection_id = Some(id.clone());
    f.execute(Command::SaveMacro {
        old_key: None,
        config: value,
    })
    .unwrap();

    assert_eq!(
        f.execute(Command::SetCollectionHotkey(id.clone(), Some("A".into()))),
        Ok(CommandOutcome::Changed)
    );
    assert_eq!(
        f.persistence.collection_repo.load_all()[0]
            .toggle_enabled_hotkey
            .as_deref(),
        Some("A")
    );
    assert_eq!(
        f.collections.list_collections()[0]
            .toggle_enabled_hotkey
            .as_deref(),
        Some("A")
    );
    // Input is simulated at the domain boundary: no hooks or real key injection.
    assert!(
        f.macros
            .process_key_event(65, true, false, true)
            .toggle_request
            .is_none()
    );
    assert!(
        f.macros
            .process_key_event(65, false, true, false)
            .toggle_request
            .is_none()
    );
    let request = f.macros.process_key_event(65, true, false, false);
    assert!(request.should_block);
    let enabled = f
        .macros
        .apply_toggle(request.toggle_request.unwrap())
        .unwrap()
        .enabled;
    assert_eq!(f.collections.list_collections()[0].enabled, enabled);
    assert_eq!(f.persistence.macro_repo.load_all()[0].enabled, enabled);
    let request = f
        .macros
        .process_key_event(65, true, false, false)
        .toggle_request
        .unwrap();
    assert_eq!(f.macros.apply_toggle(request).unwrap().enabled, !enabled);
    assert_eq!(f.persistence.macro_repo.load_all()[0].enabled, !enabled);

    f.execute(Command::SetCollectionHotkey(id.clone(), Some("B".into())))
        .unwrap();
    assert!(
        f.macros
            .process_key_event(65, true, false, false)
            .toggle_request
            .is_none()
    );
    assert!(
        f.macros
            .process_key_event(66, true, false, false)
            .toggle_request
            .is_some()
    );
    assert_eq!(
        f.persistence.collection_repo.load_all()[0]
            .toggle_enabled_hotkey
            .as_deref(),
        Some("B")
    );
    f.execute(Command::SetCollectionHotkey(id.clone(), None))
        .unwrap();
    assert!(
        f.macros
            .process_key_event(66, true, false, false)
            .toggle_request
            .is_none()
    );
    assert_eq!(
        f.persistence.collection_repo.load_all()[0].toggle_enabled_hotkey,
        None
    );
    assert_eq!(
        f.collections.list_collections()[0].toggle_enabled_hotkey,
        None
    );

    f.execute(Command::SetCollectionHotkey(id.clone(), Some("A".into())))
        .unwrap();
    f.execute(Command::DeleteCollection(id)).unwrap();
    assert!(
        f.macros
            .process_key_event(65, true, false, false)
            .toggle_request
            .is_none()
    );
}

#[test]
fn collection_hotkey_validation_preserves_the_existing_assignment() {
    let f = Fixture::new();
    f.execute(Command::CreateCollection("Work".into())).unwrap();
    let id = f.collections.list_collections()[0].id.clone();
    f.execute(Command::SetCollectionHotkey(id.clone(), Some("A".into())))
        .unwrap();
    for key in ["", "a", "Ctrl+F6", "not-a-key"] {
        assert_eq!(
            f.execute(Command::SetCollectionHotkey(id.clone(), Some(key.into()))),
            Err(CommandError::InvalidToggleHotkey)
        );
        assert_eq!(
            f.persistence.collection_repo.load_all()[0]
                .toggle_enabled_hotkey
                .as_deref(),
            Some("A")
        );
        assert!(
            f.macros
                .process_key_event(65, true, false, false)
                .toggle_request
                .is_some()
        );
    }
    assert_eq!(
        f.execute(Command::SetCollectionHotkey(
            "missing".into(),
            Some("B".into())
        )),
        Err(CommandError::CollectionNotFound)
    );
    assert_eq!(
        f.execute(Command::SetCollectionHotkey("missing".into(), None)),
        Err(CommandError::CollectionNotFound)
    );
    assert!(
        f.macros
            .process_key_event(66, true, false, false)
            .toggle_request
            .is_none()
    );
}

#[test]
fn snapshot_returns_sorted_configs_collections_and_canonical_keys() {
    let f = Fixture::new();
    for key in ["B", "A"] {
        let mut value = config();
        value.trigger_key = key.into();
        f.execute(Command::SaveMacro {
            old_key: None,
            config: value,
        })
        .unwrap();
    }
    f.execute(Command::CreateCollection("Work".into())).unwrap();
    let snapshot = queries::snapshot(&f.macros, &f.collections);
    assert_eq!(
        snapshot
            .macros
            .iter()
            .map(|m| m.trigger_key.as_str())
            .collect::<Vec<_>>(),
        ["A", "B"]
    );
    assert_eq!(snapshot.collections[0].name, "Work");
    assert_eq!(snapshot.keys, ["A", "B", "F6"]);
}

#[test]
fn summaries_and_edit_details_are_separate_and_save_dto_preserves_the_sequence() {
    let f = Fixture::new();
    let mut value = config();
    value.actions = (1..=500)
        .map(|duration_ms| Action::Wait { duration_ms })
        .collect();
    f.execute(Command::SaveMacro {
        old_key: None,
        config: value,
    })
    .unwrap();
    let snapshot = queries::snapshot(&f.macros, &f.collections);
    assert_eq!(snapshot.macros[0].actions_count, 500);
    let mut detail = queries::macro_for_editing(&f.macros, "F6").unwrap();
    assert_eq!(detail.actions.len(), 500);
    detail.actions.rotate_left(1);
    assert!(matches!(
        f.macros.get_macro("F6").unwrap().actions[0],
        Action::Wait { duration_ms: 1 }
    ));
    f.execute(Command::SaveMacro {
        old_key: Some("F6".into()),
        config: detail.into(),
    })
    .unwrap();
    let stored = f.persistence.macro_repo.load_all().remove(0);
    assert!(matches!(stored.actions[0], Action::Wait { duration_ms: 2 }));
    assert!(matches!(
        stored.actions[499],
        Action::Wait { duration_ms: 1 }
    ));
    assert!(matches!(
        queries::macro_for_editing(&f.macros, "missing"),
        Err(CommandError::MacroNotFound)
    ));
}
