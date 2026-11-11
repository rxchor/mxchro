mod action_drag;
pub mod action_form;
mod action_motion;
pub mod actions_list;
mod layout;
pub mod macro_form;

use crate::view::i18n::text;
use crate::view::ui::{components::EventHandler, state::MacroDraft};
use crate::view::ui::{components::field_error::field_error, state::field_errors::EditorField};
use action_form::{ActionForm, ActionFormEvent};
use actions_list::ActionsListEvent;
use gpui_kit::{
    base::Disableable,
    component::button::{Button, ButtonVariants},
    *,
};
use macro_form::{MacroForm, MacroFormEvent};
use std::rc::Rc;

pub enum EditorEvent {
    Form(MacroFormEvent),
    AddAction(ActionFormEvent),
    Actions(ActionsListEvent),
    Save,
    Cancel,
}

#[derive(IntoElement)]
pub struct MacroEditorPage {
    pub draft: MacroDraft,
    pub pending: bool,
    pub interaction_locked: bool,
    pub capture_enabled: bool,
    pub on_action: EventHandler<EditorEvent>,
}

impl RenderOnce for MacroEditorPage {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let form = self.on_action.clone();
        let add_action = self.on_action.clone();
        let save = self.on_action.clone();
        let sequence = div()
            .flex_grow(2.)
            .flex_basis(px(460.))
            .min_w_0()
            .flex()
            .flex_col()
            .gap_3()
            .child(div().text_lg().child(text("editor.sequence")))
            .child(self.draft.actions_list.clone())
            .children(
                self.draft
                    .errors
                    .get(EditorField::Actions)
                    .map(|message| field_error(message, cx)),
            );
        let add = div()
            .flex_grow(1.)
            .flex_basis(px(300.))
            .min_w_0()
            .flex()
            .flex_col()
            .gap_3()
            .child(div().text_lg().child(text("editor.add_action")))
            .child(ActionForm {
                errors: self.draft.errors.clone(),
                key_input: self.draft.action_key,
                wait_input: self.draft.wait,
                disabled: self.pending || self.interaction_locked,
                capture_enabled: self.capture_enabled,
                on_action: Rc::new(move |event, window, cx| {
                    add_action(&EditorEvent::AddAction(event.clone()), window, cx)
                }),
            });
        div()
            .w_full()
            .max_w(px(1152.))
            .mx_auto()
            .min_w_0()
            .flex()
            .flex_col()
            .gap_4()
            .child(div().text_2xl().child(if self.draft.old_key.is_some() {
                text("editor.edit")
            } else {
                text("editor.new")
            }))
            .child(MacroForm {
                errors: self.draft.errors.clone(),
                config: self.draft.config,
                trigger: self.draft.trigger,
                hotkey: self.draft.hotkey,
                mode: self.draft.mode,
                collection: self.draft.collection,
                disabled: self.pending || self.interaction_locked,
                capture_enabled: self.capture_enabled,
                on_action: Rc::new(move |event, window, cx| {
                    form(&EditorEvent::Form(event.clone()), window, cx)
                }),
            })
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_start()
                    .gap_6()
                    .child(sequence)
                    .child(add),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .justify_end()
                    .mt_4()
                    .child(
                        Button::new("save")
                            .primary()
                            .label(if self.pending {
                                text("editor.saving")
                            } else {
                                text("editor.save")
                            })
                            .disabled(self.pending || self.interaction_locked)
                            .on_click(move |_, window, cx| save(&EditorEvent::Save, window, cx)),
                    )
                    .child(
                        Button::new("cancel")
                            .label(text("common.cancel"))
                            .disabled(self.pending || self.interaction_locked)
                            .on_click(move |_, window, cx| {
                                (self.on_action)(&EditorEvent::Cancel, window, cx)
                            }),
                    ),
            )
    }
}
