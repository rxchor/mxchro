use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::core::types::{Action, MacroConfig, MacroSummary, Mode};
use crate::infrastructure::keyboard::mapper::KeyMapper;
use crate::infrastructure::keyboard::simulator::Keyboard;

pub struct RepeatState {
    pub active: bool,
    pub should_stop: bool,
}

struct MacroInstance {
    trigger_key: u16,
    mode: Mode,
    action: Action,
    block_original_event: bool,
    ignore_injected_keys: bool,
    enabled: bool,
    actions_count: usize,
    repeat_state: Arc<(Mutex<RepeatState>, Condvar)>,
    thread_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

pub struct MacroManager {
    macros: RwLock<HashMap<u16, MacroInstance>>,
    key_mapper: &'static KeyMapper,
}

impl MacroManager {
    pub fn new() -> Self {
        MacroManager {
            macros: RwLock::new(HashMap::new()),
            key_mapper: KeyMapper::instance(),
        }
    }

    pub fn add_macro(&self, config: MacroConfig) -> bool {
        let vk = match self.key_mapper.string_to_vk(&config.trigger_key) {
            Some(vk) => vk,
            None => return false,
        };

        let actions = self.build_actions(&config.actions);
        let actions_count = actions.len();
        let combined_action = self.combine_actions(actions);

        let repeat_state = Arc::new((
            Mutex::new(RepeatState {
                active: false,
                should_stop: false,
            }),
            Condvar::new(),
        ));

        let mut instance = MacroInstance {
            trigger_key: vk,
            mode: config.mode.clone(),
            action: combined_action,
            block_original_event: config.block_original_event,
            ignore_injected_keys: config.ignore_injected_keys,
            enabled: config.enabled,
            actions_count,
            repeat_state,
            thread_handle: Arc::new(Mutex::new(None)),
        };

        if instance.enabled {
            instance.spawn_repeat_thread();
        }

        let mut macros = self.macros.write().unwrap();
        if !macros.contains_key(&vk) {
            macros.insert(vk, instance);
            true
        } else {
            false
        }
    }

    pub fn delete_macro(&self, trigger_key: &str) -> bool {
        let vk = match self.key_mapper.string_to_vk(trigger_key) {
            Some(vk) => vk,
            None => return false,
        };

        let mut macros = self.macros.write().unwrap();
        if let Some(mut instance) = macros.remove(&vk) {
            instance.stop_repeat_thread();
            true
        } else {
            false
        }
    }

    pub fn enable_macro(&self, trigger_key: &str) -> bool {
        let vk = match self.key_mapper.string_to_vk(trigger_key) {
            Some(vk) => vk,
            None => return false,
        };

        let mut macros = self.macros.write().unwrap();
        if let Some(instance) = macros.get_mut(&vk) {
            instance.enable();
            true
        } else {
            false
        }
    }

    pub fn disable_macro(&self, trigger_key: &str) -> bool {
        let vk = match self.key_mapper.string_to_vk(trigger_key) {
            Some(vk) => vk,
            None => return false,
        };

        let mut macros = self.macros.write().unwrap();
        if let Some(instance) = macros.get_mut(&vk) {
            instance.disable();
            true
        } else {
            false
        }
    }

    pub fn list_macros(&self) -> Vec<MacroSummary> {
        let macros = self.macros.read().unwrap();
        macros
            .values()
            .map(|m| MacroSummary {
                trigger_key: self.key_mapper.vk_to_string(&m.trigger_key).unwrap_or_default().to_string(),
                mode: m.mode.clone(),
                actions_count: m.actions_count,
                block_original_event: m.block_original_event,
                ignore_injected_keys: m.ignore_injected_keys,
                enabled: m.enabled,
            })
            .collect()
    }

    pub fn get_available_keys(&self) -> Vec<String> {
        self.key_mapper.strings_vec()
    }

    pub fn on_key_down(&self, vk: u16, is_injected: bool) {
        let macros = self.macros.read().unwrap();
        
        if let Some(instance) = macros.get(&vk) {
            if !instance.enabled {
                return;
            }

            if is_injected && instance.ignore_injected_keys {
                return;
            }

            match instance.mode {
                Mode::Once => {
                    let action = instance.action.clone();
                    std::thread::spawn(move || action());
                }
                Mode::Hold => {
                    let mut state = instance.repeat_state.0.lock().unwrap();
                    state.active = true;
                    instance.repeat_state.1.notify_one();
                }
                Mode::Toggle => {
                    let mut state = instance.repeat_state.0.lock().unwrap();
                    state.active = !state.active;
                    if state.active {
                        instance.repeat_state.1.notify_one();
                    }
                }
            }
        }
    }

    pub fn on_key_up(&self, vk: u16) {
        let macros = self.macros.read().unwrap();
        
        if let Some(instance) = macros.get(&vk) {
            if !instance.enabled {
                return;
            }

            if instance.mode == Mode::Hold {
                let mut state = instance.repeat_state.0.lock().unwrap();
                state.active = false;
            }
        }
    }

    pub fn should_block_key(&self, vk: u16) -> bool {
        let macros = self.macros.read().unwrap();
        macros
            .get(&vk)
            .map_or(false, |m| m.enabled && m.block_original_event)
    }

    fn build_actions(&self, events: &[crate::core::types::KeyEvent]) -> Vec<Action> {
        events
            .iter()
            .filter_map(|event| {
                if let Some(sleep_ms) = event.sleep {
                    if event.key.is_none() {
                        return Some(Arc::new(move || {
                            std::thread::sleep(Duration::from_millis(sleep_ms));
                        }) as Action);
                    }
                }

                if let Some(key_str) = &event.key {
                    let vk = self.key_mapper.string_to_vk(key_str)?;
                    let hold_duration = event.hold.map(|ms| Duration::from_millis(ms));

                    return Some(Arc::new(move || {
                        Keyboard::press_key(vk, hold_duration);
                    }) as Action);
                }

                None
            })
            .collect()
    }

    fn combine_actions(&self, actions: Vec<Action>) -> Action {
        Arc::new(move || {
            for action in &actions {
                action();
            }
        })
    }
}

impl MacroInstance {
    fn enable(&mut self) {
        self.enabled = true;
        {
            let (lock, _) = &*self.repeat_state;
            let mut state = lock.lock().unwrap();
            state.active = false;
            state.should_stop = false;
        }
        self.spawn_repeat_thread();
    }

    fn disable(&mut self) {
        self.enabled = false;
        self.stop_repeat_thread();
    }

    fn spawn_repeat_thread(&mut self) {
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
                    std::thread::sleep(Duration::from_millis(10));
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

    fn stop_repeat_thread(&mut self) {
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
}
