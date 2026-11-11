use super::{PendingOperation, editor_field};
use crate::{
    application::{commands::Command, error::CommandError, queries::Query},
    view::{i18n::text, ui::state::field_errors::EditorField},
};

#[test]
fn only_acknowledged_edit_operations_have_success_messages() {
    assert_eq!(
        PendingOperation::for_request(&Command::SetMacroEnabled("A".into(), true).into())
            .success_key(),
        None
    );
    assert_eq!(
        PendingOperation::for_request(&Command::SetCollectionEnabled("id".into(), true).into())
            .success_key(),
        None
    );
    assert_eq!(
        PendingOperation::for_request(&Query::EditMacro("A".into()).into()).success_key(),
        None
    );
    for operation in [
        PendingOperation::SaveMacro,
        PendingOperation::DeleteMacro,
        PendingOperation::CreateCollection,
        PendingOperation::RenameCollection,
        PendingOperation::DeleteCollection,
        PendingOperation::CollectionHotkey,
    ] {
        let key = operation.success_key().unwrap();
        assert_ne!(text(key), key);
    }
}

#[test]
fn field_validation_is_distinct_from_operation_errors() {
    assert_eq!(
        editor_field(&CommandError::InvalidTriggerKey),
        Some(EditorField::Trigger)
    );
    assert_eq!(
        editor_field(&CommandError::InvalidToggleHotkey),
        Some(EditorField::Hotkey)
    );
    assert_eq!(
        editor_field(&CommandError::NoActions),
        Some(EditorField::Actions)
    );
    assert_eq!(editor_field(&CommandError::OperationFailed), None);
    let mut errors = crate::view::ui::state::field_errors::FieldErrors::default();
    errors.set(EditorField::Mode, Some("mode".into()));
    errors.set(EditorField::Trigger, Some("trigger".into()));
    errors.set(EditorField::Mode, None);
    assert_eq!(errors.get(EditorField::Mode), None);
    assert_eq!(errors.get(EditorField::Trigger).as_deref(), Some("trigger"));
    errors.clear();
    assert_eq!(errors.get(EditorField::Trigger), None);
}
