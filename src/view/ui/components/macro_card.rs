use super::EventHandler;
use super::pending_style::{ButtonPalette, PendingButtonExt};
use super::toggle_hotkey::toggle_hotkey_tag;
use crate::application::dto::MacroDTO;
use crate::view::i18n::{self, action_count, mode_label, text};
use gpui_kit::{
    base::Disableable,
    component::{ActiveTheme, Sizable, button::Button, tag::Tag},
    *,
};

#[derive(Clone)]
pub enum MacroCardEvent {
    Edit(String),
    Toggle(String, bool),
    Delete(String),
}

#[derive(IntoElement)]
pub struct MacroCard {
    pub config: MacroDTO,
    pub disabled: bool,
    pub stable_pending: bool,
    pub hotkey_inactive: bool,
    pub on_action: EventHandler<MacroCardEvent>,
}

impl RenderOnce for MacroCard {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let edit_key = self.config.trigger_key.clone();
        let toggle_key = self.config.trigger_key.clone();
        let delete_key = self.config.trigger_key.clone();
        let enabled = self.config.enabled;
        let edit = self.on_action.clone();
        let toggle = self.on_action.clone();
        div()
            .flex()
            .items_center()
            .justify_between()
            .gap_3()
            .p_4()
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_lg()
                            .child(i18n::key_label(&self.config.trigger_key)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap_2()
                            .min_h(px(24.))
                            .child(Tag::secondary().child(mode_label(&self.config.mode)))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(action_count(self.config.actions_count)),
                            )
                            .children(toggle_hotkey_tag(
                                self.config.toggle_enabled_hotkey.as_deref(),
                                self.hotkey_inactive || self.config.collection_id.is_some(),
                            )),
                    )
                    .child(
                        div().flex().items_center().h(px(26.)).child(
                            (if enabled {
                                Tag::success()
                            } else {
                                Tag::danger()
                            })
                            .outline()
                            .child(if enabled {
                                text("common.enabled")
                            } else {
                                text("common.disabled")
                            }),
                        ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_shrink_0()
                    .gap_2()
                    .child(
                        Button::new(SharedString::from(format!("edit-{toggle_key}")))
                            .small()
                            .label(text("common.edit"))
                            .disabled(self.disabled)
                            .stable_pending(self.stable_pending, ButtonPalette::Default, cx)
                            .on_click(move |_, window, cx| {
                                edit(&MacroCardEvent::Edit(edit_key.clone()), window, cx)
                            }),
                    )
                    .child(
                        Button::new(SharedString::from(format!("toggle-{toggle_key}")))
                            .small()
                            .label(if enabled {
                                text("common.disable")
                            } else {
                                text("common.enable")
                            })
                            .disabled(self.disabled)
                            .stable_pending(self.stable_pending, ButtonPalette::Default, cx)
                            .on_click(move |_, window, cx| {
                                toggle(
                                    &MacroCardEvent::Toggle(toggle_key.clone(), !enabled),
                                    window,
                                    cx,
                                )
                            }),
                    )
                    .child(
                        Button::new(SharedString::from(format!("delete-{delete_key}")))
                            .small()
                            .label(text("common.delete"))
                            .disabled(self.disabled)
                            .stable_pending(self.stable_pending, ButtonPalette::Default, cx)
                            .on_click(move |_, window, cx| {
                                (self.on_action)(
                                    &MacroCardEvent::Delete(delete_key.clone()),
                                    window,
                                    cx,
                                )
                            }),
                    ),
            )
    }
}
