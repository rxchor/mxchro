use gpui_kit::{component::ActiveTheme, *};

/// Equal-width fields wrap naturally when their container becomes too narrow.
pub(super) fn field_column() -> Div {
    div().flex_1().flex_basis(px(240.)).min_w_0()
}

pub(super) fn panel(cx: &App) -> Div {
    div()
        .w_full()
        .min_w_0()
        .p_5()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().sidebar)
        .flex()
        .flex_col()
        .gap_4()
}

/// Shared height of the sequence and the combined key/wait cards (including their gap).
pub(super) const ACTION_PANEL_HEIGHT: f32 = 360.;

pub(super) const ACTION_ROW_HEIGHT: f32 = 50.;
pub(super) const ACTION_ROW_SPACING: f32 = 8.;
pub(super) const ACTION_ROW_STRIDE: f32 = ACTION_ROW_HEIGHT + ACTION_ROW_SPACING;
