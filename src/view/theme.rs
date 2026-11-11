//! Central Mxchro palette. Hexadecimal RGB colors (0xRRGGBB).
//! Three explicit dark palettes; switching synchronizes GPUI-KIT and Base tokens.

use crate::application::dto::ThemeChoice;

use gpui_kit::{
    App, Hsla,
    component::{Theme, ThemeColor, ThemeMode},
    rgb,
};

pub const BACKGROUND: u32 = 0x090909;
pub const SURFACE: u32 = 0x121212;
pub const ELEVATED: u32 = 0x1c1c1c;
pub const HOVER: u32 = 0x262626;
pub const PRESSED: u32 = 0x333333;
pub const BORDER: u32 = 0x303030;
pub const TEXT: u32 = 0xf5f5f5;
pub const TEXT_MUTED: u32 = 0xa3a3a3;
pub const PRIMARY: u32 = 0xe5e5e5;
pub const PRIMARY_HOVER: u32 = 0xffffff;
pub const PRIMARY_PRESSED: u32 = 0xc4c4c4;
pub const DANGER: u32 = 0xf87171;
pub const DANGER_HOVER: u32 = 0xfca5a5;
pub const DANGER_PRESSED: u32 = 0xef4444;

pub const ORIGINAL_LOGO_BACKGROUND: u32 = 0x1c1c1c;
pub const ORIGINAL_LOGO_M: u32 = 0xf5f5f5;
pub const ORIGINAL_LOGO_X: u32 = 0xa3a3a3;

#[derive(Clone, Copy)]
pub struct Palette {
    pub background: u32,
    pub surface: u32,
    pub elevated: u32,
    pub hover: u32,
    pub pressed: u32,
    pub border: u32,
    pub text: u32,
    pub muted: u32,
    pub primary: u32,
    pub primary_hover: u32,
    pub primary_pressed: u32,
    pub danger: u32,
    pub danger_hover: u32,
    pub danger_pressed: u32,
    pub success: u32,
    pub warning: u32,
    pub info: u32,
    pub logo_m: u32,
    pub logo_x: u32,
}

pub const MONOCHROME: Palette = Palette {
    background: BACKGROUND,
    surface: SURFACE,
    elevated: ELEVATED,
    hover: HOVER,
    pressed: PRESSED,
    border: BORDER,
    text: TEXT,
    muted: TEXT_MUTED,
    primary: PRIMARY,
    primary_hover: PRIMARY_HOVER,
    primary_pressed: PRIMARY_PRESSED,
    danger: DANGER,
    danger_hover: DANGER_HOVER,
    danger_pressed: DANGER_PRESSED,
    success: 0x86efac,
    warning: 0xfde047,
    info: 0x93c5fd,
    logo_m: 0xf5f5f5,
    logo_x: 0xa3a3a3,
};

/// Catppuccin Mocha: base/mantle/surface and mauve accent.
pub const CATPPUCCIN: Palette = Palette {
    background: 0x1e1e2e,
    surface: 0x181825,
    elevated: 0x313244,
    hover: 0x45475a,
    pressed: 0x585b70,
    border: 0x45475a,
    text: 0xcdd6f4,
    muted: 0xa6adc8,
    primary: 0xcba6f7,
    primary_hover: 0xf5c2e7,
    primary_pressed: 0xb4befe,
    danger: 0xf38ba8,
    danger_hover: 0xf5c2e7,
    danger_pressed: 0xeba0ac,
    success: 0xa6e3a1,
    warning: 0xf9e2af,
    info: 0x89b4fa,
    logo_m: 0xcba6f7,
    logo_x: 0x89dceb,
};

/// Rosé Pine (main): base/surface/overlay and rose accent.
pub const ROSE_PINE: Palette = Palette {
    background: 0x191724,
    surface: 0x1f1d2e,
    elevated: 0x26233a,
    hover: 0x403d52,
    pressed: 0x524f67,
    border: 0x403d52,
    text: 0xe0def4,
    muted: 0x908caa,
    primary: 0xebbcba,
    primary_hover: 0xf6c177,
    primary_pressed: 0xc4a7e7,
    danger: 0xeb6f92,
    danger_hover: 0xebbcba,
    danger_pressed: 0xeb6f92,
    success: 0x9ccfd8,
    warning: 0xf6c177,
    info: 0xc4a7e7,
    logo_m: 0xebbcba,
    logo_x: 0xc4a7e7,
};

pub fn palette(choice: ThemeChoice) -> Palette {
    match choice {
        ThemeChoice::Monochrome => MONOCHROME,
        ThemeChoice::Catppuccin => CATPPUCCIN,
        ThemeChoice::RosePine => ROSE_PINE,
    }
}

fn color(value: u32) -> Hsla {
    rgb(value).into()
}

/// Call after `gpui_kit::init`, before opening windows.
/// Theme::update synchronizes colors, tokens and GPUI-KIT's base layer.
pub fn init(cx: &mut App) {
    Theme::change(ThemeMode::Dark, None, cx);
    apply(ThemeChoice::default(), cx);
}

pub fn apply(choice: ThemeChoice, cx: &mut App) {
    Theme::update(cx, |theme| {
        apply_palette(&mut theme.colors, palette(choice))
    });
}

fn apply_palette(colors: &mut ThemeColor, p: Palette) {
    colors.background = color(p.background);
    colors.foreground = color(p.text);
    colors.border = color(p.border);
    colors.input = color(p.border);
    colors.muted = color(p.elevated);
    colors.muted_foreground = color(p.muted);
    colors.accent = color(p.hover);
    colors.accent_foreground = color(p.text);
    colors.caret = color(p.text);
    colors.ring = color(p.primary);
    colors.selection = color(p.pressed);

    colors.primary = color(p.primary);
    colors.primary_foreground = color(p.background);
    colors.primary_hover = color(p.primary_hover);
    colors.primary_active = color(p.primary_pressed);
    colors.secondary = color(p.elevated);
    colors.secondary_foreground = color(p.text);
    colors.secondary_hover = color(p.hover);
    colors.secondary_active = color(p.pressed);

    colors.button = color(p.elevated);
    colors.button_foreground = color(p.text);
    colors.button_hover = color(p.hover);
    colors.button_active = color(p.pressed);
    colors.button_primary = colors.primary;
    colors.button_primary_foreground = colors.primary_foreground;
    colors.button_primary_hover = colors.primary_hover;
    colors.button_primary_active = colors.primary_active;
    colors.button_secondary = colors.secondary;
    colors.button_secondary_foreground = colors.secondary_foreground;
    colors.button_secondary_hover = colors.secondary_hover;
    colors.button_secondary_active = colors.secondary_active;

    colors.danger = color(p.danger);
    colors.danger_foreground = color(p.background);
    colors.danger_hover = color(p.danger_hover);
    colors.danger_active = color(p.danger_pressed);
    colors.button_danger = colors.danger;
    colors.button_danger_foreground = colors.danger_foreground;
    colors.button_danger_hover = colors.danger_hover;
    colors.button_danger_active = colors.danger_active;

    colors.sidebar = color(p.surface);
    colors.sidebar_foreground = color(p.text);
    colors.sidebar_border = color(p.border);
    colors.sidebar_accent = color(p.hover);
    colors.sidebar_accent_foreground = color(p.text);
    colors.sidebar_primary = colors.primary;
    colors.sidebar_primary_foreground = colors.primary_foreground;
    colors.popover = color(p.elevated);
    colors.popover_foreground = color(p.text);
    colors.title_bar = color(p.surface);
    colors.title_bar_border = color(p.border);
    colors.list = color(p.surface);
    colors.list_head = color(p.elevated);
    colors.list_even = color(p.elevated);
    colors.list_hover = color(p.hover);
    colors.list_active = color(p.pressed);
    colors.list_active_border = color(p.primary);
    colors.list_highlight_foreground = color(p.primary);
    colors.scrollbar = color(p.background);
    colors.scrollbar_thumb = color(p.border);
    colors.scrollbar_thumb_hover = color(p.muted);
    colors.link = color(p.primary);
    colors.link_hover = color(p.primary_hover);
    colors.link_active = color(p.primary_pressed);
    colors.group_box = color(p.surface);
    colors.group_box_foreground = color(p.text);
    colors.accordion = color(p.surface);
    colors.drag_border = color(p.primary);
    colors.drop_target = color(p.hover);

    colors.success = color(p.success);
    colors.success_foreground = color(p.background);
    colors.success_hover = color(p.success);
    colors.success_active = color(p.success);
    colors.warning = color(p.warning);
    colors.warning_foreground = color(p.background);
    colors.warning_hover = color(p.warning);
    colors.warning_active = color(p.warning);
    colors.info = color(p.info);
    colors.info_foreground = color(p.background);
    colors.info_hover = color(p.info);
    colors.info_active = color(p.info);
    colors.button_success = colors.success;
    colors.button_success_foreground = colors.success_foreground;
    colors.button_success_hover = colors.success_hover;
    colors.button_success_active = colors.success_active;
    colors.button_warning = colors.warning;
    colors.button_warning_foreground = colors.warning_foreground;
    colors.button_warning_hover = colors.warning_hover;
    colors.button_warning_active = colors.warning_active;
    colors.button_info = colors.info;
    colors.button_info_foreground = colors.info_foreground;
    colors.button_info_hover = colors.info_hover;
    colors.button_info_active = colors.info_active;
}

#[cfg(test)]
#[path = "../../tests/unit/view/theme.rs"]
mod tests;
