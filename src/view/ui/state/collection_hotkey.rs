use crate::{
    application::dto::CollectionDTO,
    view::ui::components::{
        app_select::{AppSelectState, SelectOption, select_state},
        key_select::hotkey_options,
    },
};
use gpui_kit::*;

/// Retained for the selected collection; searching is not a committed change.
pub struct CollectionHotkeyDraft {
    pub id: String,
    pub select: Entity<AppSelectState>,
    items: Vec<SelectOption>,
}

impl CollectionHotkeyDraft {
    pub fn new(
        collection: &CollectionDTO,
        keys: &[String],
        window: &mut Window,
        cx: &mut App,
    ) -> Self {
        let items = hotkey_options(keys, collection.toggle_enabled_hotkey.as_deref());
        Self {
            id: collection.id.clone(),
            select: select_state(
                items.clone(),
                collection.toggle_enabled_hotkey.as_deref(),
                true,
                window,
                cx,
            ),
            items,
        }
    }

    pub fn sync(
        &mut self,
        collection: &CollectionDTO,
        keys: &[String],
        window: &mut Window,
        cx: &mut App,
    ) {
        let items = hotkey_options(keys, collection.toggle_enabled_hotkey.as_deref());
        if self.items != items {
            self.select.update(cx, |state, cx| {
                state.set_items(items.clone().into(), window, cx)
            });
            self.items = items;
        }
        if self.select.read(cx).selected_value() != collection.toggle_enabled_hotkey.as_ref() {
            self.select.update(cx, |state, cx| {
                if let Some(key) = &collection.toggle_enabled_hotkey {
                    state.set_selected_value(key, window, cx);
                } else {
                    state.set_selected_index(None, window, cx);
                }
                cx.notify();
            });
        }
    }
}
