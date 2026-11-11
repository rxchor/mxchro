use super::layout::{ACTION_PANEL_HEIGHT, panel};
use crate::view::i18n::text;
use crate::view::ui::components::{
    AppInputField, AppSelect, EventHandler, app_select::AppSelectState,
};
use crate::view::ui::state::field_errors::{EditorField, FieldErrors};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    base::{Disableable, StyledExt},
    component::{button::Button, input::InputState},
    *,
};

#[derive(Clone)]
pub enum ActionFormEvent {
    AddKey(Option<bool>),
    AddWait,
    CaptureKey,
}

#[derive(IntoElement)]
pub struct ActionForm {
    pub errors: FieldErrors,
    pub key_input: Entity<AppSelectState>,
    pub wait_input: Entity<InputState>,
    pub disabled: bool,
    pub capture_enabled: bool,
    pub on_action: EventHandler<ActionFormEvent>,
}

impl RenderOnce for ActionForm {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut buttons = div().w_full().flex().gap_2();
        let capture = self.on_action.clone();
        for (id, label, description, press) in [
            ("pair", text("editor.key_both"), text("editor.pair"), None),
            (
                "press",
                text("editor.key_press"),
                text("editor.press_only"),
                Some(true),
            ),
            (
                "release",
                text("editor.key_release"),
                text("editor.release_only"),
                Some(false),
            ),
        ] {
            let on_action = self.on_action.clone();
            buttons = buttons.child(
                Button::new(id)
                    .flex_1()
                    .min_w_0()
                    .label(label)
                    .accessibility_label(description.clone())
                    .tooltip(description)
                    .disabled(self.disabled)
                    .on_click(move |_, window, cx| {
                        on_action(&ActionFormEvent::AddKey(press), window, cx)
                    }),
            );
        }
        let key_form = panel(cx)
            .id("key-action-form")
            .overflow_y_scroll()
            .flex_grow(3.)
            .flex_basis(px(0.))
            .min_h_0()
            .justify_between()
            .p_4()
            .gap_3()
            .child(
                div()
                    .text_sm()
                    .font_semibold()
                    .child(text("editor.key_action")),
            )
            .child(
                AppSelect::new(text("editor.action_key"), &self.key_input)
                    .error(self.errors.get(EditorField::ActionKey))
                    .when(self.capture_enabled, |select| {
                        select.on_capture(std::rc::Rc::new(move |_, window, cx| {
                            capture(&ActionFormEvent::CaptureKey, window, cx)
                        }))
                    })
                    .disabled(self.disabled),
            )
            .child(buttons);
        let wait_form = panel(cx)
            .id("wait-action-form")
            .overflow_y_scroll()
            .flex_grow(2.)
            .flex_basis(px(0.))
            .min_h_0()
            .justify_between()
            .p_4()
            .gap_3()
            .child(
                div()
                    .text_sm()
                    .font_semibold()
                    .child(text("editor.wait_action")),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_end()
                    .gap_3()
                    .child(
                        div()
                            .flex_1()
                            .flex_basis(px(120.))
                            .min_w(px(120.))
                            .max_w(px(180.))
                            .child(
                                AppInputField::new(text("editor.wait"), &self.wait_input)
                                    .error(self.errors.get(EditorField::Wait))
                                    .disabled(self.disabled),
                            ),
                    )
                    .child(
                        Button::new("wait")
                            .label(text("editor.add_wait"))
                            .disabled(self.disabled)
                            .on_click(move |_, window, cx| {
                                (self.on_action)(&ActionFormEvent::AddWait, window, cx)
                            }),
                    ),
            );
        div()
            .w_full()
            .h(px(ACTION_PANEL_HEIGHT))
            .flex_shrink_0()
            .min_w_0()
            .flex()
            .flex_col()
            .gap_4()
            .child(key_form)
            .child(wait_form)
    }
}
