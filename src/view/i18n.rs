//! Presentation text only. Mapper names, domain values and user data are not translated.
use crate::application::dto::{LogoChoice, ThemeChoice};
use crate::domain::types::Mode;

pub fn theme_label(theme: ThemeChoice) -> String {
    text(match theme {
        ThemeChoice::Monochrome => "settings.monochrome",
        ThemeChoice::Catppuccin => "settings.catppuccin",
        ThemeChoice::RosePine => "settings.rose_pine",
    })
}

pub fn logo_label(logo: LogoChoice) -> String {
    text(match logo {
        LogoChoice::FollowTheme => "settings.follow_theme",
        LogoChoice::Monochrome => "settings.monochrome",
        LogoChoice::Catppuccin => "settings.catppuccin_logo",
        LogoChoice::RosePine => "settings.rose_pine",
    })
}

pub const DEFAULT_LOCALE: &str = "en";

pub fn init() {
    rust_i18n::set_locale(DEFAULT_LOCALE);
    gpui_kit::component::set_locale(DEFAULT_LOCALE);
}

/// Return an owned string suitable for GPUI labels and cross-thread error messages.
pub fn text(key: &str) -> String {
    rust_i18n::t!(key).into_owned()
}

pub fn key_label(key: &str) -> String {
    let readable = key.replace('_', " ");
    let mut chars = readable.chars();
    let readable = match chars.next() {
        Some(first) => first.to_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    };
    readable
}

pub fn mode_label(mode: &Mode) -> String {
    text(match mode {
        Mode::Once => "mode.once",
        Mode::Hold => "mode.hold",
        Mode::Toggle => "mode.toggle",
    })
}

pub fn macro_count(count: usize) -> String {
    let key = if count == 1 {
        "home.count.one"
    } else {
        "home.count.other"
    };
    rust_i18n::t!(key, count = count).into_owned()
}

pub fn action_count(count: usize) -> String {
    let key = if count == 1 {
        "macro.action_count.one"
    } else {
        "macro.action_count.other"
    };
    rust_i18n::t!(key, count = count).into_owned()
}

#[cfg(test)]
#[path = "../../tests/unit/view/i18n.rs"]
mod tests;
