use crate::{
    application::dto::{AppSettingsDTO, LogoChoice, ThemeChoice},
    view::{
        assets::logo_path,
        i18n::{logo_label, text, theme_label},
        theme,
        ui::components::{EventHandler, field_error::field_error},
    },
};
use gpui_kit::{
    base::{Disableable, Selectable},
    component::{ActiveTheme, button::Button},
    *,
};

#[derive(Clone, Copy)]
pub enum SettingsEvent {
    Theme(ThemeChoice),
    Logo(LogoChoice),
    Retry,
    Back,
}

#[derive(IntoElement)]
pub struct SettingsPage {
    pub settings: AppSettingsDTO,
    pub pending: bool,
    pub error: Option<String>,
    pub on_action: EventHandler<SettingsEvent>,
}

impl RenderOnce for SettingsPage {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let back = self.on_action.clone();
        let mut themes = div().id("theme-options").flex().flex_wrap().gap_3();
        for choice in ThemeChoice::ALL {
            let palette = theme::palette(choice);
            let on_action = self.on_action.clone();
            themes = themes.child(
                div()
                    .flex_1()
                    .min_w(px(190.))
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_4()
                    .rounded_lg()
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().sidebar)
                    .child(
                        div()
                            .h(px(64.))
                            .rounded_md()
                            .bg(rgb(palette.background))
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap_2()
                            .children(
                                [
                                    palette.surface,
                                    palette.text,
                                    palette.primary,
                                    palette.danger,
                                ]
                                .into_iter()
                                .map(|color| div().size(px(20.)).rounded_full().bg(rgb(color))),
                            ),
                    )
                    .child(
                        Button::new(choice.key())
                            .label(theme_label(choice))
                            .w_full()
                            .selected(self.settings.theme == choice)
                            .disabled(self.pending)
                            .on_click(move |_, window, cx| {
                                on_action(&SettingsEvent::Theme(choice), window, cx)
                            }),
                    ),
            );
        }
        let mut logos = div().id("logo-options").flex().flex_wrap().gap_3();
        for choice in LogoChoice::ALL {
            let on_action = self.on_action.clone();
            logos = logos.child(
                div()
                    .w(px(140.))
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_3()
                    .p_3()
                    .rounded_lg()
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().sidebar)
                    .child(img(logo_path(choice, self.settings.theme)).size(px(64.)))
                    .child(
                        Button::new(choice.key())
                            .label(logo_label(choice))
                            .w_full()
                            .selected(self.settings.logo == choice)
                            .disabled(self.pending)
                            .on_click(move |_, window, cx| {
                                on_action(&SettingsEvent::Logo(choice), window, cx)
                            }),
                    ),
            );
        }
        let mut content = div()
            .w_full()
            .max_w(px(1152.))
            .mx_auto()
            .flex()
            .flex_col()
            .gap_6()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(div().text_2xl().child(text("settings.title")))
                    .child(
                        Button::new("back")
                            .label(text("settings.back"))
                            .on_click(move |_, window, cx| back(&SettingsEvent::Back, window, cx)),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(text("settings.description")),
            )
            .child(div().text_lg().child(text("settings.theme")))
            .child(themes)
            .child(div().text_lg().child(text("settings.logo")))
            .child(logos)
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(text("settings.logo_help")),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(text(if self.pending {
                        "settings.saving"
                    } else if self.error.is_some() {
                        "settings.unsaved"
                    } else {
                        "settings.autosave"
                    })),
            );
        if let Some(error) = self.error {
            content = content.child(field_error(error, cx)).child(
                Button::new("retry-settings")
                    .label(text("settings.retry"))
                    .disabled(self.pending)
                    .on_click(move |_, window, cx| {
                        (self.on_action)(&SettingsEvent::Retry, window, cx)
                    }),
            );
        }
        content
    }
}
