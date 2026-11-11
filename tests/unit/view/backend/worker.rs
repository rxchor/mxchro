use super::*;
use crate as app;
use crate::application::{commands::Command, error::CommandError, queries::Query};
use std::sync::{Arc, mpsc};
#[path = "../../../support/mod.rs"]
mod support;
use support::{Fixture, config};

#[test]
fn worker_preserves_order_errors_saved_flags_and_external_refreshes() {
    let f = Fixture::new();
    let macros = Arc::clone(&f.macros);
    let collections = Arc::clone(&f.collections);
    let dirty = Arc::new(AtomicBool::new(false));
    let worker_dirty = Arc::clone(&dirty);
    let (commands, receiver) = mpsc::channel();
    let (sender, updates) = mpsc::channel();
    let thread =
        std::thread::spawn(move || run(&macros, &collections, receiver, sender, &worker_dirty));
    let next = || updates.recv_timeout(Duration::from_secs(2)).unwrap();
    let initial = next();
    assert!(!initial.request_completed);
    assert!(initial.snapshot.macros.is_empty());
    assert!(!initial.saved);
    assert!(initial.error.is_none());
    commands
        .send(
            Command::SaveMacro {
                old_key: None,
                config: config(),
            }
            .into(),
        )
        .unwrap();
    let mut invalid = config();
    invalid.actions.clear();
    commands
        .send(
            Command::SaveMacro {
                old_key: Some("F6".into()),
                config: invalid,
            }
            .into(),
        )
        .unwrap();
    commands
        .send(Command::CreateCollection("  Work  ".into()).into())
        .unwrap();
    let saved = next();
    assert!(saved.request_completed);
    assert!(saved.saved);
    assert!(saved.error.is_none());
    assert_eq!(saved.snapshot.macros.len(), 1);
    let rejected = next();
    assert!(rejected.request_completed);
    assert!(!rejected.saved);
    assert_eq!(rejected.error, Some(CommandError::NoActions));
    assert_eq!(rejected.snapshot.macros[0].actions_count, 2);
    let changed = next();
    assert!(changed.request_completed);
    assert!(!changed.saved);
    assert!(changed.error.is_none());
    assert_eq!(changed.snapshot.collections[0].name, "Work");
    commands.send(Query::EditMacro("F6".into()).into()).unwrap();
    let editing = next();
    assert!(editing.request_completed);
    assert_eq!(editing.editor.unwrap().actions.len(), 2);
    assert!(!editing.saved);
    commands
        .send(Query::EditMacro("missing".into()).into())
        .unwrap();
    let missing = next();
    assert!(missing.request_completed);
    assert!(missing.editor.is_none());
    assert_eq!(missing.error, Some(CommandError::MacroNotFound));
    assert!(f.macros.enable_macro("F6"));
    dirty.store(true, Ordering::Release);
    let refreshed = next();
    assert!(!refreshed.request_completed);
    assert!(refreshed.snapshot.macros[0].enabled);
    assert!(!refreshed.saved);
    assert!(refreshed.error.is_none());
    drop(commands);
    thread.join().unwrap();
}

#[test]
fn worker_exits_when_initial_update_receiver_is_gone() {
    let f = Fixture::new();
    let (_commands, receiver) = mpsc::channel();
    let (sender, updates) = mpsc::channel();
    drop(updates);
    run(
        &f.macros,
        &f.collections,
        receiver,
        sender,
        &AtomicBool::new(false),
    );
    assert!(f.macros.list_macros().is_empty());
}
