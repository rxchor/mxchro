use std::hash::Hash;
use std::sync::Arc;

use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

use super::dto::{MacroDto, MacroSummaryDto};
use super::key_event::{build_actions, KeyEvent};
use super::key_mode::Mode;
use crate::key_io::key_mapper::KeyMapper;

#[allow(dead_code)]
pub struct Macro {
    pub trigger_key: VIRTUAL_KEY,
    pub mode: Mode,
    pub action: KeyEvent,
    pub block_original_event: bool,
    pub ignore_injected_keys: bool,
    pub repeat: bool,
}

impl Macro {
    pub fn from_dto(dto: MacroDto) -> Option<Self> {
        let mapper = KeyMapper::instance();
        let vk = mapper.string_to_vk(&dto.trigger_key)?;
        let trigger_key = VIRTUAL_KEY(vk);

        let actions = build_actions(&dto.actions);
        let combined_action: KeyEvent = Arc::new(move || {
            for action in &actions {
                action();
            }
        });

        Some(Macro {
            trigger_key,
            mode: dto.mode,
            action: combined_action,
            block_original_event: dto.block_original_event,
            ignore_injected_keys: dto.ignore_injected_keys,
            repeat: false,
        })
    }

    pub fn to_summary(&self, mapper: &KeyMapper) -> Option<MacroSummaryDto> {
        let trigger_key_str = mapper.vk_to_string(&self.trigger_key.0)?;
        Some(MacroSummaryDto {
            trigger_key: trigger_key_str,
            mode: self.mode.clone(),
            actions_count: 0,
            block_original_event: self.block_original_event,
            ignore_injected_keys: self.ignore_injected_keys,
        })
    }
}

impl PartialEq for Macro {
    fn eq(&self, other: &Self) -> bool {
        return self.trigger_key.0 == other.trigger_key.0;
    }
}

impl Hash for Macro {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.trigger_key.0.hash(state);
    }
}  