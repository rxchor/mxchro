use crate::view::i18n::key_label;
use gpui_kit::{ParentElement, component::tag::Tag};

fn label(key: &str, inactive: bool) -> String {
    let key = key_label(key);
    if inactive {
        rust_i18n::t!("hotkey_summary.inactive", key = key).into_owned()
    } else {
        rust_i18n::t!("hotkey_summary.assigned", key = key).into_owned()
    }
}

/// Read-only assignment summary, not a key binding or an interactive control.
pub fn toggle_hotkey_tag(key: Option<&str>, inactive: bool) -> Option<Tag> {
    key.map(|key| {
        (if inactive {
            Tag::warning()
        } else {
            Tag::secondary()
        })
        .outline()
        .child(label(key, inactive))
    })
}

#[cfg(test)]
#[path = "../../../../tests/unit/view/ui/toggle_hotkey.rs"]
mod tests;
