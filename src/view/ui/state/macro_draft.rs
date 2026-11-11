use super::field_errors::FieldErrors;
use crate::application::dto::{CollectionDTO, MacroEditDTO, SaveMacroDTO};
use crate::domain::types::{Action, Mode};
use crate::view::i18n::{mode_label, text};
use crate::view::ui::components::app_input_field::text_input;
use crate::view::ui::components::{
    app_select::{AppSelectState, SelectOption, select_state},
    key_select::{hotkey_options, key_options},
};
use crate::view::ui::pages::macro_editor::actions_list::ActionsList;
use gpui_kit::{component::input::InputState, *};

#[derive(Clone)]
pub struct MacroDraft {
    pub errors: FieldErrors,
    pub old_key: Option<String>,
    pub config: MacroEditDTO,
    pub trigger: Entity<AppSelectState>,
    pub hotkey: Entity<AppSelectState>,
    pub action_key: Entity<AppSelectState>,
    pub mode: Entity<AppSelectState>,
    pub collection: Entity<AppSelectState>,
    collection_items: Vec<SelectOption>,
    pub wait: Entity<InputState>,
    pub actions_scroll: ScrollHandle,
    pub actions_list: Entity<ActionsList>,
}

impl MacroDraft {
    pub fn new(
        config: Option<MacroEditDTO>,
        collection_id: Option<String>,
        keys: &[String],
        collections: &[CollectionDTO],
        window: &mut Window,
        cx: &mut App,
    ) -> Self {
        let old_key = config.as_ref().map(|m| m.trigger_key.clone());
        let config = config.unwrap_or(MacroEditDTO {
            trigger_key: String::new(),
            mode: Mode::Once,
            actions: vec![],
            block_original_event: true,
            ignore_injected_keys: true,
            enabled: true,
            collection_id,
            toggle_enabled_hotkey: None,
        });
        let collection_items = collection_options(collections);
        let actions_scroll = ScrollHandle::new();
        let actions_list =
            cx.new(|cx| ActionsList::new(config.actions.clone(), actions_scroll.clone(), cx));
        Self {
            errors: FieldErrors::default(),
            trigger: select_state(
                key_options(keys),
                Some(&config.trigger_key),
                true,
                window,
                cx,
            ),
            hotkey: select_state(
                hotkey_options(keys, config.toggle_enabled_hotkey.as_deref()),
                config.toggle_enabled_hotkey.as_deref(),
                true,
                window,
                cx,
            ),
            action_key: select_state(key_options(keys), None, true, window, cx),
            mode: select_state(
                mode_options(),
                Some(&format!("{:?}", config.mode)),
                false,
                window,
                cx,
            ),
            wait: text_input("100", &text("editor.milliseconds"), window, cx),
            collection: select_state(
                collection_items.clone(),
                config.collection_id.as_deref(),
                true,
                window,
                cx,
            ),
            actions_scroll,
            actions_list,
            collection_items,
            old_key,
            config,
        }
    }

    pub fn read_config(&self, cx: &App) -> Result<SaveMacroDTO, String> {
        let mut config = self.config.clone();
        config.trigger_key = self
            .trigger
            .read(cx)
            .selected_value()
            .cloned()
            .unwrap_or_default();
        config.toggle_enabled_hotkey = self.hotkey.read(cx).selected_value().cloned();
        config.collection_id = self.collection.read(cx).selected_value().cloned();
        config.mode = selected_mode(self.mode.read(cx).selected_value().map(String::as_str))?;
        Ok(config.into())
    }

    /// Update labels/options only when the catalog changes, keeping the selected ID
    /// and an in-progress search across unrelated renders and backend snapshots.
    pub fn sync_collections(
        &mut self,
        collections: &[CollectionDTO],
        window: &mut Window,
        cx: &mut App,
    ) {
        let items = collection_options(collections);
        if items != self.collection_items {
            self.collection.update(cx, |state, cx| {
                state.set_items(items.clone().into(), window, cx)
            });
            self.collection_items = items;
        }
    }

    pub fn add_key(
        &mut self,
        press: Option<bool>,
        keys: &[String],
        cx: &App,
    ) -> Result<(), String> {
        let key = self
            .action_key
            .read(cx)
            .selected_value()
            .cloned()
            .unwrap_or_default();
        if !keys.contains(&key) {
            return Err(text("error.select_action_key"));
        }
        match press {
            Some(press) => self.config.actions.push(Action::Key { key, press }),
            None => {
                self.config.actions.push(Action::Key {
                    key: key.clone(),
                    press: true,
                });
                self.config.actions.push(Action::Key { key, press: false });
            }
        }
        Ok(())
    }

    pub fn add_wait(&mut self, cx: &App) -> Result<(), String> {
        let duration_ms = self
            .wait
            .read(cx)
            .value()
            .trim()
            .parse::<u64>()
            .ok()
            .filter(|duration| *duration > 0)
            .ok_or_else(|| text("error.positive_wait"))?;
        self.config.actions.push(Action::Wait { duration_ms });
        Ok(())
    }

    pub fn move_action(&mut self, from: usize, to: usize) -> bool {
        move_action(&mut self.config.actions, from, to)
    }
}

fn move_action(actions: &mut [Action], from: usize, to: usize) -> bool {
    if from >= actions.len() || to >= actions.len() || from == to {
        return false;
    }
    if from < to {
        actions[from..=to].rotate_left(1);
    } else {
        actions[to..=from].rotate_right(1);
    }
    true
}

fn collection_options(collections: &[CollectionDTO]) -> Vec<SelectOption> {
    collections
        .iter()
        .map(|collection| SelectOption {
            value: collection.id.clone(),
            label: collection.name.clone().into(),
        })
        .collect()
}

fn mode_options() -> Vec<SelectOption> {
    [Mode::Once, Mode::Hold, Mode::Toggle]
        .into_iter()
        .map(|mode| {
            let value = format!("{mode:?}");
            SelectOption {
                label: mode_label(&mode).into(),
                value,
            }
        })
        .collect()
}

fn selected_mode(value: Option<&str>) -> Result<Mode, String> {
    match value {
        Some("Once") => Ok(Mode::Once),
        Some("Hold") => Ok(Mode::Hold),
        Some("Toggle") => Ok(Mode::Toggle),
        _ => Err(text("error.select_mode")),
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/view/ui/macro_draft.rs"]
mod tests;
