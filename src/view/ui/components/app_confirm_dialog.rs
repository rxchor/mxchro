//! Shared destructive confirmation. Root owns the actual dialog/overlay/focus trap.
use super::EventHandler;
use crate::view::i18n::text;
use gpui_kit::{
    component::{
        WindowExt,
        button::{Button, ButtonVariants},
        dialog::{DialogEntrance, DialogFooter},
    },
    *,
};
use std::{cell::Cell, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfirmDialogEvent {
    Confirm,
    Cancel,
}

#[derive(Default)]
struct DecisionGate(Cell<bool>);

impl DecisionGate {
    fn decide(&self, decision: ConfirmDialogEvent) -> Option<ConfirmDialogEvent> {
        (!self.0.replace(true)).then_some(decision)
    }
}

pub struct AppConfirmDialog {
    title: String,
    message: String,
    confirm_label: String,
}

impl AppConfirmDialog {
    pub fn new(title: String, message: String, confirm_label: String) -> Self {
        Self {
            title,
            message,
            confirm_label,
        }
    }

    pub fn open(
        self,
        on_decision: EventHandler<ConfirmDialogEvent>,
        window: &mut Window,
        cx: &mut App,
    ) {
        let gate = Rc::new(DecisionGate::default());
        let first_render = Cell::new(true);
        let cancel_focus = cx.focus_handle();
        let confirm_focus = cx.focus_handle();
        window.open_dialog(cx, move |dialog, window, cx| {
            // Builders can run again. Focus once per opening, never during a
            // subsequent rebuild, and make cancellation the safe default.
            if first_render.replace(false) {
                cancel_focus.focus(window, cx);
            }
            let ok_gate = gate.clone();
            let ok_handler = on_decision.clone();
            let ok_focus = confirm_focus.clone();
            let cancel_gate = gate.clone();
            let cancel_handler = on_decision.clone();
            let close_gate = gate.clone();
            let close_handler = on_decision.clone();
            let button_cancel_gate = gate.clone();
            let button_cancel_handler = on_decision.clone();
            let button_confirm_gate = gate.clone();
            let button_confirm_handler = on_decision.clone();
            dialog
                .title(self.title.clone())
                .w(px(440.))
                .centered()
                .entrance(DialogEntrance::Fade)
                .overlay_closable(true)
                .keyboard(true)
                .child(div().min_w_0().child(self.message.clone()))
                .footer(
                    DialogFooter::new()
                        .child(
                            Button::new("app-confirm-cancel")
                                .label(text("common.cancel"))
                                .track_focus(&cancel_focus)
                                .on_click(move |_, window, cx| {
                                    resolve(
                                        &button_cancel_gate,
                                        &button_cancel_handler,
                                        ConfirmDialogEvent::Cancel,
                                        window,
                                        cx,
                                    );
                                    window.close_dialog(cx);
                                }),
                        )
                        .child(
                            Button::new("app-confirm-accept")
                                .danger()
                                .label(self.confirm_label.clone())
                                .track_focus(&confirm_focus)
                                .on_click(move |_, window, cx| {
                                    resolve(
                                        &button_confirm_gate,
                                        &button_confirm_handler,
                                        ConfirmDialogEvent::Confirm,
                                        window,
                                        cx,
                                    );
                                    window.close_dialog(cx);
                                }),
                        ),
                )
                .on_ok(move |_, window, cx| {
                    let decision = if ok_focus.is_focused(window) {
                        ConfirmDialogEvent::Confirm
                    } else {
                        ConfirmDialogEvent::Cancel
                    };
                    resolve(&ok_gate, &ok_handler, decision, window, cx);
                    true
                })
                .on_cancel(move |_, window, cx| {
                    resolve(
                        &cancel_gate,
                        &cancel_handler,
                        ConfirmDialogEvent::Cancel,
                        window,
                        cx,
                    );
                    true
                })
                .on_close(move |_, window, cx| {
                    resolve(
                        &close_gate,
                        &close_handler,
                        ConfirmDialogEvent::Cancel,
                        window,
                        cx,
                    );
                })
        });
    }
}

fn resolve(
    gate: &DecisionGate,
    handler: &EventHandler<ConfirmDialogEvent>,
    decision: ConfirmDialogEvent,
    window: &mut Window,
    cx: &mut App,
) {
    if let Some(decision) = gate.decide(decision) {
        let handler = handler.clone();
        // close_dialog can originate in the coordinator's poll. Notify only
        // after that borrow is released, and only once across OK/cancel/close.
        window.defer(cx, move |window, cx| handler(&decision, window, cx));
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/view/ui/app_confirm_dialog.rs"]
mod tests;
