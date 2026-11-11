//! Keep short toggle requests from dimming every control on the home page.
//! These overrides change presentation only: the controls remain truly disabled.
use gpui_kit::{
    component::{ActiveTheme, button::Button},
    *,
};

#[derive(Clone, Copy)]
pub enum ButtonPalette {
    Default,
    Selected,
    Primary,
}

pub trait PendingButtonExt {
    fn stable_pending(self, stable: bool, palette: ButtonPalette, cx: &App) -> Self;
}

impl PendingButtonExt for Button {
    fn stable_pending(self, stable: bool, palette: ButtonPalette, cx: &App) -> Self {
        if !stable {
            return self;
        }
        let theme = cx.theme();
        match palette {
            ButtonPalette::Default => self
                .bg(theme.tokens.button)
                .text_color(theme.button_foreground)
                .border_color(theme.input),
            ButtonPalette::Selected => self
                .bg(theme.tokens.button_active)
                .text_color(theme.button_foreground)
                .border_color(theme.input),
            ButtonPalette::Primary => self
                .bg(theme.tokens.button_primary)
                .text_color(theme.button_primary_foreground)
                .border_color(theme.primary),
        }
    }
}
