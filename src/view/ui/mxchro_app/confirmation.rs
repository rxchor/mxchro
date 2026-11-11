use super::*;
use crate::view::ui::components::{AppConfirmDialog, ConfirmDialogEvent};

#[derive(Clone)]
pub(super) enum DeleteTarget {
    Macro(String),
    Collection(String),
}

pub(super) struct PendingConfirmation {
    id: u64,
    target: DeleteTarget,
}

impl MxchroApp {
    fn can_delete(&self, target: &DeleteTarget) -> bool {
        if self.pending || !self.ready || self.capture.is_some() || self.settings_open {
            return false;
        }
        match target {
            DeleteTarget::Macro(key) => {
                self.editor.is_none()
                    && self
                        .data
                        .macros
                        .iter()
                        .any(|config| &config.trigger_key == key)
            }
            DeleteTarget::Collection(id) => {
                self.editor.is_none()
                    && self
                        .data
                        .collections
                        .iter()
                        .any(|collection| &collection.id == id)
            }
        }
    }

    pub(super) fn confirm_deletion(
        &mut self,
        target: DeleteTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.confirmation.is_some() || !self.can_delete(&target) {
            return;
        }
        let (title, message, label) = match &target {
            DeleteTarget::Macro(key) => (
                text("confirm.macro_title"),
                rust_i18n::t!(
                    "confirm.macro_message",
                    key = crate::view::i18n::key_label(key)
                )
                .into_owned(),
                text("common.delete"),
            ),
            DeleteTarget::Collection(id) => {
                let Some(collection) = self
                    .data
                    .collections
                    .iter()
                    .find(|collection| &collection.id == id)
                else {
                    return;
                };
                (
                    text("confirm.collection_title"),
                    rust_i18n::t!("confirm.collection_message", name = &collection.name)
                        .into_owned(),
                    text("common.delete"),
                )
            }
        };
        self.collection_name.close();
        self.confirmation_id = self.confirmation_id.wrapping_add(1);
        let id = self.confirmation_id;
        self.confirmation = Some(PendingConfirmation { id, target });
        self.sync_actions_disabled(cx);
        AppConfirmDialog::new(title, message, label).open(
            Rc::new(
                cx.listener(move |app, decision, _, cx| {
                    app.resolve_confirmation(id, *decision, cx)
                }),
            ),
            window,
            cx,
        );
        cx.notify();
    }

    fn resolve_confirmation(
        &mut self,
        id: u64,
        decision: ConfirmDialogEvent,
        cx: &mut Context<Self>,
    ) {
        if self.confirmation.as_ref().map(|pending| pending.id) != Some(id) {
            return;
        }
        let pending = self.confirmation.take().expect("Matching confirmation");
        self.sync_actions_disabled(cx);
        if decision == ConfirmDialogEvent::Confirm && self.can_delete(&pending.target) {
            match pending.target {
                DeleteTarget::Macro(key) => self.send(Command::DeleteMacro(key), cx),
                DeleteTarget::Collection(id) => self.send(Command::DeleteCollection(id), cx),
            }
        }
        cx.notify();
    }

    pub(super) fn reconcile_confirmation(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self
            .confirmation
            .as_ref()
            .is_some_and(|pending| !self.can_delete(&pending.target))
        {
            self.confirmation = None;
            self.sync_actions_disabled(cx);
            window.close_dialog(cx);
            cx.notify();
        }
    }
}
