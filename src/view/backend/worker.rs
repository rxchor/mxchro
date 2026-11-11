//! Ordered, asynchronous transport. Use cases live in application/commands, not here.
use super::Update;
use crate::{
    application::engine::{collections::CollectionManager, macros::MacroManager},
    application::{
        commands::{self, CommandOutcome},
        queries::{self, Query},
        request::Request,
    },
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    time::Duration,
};

/// Injectable worker loop: tests use fake input and exclusive temporary persistence,
/// never the production assembly above (which installs global Windows hooks).
pub(crate) fn run(
    macros: &MacroManager,
    collections: &CollectionManager,
    commands: Receiver<Request>,
    updates: Sender<Update>,
    dirty: &AtomicBool,
) {
    let publish = |request_completed, error, saved, editor| {
        updates
            .send(Update {
                request_completed,
                snapshot: queries::snapshot(macros, collections),
                error,
                saved,
                editor,
            })
            .is_ok()
    };
    if !publish(false, None, false, None) {
        return;
    }
    loop {
        match commands.recv_timeout(Duration::from_millis(100)) {
            Ok(Request::Command(command)) => {
                let result = commands::execute(macros, collections, command);
                let saved = matches!(result, Ok(CommandOutcome::MacroSaved));
                if !publish(true, result.err(), saved, None) {
                    break;
                }
            }
            Ok(Request::Query(Query::EditMacro(key))) => {
                let result = queries::macro_for_editing(macros, &key);
                let (editor, error) = match result {
                    Ok(editor) => (Some(editor), None),
                    Err(error) => (None, Some(error)),
                };
                if !publish(true, error, false, editor) {
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if dirty.swap(false, Ordering::AcqRel) && !publish(false, None, false, None) {
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/view/backend/worker.rs"]
mod tests;
