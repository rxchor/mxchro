//! Hook-facing fast path. Never acquire the CRUD macro map or touch repositories.
use super::*;

impl MacroManager {
    pub fn process_key_event(
        &self,
        vk: u16,
        is_key_down: bool,
        is_key_up: bool,
        is_injected: bool,
    ) -> KeyEventResult {
        if is_key_down && !is_injected {
            let target = self.toggle_hotkeys.read().unwrap().get(&vk).cloned();
            if let Some(target) = target {
                return KeyEventResult {
                    should_block: true,
                    toggle_request: Some(target),
                };
            }
        }
        let input_state = self.input_states.read().unwrap().get(&vk).cloned();
        let Some(instance) = input_state else {
            return KeyEventResult {
                should_block: false,
                toggle_request: None,
            };
        };
        log::trace!(
            "process_key_event: vk={}, down={}, up={}, injected={}, mode={:?}",
            vk,
            is_key_down,
            is_key_up,
            is_injected,
            instance.mode
        );
        // Only physical input is blocked. Injected actions must reach the target app.
        let enabled = instance.enabled.load(Ordering::Acquire);
        let should_block = enabled && instance.block_original_event && !is_injected;
        let result = KeyEventResult {
            should_block,
            toggle_request: None,
        };
        if !enabled {
            log::trace!("Macro disabled, skipping");
            return result;
        }
        if is_injected {
            if instance.ignore_injected_keys {
                log::trace!("Ignoring injected event (ignore_injected_keys=true)");
                return result;
            }
            match instance.mode {
                Mode::Hold if is_key_up => {
                    log::debug!(
                        "Self-triggering prevented: ignoring injected KeyUp for Hold macro (vk={})",
                        vk
                    );
                    return result;
                }
                Mode::Toggle if is_key_down => {
                    log::debug!(
                        "Self-triggering prevented: ignoring injected KeyDown for Toggle macro (vk={})",
                        vk
                    );
                    return result;
                }
                _ => {}
            }
        }
        if is_key_down {
            match instance.mode {
                Mode::Once => {
                    let mut state = instance.repeat_state.0.lock().unwrap();
                    if !state.is_executing {
                        log::debug!("Once macro triggered (vk={})", vk);
                        state.active = true;
                        instance.repeat_state.1.notify_one();
                    } else {
                        log::trace!("Once macro already executing, ignoring (vk={})", vk);
                    }
                }
                Mode::Hold => {
                    let mut state = instance.repeat_state.0.lock().unwrap();
                    log::debug!("Hold macro activated (vk={})", vk);
                    state.active = true;
                    instance.repeat_state.1.notify_one();
                }
                Mode::Toggle => {
                    let mut state = instance.repeat_state.0.lock().unwrap();
                    state.active = !state.active;
                    log::debug!(
                        "Toggle macro state changed (vk={}, active={})",
                        vk,
                        state.active
                    );
                    if state.active {
                        instance.repeat_state.1.notify_one();
                    }
                }
            }
        } else if is_key_up && instance.mode == Mode::Hold {
            let mut state = instance.repeat_state.0.lock().unwrap();
            log::debug!("Hold macro deactivated (vk={})", vk);
            state.active = false;
        }
        result
    }
}
