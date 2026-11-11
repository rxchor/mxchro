//! In-process interface adapter. This module does not assemble native dependencies.
pub(crate) mod error;
pub mod settings;
pub(crate) mod worker;
use crate::application::{
    dto::MacroEditDTO, error::CommandError, queries::Snapshot, request::Request,
};
use std::sync::mpsc::{Receiver, Sender};

pub struct Update {
    /// A response to a queued request, not an unsolicited snapshot refresh.
    pub request_completed: bool,
    pub snapshot: Snapshot,
    pub error: Option<CommandError>,
    pub saved: bool,
    pub editor: Option<MacroEditDTO>,
}

pub struct CommandBus {
    pub requests: Sender<Request>,
    pub updates: Receiver<Update>,
}
