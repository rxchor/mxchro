use std::sync::Arc;
use std::time::Duration;

use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

use super::dto::KeyEventDto;
use crate::key_io::keyboard::Keyboard;
use crate::key_io::key_mapper::KeyMapper;

#[allow(dead_code)]
pub type KeyEvent = Arc<dyn Fn() + Send + Sync + 'static>;

pub fn build_action(dto: &KeyEventDto) -> Option<KeyEvent> {
    if let Some(sleep_ms) = dto.sleep {
        if dto.key.is_none() {
            let action: KeyEvent = Arc::new(move || {
                std::thread::sleep(Duration::from_millis(sleep_ms));
            });
            return Some(action);
        }
    }

    if let Some(key_str) = &dto.key {
        let mapper = KeyMapper::instance();
        let vk = mapper.string_to_vk(key_str)?;
        let virtual_key = VIRTUAL_KEY(vk);
        let hold_duration = dto.hold.map(|ms| Duration::from_millis(ms));

        let action: KeyEvent = Arc::new(move || {
            Keyboard::press_key(virtual_key, hold_duration);
        });
        return Some(action);
    }

    None
}

pub fn build_actions(dtos: &[KeyEventDto]) -> Vec<KeyEvent> {
    dtos.iter()
        .filter_map(|dto| build_action(dto))
        .collect()
}