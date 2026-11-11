use crate::view::i18n::key_label;

use super::app_select::SelectOption;

/// Show readable labels without changing the canonical mapper values.
pub fn key_options(keys: &[String]) -> Vec<SelectOption> {
    keys.iter()
        .map(|key| SelectOption {
            value: key.clone(),
            label: key_label(key).into(),
        })
        .collect()
}

/// Preserve an unknown legacy assignment so it remains visible and clearable.
pub fn hotkey_options(keys: &[String], current: Option<&str>) -> Vec<SelectOption> {
    let mut items = key_options(keys);
    if let Some(key) = current
        && !items.iter().any(|item| item.value == key)
    {
        items.push(SelectOption {
            value: key.into(),
            label: key.to_owned().into(),
        });
    }
    items
}

#[cfg(test)]
#[path = "../../../../tests/unit/view/ui/key_select.rs"]
mod tests;
