use gpui_kit::{component::ActiveTheme, *};

/// Inline validation, adjacent to the relevant control rather than a toast.
pub fn field_error(message: String, cx: &App) -> impl IntoElement {
    div()
        .id("field-error")
        .role(Role::Alert)
        .text_sm()
        .text_color(cx.theme().danger)
        .child(message)
}
