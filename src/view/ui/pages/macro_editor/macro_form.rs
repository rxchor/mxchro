use super::layout::{field_column, panel};
use crate::view::i18n::text;
use crate::view::ui::state::field_errors::{EditorField, FieldErrors};
use crate::{
    application::dto::MacroEditDTO,
    view::ui::components::{AppCheckBox, AppSelect, EventHandler, app_select::AppSelectState},
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    base::Disableable,
    component::{ActiveTheme, button::Button},
    *,
};

#[derive(Clone, Copy)]
pub enum MacroOption {
    Enabled,
    BlockOriginal,
    IgnoreInjected,
}

impl MacroOption {
    pub fn get(self, config: &MacroEditDTO) -> bool {
        match self {
            Self::Enabled => config.enabled,
            Self::BlockOriginal => config.block_original_event,
            Self::IgnoreInjected => config.ignore_injected_keys,
        }
    }

    pub fn set(self, config: &mut MacroEditDTO, value: bool) {
        match self {
            Self::Enabled => config.enabled = value,
            Self::BlockOriginal => config.block_original_event = value,
            Self::IgnoreInjected => config.ignore_injected_keys = value,
        }
    }
}

#[derive(Clone)]
pub enum MacroFormEvent {
    SetOption(MacroOption, bool),
    ClearHotkey,
    Capture(EditorField),
}

#[cfg(test)]
#[path = "../../../../../tests/unit/view/ui/macro_form.rs"]
mod tests;

#[derive(IntoElement)]
pub struct MacroForm {
    pub errors: FieldErrors,
    pub config: MacroEditDTO,
    pub trigger: Entity<AppSelectState>,
    pub hotkey: Entity<AppSelectState>,
    pub mode: Entity<AppSelectState>,
    pub collection: Entity<AppSelectState>,
    pub disabled: bool,
    pub capture_enabled: bool,
    pub on_action: EventHandler<MacroFormEvent>,
}

impl RenderOnce for MacroForm {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let grouped = self.collection.read(cx).selected_value().is_some();
        let has_hotkey = self.hotkey.read(cx).selected_value().is_some();
        let clear_hotkey = self.on_action.clone();
        let trigger_capture = self.on_action.clone();
        let hotkey_capture = self.on_action.clone();
        let mut form = panel(cx)
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_6()
                    .child(
                        field_column().child(
                            AppSelect::new(text("editor.trigger"), &self.trigger)
                                .error(self.errors.get(EditorField::Trigger))
                                .when(self.capture_enabled, |select| {
                                    select.on_capture(std::rc::Rc::new(move |_, window, cx| {
                                        trigger_capture(
                                            &MacroFormEvent::Capture(EditorField::Trigger),
                                            window,
                                            cx,
                                        )
                                    }))
                                })
                                .disabled(self.disabled),
                        ),
                    )
                    .child(
                        field_column().child(
                            AppSelect::new(text("editor.mode"), &self.mode)
                                .error(self.errors.get(EditorField::Mode))
                                .disabled(self.disabled),
                        ),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(text("editor.mode_help")),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_6()
                    .child(
                        field_column().child(
                            AppSelect::new(text("editor.collection"), &self.collection)
                                .error(self.errors.get(EditorField::Collection))
                                .optional(true)
                                .placeholder(text("common.no_collection"))
                                .disabled(self.disabled),
                        ),
                    )
                    .child(
                        field_column()
                            .child(
                                AppSelect::new(text("editor.hotkey"), &self.hotkey)
                                    .error(self.errors.get(EditorField::Hotkey))
                                    .when(self.capture_enabled, |select| {
                                        select.on_capture(std::rc::Rc::new(move |_, window, cx| {
                                            hotkey_capture(
                                                &MacroFormEvent::Capture(EditorField::Hotkey),
                                                window,
                                                cx,
                                            )
                                        }))
                                    })
                                    .optional(true)
                                    .disabled(self.disabled || grouped),
                            )
                            .when(grouped, |field| {
                                field.child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(text("editor.collection_hotkey_help")),
                                )
                            })
                            .when(grouped && has_hotkey, |field| {
                                field.child(
                                    Button::new("clear-macro-hotkey")
                                        .label(text("editor.clear_hotkey"))
                                        .disabled(self.disabled)
                                        .on_click(move |_, window, cx| {
                                            clear_hotkey(&MacroFormEvent::ClearHotkey, window, cx)
                                        }),
                                )
                            }),
                    ),
            )
            .child(div().border_t_1().border_color(cx.theme().border));
        let mut options = div().flex().flex_wrap().gap_4();
        for (id, label, option) in [
            (
                "block",
                text("editor.block_original"),
                MacroOption::BlockOriginal,
            ),
            (
                "injected",
                text("editor.ignore_injected"),
                MacroOption::IgnoreInjected,
            ),
        ] {
            let on_action = self.on_action.clone();
            options = options.child(
                field_column().child(
                    AppCheckBox::new(id, label)
                        .checked(option.get(&self.config))
                        .disabled(self.disabled)
                        .on_change(move |checked, window, cx| {
                            on_action(&MacroFormEvent::SetOption(option, *checked), window, cx)
                        }),
                ),
            );
        }
        let on_action = self.on_action.clone();
        form = form.child(options).child(
            AppCheckBox::new("enabled", text("common.enabled"))
                .checked(self.config.enabled)
                .disabled(self.disabled)
                .on_change(move |checked, window, cx| {
                    on_action(
                        &MacroFormEvent::SetOption(MacroOption::Enabled, *checked),
                        window,
                        cx,
                    )
                }),
        );
        form
    }
}
