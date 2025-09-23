use std::hash::Hash;

use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

use crate::model::{key_event::KeyEvent, key_mode::Mode};

#[allow(dead_code)]
pub struct Macro {
    pub trigger_key: VIRTUAL_KEY,
    pub mode: Mode, // TOGGLE, HOLD, ONCE
    pub action: KeyEvent,
    pub block_original_event: bool,
    pub ignore_injected_keys: bool, // This is heavily important
    pub repeat: bool, // Will only be used for HOLD and TOGGLE loops
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