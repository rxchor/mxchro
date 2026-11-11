use super::*;
use crate::application::error::CommandError;
use crate::view::ui::state::field_errors::EditorField;
use gpui_kit::component::{
    WindowExt,
    notification::{Notification, NotificationDelivery},
};

/// Acknowledged requests, not snapshot changes, determine success feedback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PendingOperation {
    SaveMacro,
    DeleteMacro,
    CreateCollection,
    RenameCollection,
    DeleteCollection,
    CollectionHotkey,
    Quiet,
}

impl PendingOperation {
    pub fn for_request(request: &Request) -> Self {
        match request {
            Request::Command(Command::SaveMacro { .. }) => Self::SaveMacro,
            Request::Command(Command::DeleteMacro(_)) => Self::DeleteMacro,
            Request::Command(Command::CreateCollection(_)) => Self::CreateCollection,
            Request::Command(Command::RenameCollection(..)) => Self::RenameCollection,
            Request::Command(Command::DeleteCollection(_)) => Self::DeleteCollection,
            Request::Command(Command::SetCollectionHotkey(..)) => Self::CollectionHotkey,
            _ => Self::Quiet,
        }
    }

    pub fn success_key(self) -> Option<&'static str> {
        match self {
            Self::SaveMacro => Some("notification.macro_saved"),
            Self::DeleteMacro => Some("notification.macro_deleted"),
            Self::CreateCollection => Some("notification.collection_created"),
            Self::RenameCollection => Some("notification.collection_renamed"),
            Self::DeleteCollection => Some("notification.collection_deleted"),
            Self::CollectionHotkey => Some("notification.collection_hotkey_saved"),
            Self::Quiet => None,
        }
    }
}

struct OperationError;
struct OperationSuccess;
struct CaptureListening;

fn editor_field(error: &CommandError) -> Option<EditorField> {
    match error {
        CommandError::InvalidTriggerKey => Some(EditorField::Trigger),
        CommandError::InvalidToggleHotkey
        | CommandError::GroupedMacroHotkey
        | CommandError::ToggleHotkeyInUse(_) => Some(EditorField::Hotkey),
        CommandError::CollectionNotFound => Some(EditorField::Collection),
        CommandError::NoActions | CommandError::InvalidActionKey(_) | CommandError::ZeroWait => {
            Some(EditorField::Actions)
        }
        _ => None,
    }
}

impl MxchroApp {
    pub(super) fn sync_capture_notification(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let session = self.capture_session();
        if self.capture_notification == session {
            return;
        }
        if let Some(previous) = self.capture_notification.take() {
            window.remove_notification1::<CaptureListening>(previous.0 as usize, cx);
        }
        self.capture_notification = session;
        if let Some(session) = session {
            let app = cx.weak_entity();
            window.push_notification(
                Notification::info(text("capture.listening"))
                    .id1::<CaptureListening>(session.0 as usize)
                    .placement(Anchor::TopRight)
                    .delivery(NotificationDelivery::InApp)
                    .autohide(false)
                    .on_close(move |_, cx| {
                        // Programmatic removal fires on_close too. Defer to avoid
                        // re-entering MxchroApp during its poll; never cancel a
                        // newer session when an older toast finishes closing.
                        let app = app.clone();
                        cx.defer(move |cx| {
                            let _ = app.update(cx, |app, cx| {
                                if app.capture_session() == Some(session) {
                                    app.cancel_capture(cx);
                                    cx.notify();
                                }
                            });
                        });
                    }),
                cx,
            );
        }
    }

    pub(super) fn report_error(
        &mut self,
        operation: Option<PendingOperation>,
        error: CommandError,
    ) {
        let field = editor_field(&error);
        let name_error = matches!(
            error,
            CommandError::CollectionNameRequired | CommandError::EmptyCollectionName
        );
        let message = backend_error::message(error);
        if operation == Some(PendingOperation::SaveMacro)
            && let (Some(editor), Some(field)) = (&mut self.editor, field)
        {
            editor.errors.set(field, Some(message));
        } else if matches!(
            operation,
            Some(PendingOperation::CreateCollection | PendingOperation::RenameCollection)
        ) && name_error
            && self.collection_name.target.is_some()
        {
            self.collection_name.error = Some(message);
        } else {
            self.notification_errors.push(message);
        }
    }

    pub(super) fn notify_success(operation: PendingOperation, window: &mut Window, cx: &mut App) {
        if let Some(key) = operation.success_key() {
            window.push_notification(
                Notification::success(text(key))
                    .id1::<OperationSuccess>(key)
                    .placement(Anchor::TopRight)
                    .delivery(NotificationDelivery::InApp)
                    .autohide(true),
                cx,
            );
        }
    }

    pub(super) fn flush_notification_errors(&mut self, window: &mut Window, cx: &mut App) {
        for message in self.notification_errors.drain(..) {
            window.push_notification(
                Notification::error(message.clone())
                    .id1::<OperationError>(SharedString::from(message))
                    .placement(Anchor::TopRight)
                    .delivery(NotificationDelivery::InApp)
                    .autohide(false),
                cx,
            );
        }
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/view/ui/feedback.rs"]
mod tests;
