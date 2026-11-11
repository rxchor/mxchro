use super::{
    AppInputField, EventHandler,
    pending_style::{ButtonPalette, PendingButtonExt},
};
use crate::view::{
    i18n::text,
    ui::state::collection_name::{CollectionNameDraft, CollectionNameTarget},
};
use gpui_kit::{
    base::Disableable,
    component::{
        button::{Button, ButtonVariants},
        popover::Popover,
    },
    *,
};

#[derive(Clone)]
pub enum CollectionNameEvent {
    Open(CollectionNameTarget),
    Close(CollectionNameTarget),
    Submit(CollectionNameTarget),
}

/// Controlled contextual form; its owner retains input, target and validation.
#[derive(IntoElement)]
pub struct CollectionNamePopover {
    pub target: CollectionNameTarget,
    pub draft: CollectionNameDraft,
    pub disabled: bool,
    pub stable_pending: bool,
    pub on_action: EventHandler<CollectionNameEvent>,
}

impl RenderOnce for CollectionNamePopover {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let creating = self.target == CollectionNameTarget::Create;
        let (popover_id, trigger_id, label) = if creating {
            (
                "create-collection-popover".to_owned(),
                "add-collection",
                text("sidebar.create_collection"),
            )
        } else {
            let CollectionNameTarget::Rename(id) = &self.target else {
                unreachable!()
            };
            (
                format!("rename-collection-popover-{id}"),
                "rename-collection",
                text("collection.rename"),
            )
        };
        let open = self.draft.target.as_ref() == Some(&self.target);
        let on_open = self.on_action.clone();
        let target = self.target.clone();
        let submit = self.on_action.clone();
        let submit_target = self.target.clone();
        let cancel = self.on_action;
        let disabled = self.disabled;
        Popover::new(SharedString::from(popover_id))
            .anchor(if creating {
                Anchor::BottomLeft
            } else {
                Anchor::TopLeft
            })
            .w(px(300.))
            .open(open)
            .track_focus(&self.draft.input.focus_handle(cx))
            .overlay_closable(!disabled)
            .on_open_change(move |open, window, cx| {
                if !disabled || !*open {
                    on_open(
                        &if *open {
                            CollectionNameEvent::Open(target.clone())
                        } else {
                            CollectionNameEvent::Close(target.clone())
                        },
                        window,
                        cx,
                    );
                }
            })
            .trigger(
                Button::new(trigger_id)
                    .label(label.clone())
                    .disabled(disabled)
                    .stable_pending(self.stable_pending, ButtonPalette::Default, cx),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(div().text_lg().child(label))
                    .child(
                        AppInputField::new(text("collection.name"), &self.draft.input)
                            .error(self.draft.error)
                            .disabled(disabled),
                    )
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap_2()
                            .child(
                                Button::new("cancel-collection-name")
                                    .label(text("common.cancel"))
                                    .disabled(disabled)
                                    .on_click(move |_, window, cx| {
                                        cancel(
                                            &CollectionNameEvent::Close(self.target.clone()),
                                            window,
                                            cx,
                                        )
                                    }),
                            )
                            .child(
                                Button::new("submit-collection-name")
                                    .primary()
                                    .label(if creating {
                                        text("collection.create")
                                    } else {
                                        text("collection.save_name")
                                    })
                                    .disabled(disabled)
                                    .on_click(move |_, window, cx| {
                                        submit(
                                            &CollectionNameEvent::Submit(submit_target.clone()),
                                            window,
                                            cx,
                                        )
                                    }),
                            ),
                    ),
            )
    }
}
