use std::hash::Hash;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

use super::dto::{MacroDto, MacroSummaryDto};
use super::key_event::{build_actions, KeyEvent};
use super::key_mode::Mode;
use crate::key_io::key_mapper::KeyMapper;

pub struct RepeatState {
    pub active: bool,
    pub should_stop: bool,
}

#[allow(dead_code)]
pub struct Macro {
    pub trigger_key: VIRTUAL_KEY,
    pub mode: Mode,
    pub action: KeyEvent,
    pub block_original_event: bool,
    pub ignore_injected_keys: bool,
    pub enabled: bool,
    pub actions_count: usize,
    pub repeat_state: Arc<(Mutex<RepeatState>, Condvar)>,
    thread_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl Macro {
    pub fn from_dto(dto: MacroDto) -> Option<Self> {
        let mapper = KeyMapper::instance();
        let vk = mapper.string_to_vk(&dto.trigger_key)?;
        let trigger_key = VIRTUAL_KEY(vk);

        let actions = build_actions(&dto.actions);
        let actions_count = actions.len();
        let combined_action: KeyEvent = Arc::new(move || {
            for action in &actions {
                action();
            }
        });

        let repeat_state = Arc::new((
            Mutex::new(RepeatState {
                active: false,
                should_stop: false,
            }),
            Condvar::new(),
        ));

        Some(Macro {
            trigger_key,
            mode: dto.mode,
            action: combined_action,
            block_original_event: dto.block_original_event,
            ignore_injected_keys: dto.ignore_injected_keys,
            enabled: dto.enabled,
            actions_count,
            repeat_state,
            thread_handle: Arc::new(Mutex::new(None)),
        })
    }

    pub fn to_summary(&self, mapper: &KeyMapper) -> Option<MacroSummaryDto> {
        let trigger_key_str = mapper.vk_to_string(&self.trigger_key.0)?.to_string();
        Some(MacroSummaryDto {
            trigger_key: trigger_key_str,
            mode: self.mode.clone(),
            actions_count: self.actions_count,
            block_original_event: self.block_original_event,
            ignore_injected_keys: self.ignore_injected_keys,
            enabled: self.enabled,
        })
    }

    pub fn spawn_repeat_thread(&mut self) {
        let mut handle = self.thread_handle.lock().unwrap();
        if handle.is_some() {
            return;
        }

        let repeat_state = self.repeat_state.clone();
        let action = self.action.clone();

        let join_handle = std::thread::spawn(move || {
            loop {
                let (lock, cvar) = &*repeat_state;

                {
                    let mut state = lock.lock().unwrap();
                    while !state.active && !state.should_stop {
                        state = cvar.wait(state).unwrap();
                    }
                    if state.should_stop {
                        break;
                    }
                }

                loop {
                    {
                        let state = lock.lock().unwrap();
                        if !state.active || state.should_stop {
                            break;
                        }
                    }

                    action();
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }

                {
                    let state = lock.lock().unwrap();
                    if state.should_stop {
                        break;
                    }
                }
            }
        });

        *handle = Some(join_handle);
    }

    pub fn stop_repeat_thread(&mut self) {
        {
            let (lock, cvar) = &*self.repeat_state;
            let mut state = lock.lock().unwrap();
            state.active = false;
            state.should_stop = true;
            cvar.notify_one();
        }

        if let Some(handle) = self.thread_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
        {
            let (lock, _cvar) = &*self.repeat_state;
            let mut state = lock.lock().unwrap();
            state.active = false;
            state.should_stop = false;
        }
        self.spawn_repeat_thread();
    }

    pub fn disable(&mut self) {
        self.enabled = false;
        self.stop_repeat_thread();
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