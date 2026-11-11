use super::EventHandler;
use super::app_select::{AppSelect, AppSelectState};
use super::collection_name_popover::{CollectionNameEvent, CollectionNamePopover};
use super::pending_style::{ButtonPalette, PendingButtonExt};
use crate::application::dto::CollectionDTO;
use crate::view::i18n::text;
use crate::view::ui::state::collection_name::{CollectionNameDraft, CollectionNameTarget};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{base::Disableable, component::button::Button, *};

#[derive(Clone)]
pub enum CollectionEvent {
    Toggle(String, bool),
    NameForm(CollectionNameEvent),
    Delete(String),
    CaptureHotkey(String),
}

#[derive(IntoElement)]
pub struct CollectionToolbar {
    pub name_form: CollectionNameDraft,
    pub hotkey: Option<Entity<AppSelectState>>,
    pub collection: CollectionDTO,
    pub disabled: bool,
    pub stable_pending: bool,
    pub capture_enabled: bool,
    pub on_action: EventHandler<CollectionEvent>,
}

impl RenderOnce for CollectionToolbar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let toggle_id = self.collection.id.clone();
        let toggle = self.on_action.clone();
        let rename = self.on_action.clone();
        let capture = self.on_action.clone();
        let capture_id = self.collection.id.clone();
        let buttons = div()
            .flex()
            .gap_2()
            .flex_wrap()
            .child(
                Button::new("toggle-collection")
                    .label(if self.collection.enabled {
                        text("collection.disable")
                    } else {
                        text("collection.enable")
                    })
                    .disabled(self.disabled)
                    .stable_pending(self.stable_pending, ButtonPalette::Default, cx)
                    .on_click(move |_, window, cx| {
                        toggle(
                            &CollectionEvent::Toggle(toggle_id.clone(), !self.collection.enabled),
                            window,
                            cx,
                        )
                    }),
            )
            .child(CollectionNamePopover {
                target: CollectionNameTarget::Rename(self.collection.id.clone()),
                draft: self.name_form,
                disabled: self.disabled,
                stable_pending: self.stable_pending,
                on_action: std::rc::Rc::new(move |event, window, cx| {
                    rename(&CollectionEvent::NameForm(event.clone()), window, cx)
                }),
            })
            .child(
                Button::new("delete-collection")
                    .label(text("collection.delete"))
                    .disabled(self.disabled)
                    .stable_pending(self.stable_pending, ButtonPalette::Default, cx)
                    .on_click(move |_, window, cx| {
                        (self.on_action)(
                            &CollectionEvent::Delete(self.collection.id.clone()),
                            window,
                            cx,
                        )
                    }),
            );
        let mut content = div().flex().flex_col().gap_4().child(buttons);
        if let Some(hotkey) = &self.hotkey {
            content = content
                .child(
                    div().w_full().max_w(px(360.)).child(
                        AppSelect::new(text("collection.hotkey"), hotkey)
                            .when(self.capture_enabled, |select| {
                                select.on_capture(std::rc::Rc::new(move |_, window, cx| {
                                    capture(
                                        &CollectionEvent::CaptureHotkey(capture_id.clone()),
                                        window,
                                        cx,
                                    )
                                }))
                            })
                            .optional(true)
                            .disabled(self.disabled)
                            .stable_pending(self.stable_pending),
                    ),
                )
                .child(text("collection.hotkey_help"));
        }
        content
    }
}
