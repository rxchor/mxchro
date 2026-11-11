//! Lifecycle of the persistent per-macro execution thread. No repositories or UI.
use super::*;

impl MacroInstance {
    pub(super) fn input_state(&self) -> MacroInputState {
        MacroInputState {
            mode: self.mode.clone(),
            block_original_event: self.block_original_event,
            ignore_injected_keys: self.ignore_injected_keys,
            enabled: Arc::clone(&self.input_enabled),
            repeat_state: Arc::clone(&self.repeat_state),
        }
    }

    pub fn enable(&mut self) {
        if self.enabled {
            return;
        }
        self.enabled = true;
        {
            let mut state = self.repeat_state.0.lock().unwrap();
            state.active = false;
            state.should_stop = false;
        }
        log::debug!("Enabling macro (vk={})", self.trigger_key);
        self.spawn_repeat_thread();
        self.input_enabled.store(true, Ordering::Release);
    }

    pub fn disable(&mut self) {
        self.enabled = false;
        self.input_enabled.store(false, Ordering::Release);
        log::debug!("Disabling macro (vk={})", self.trigger_key);
        self.stop_repeat_thread();
    }

    pub(super) fn spawn_repeat_thread(&mut self) {
        let mut handle = self.thread_handle.lock().unwrap();
        if handle.is_some() {
            log::trace!("Repeat thread already exists (vk={})", self.trigger_key);
            return;
        }
        let repeat_state = self.repeat_state.clone();
        let action = self.action.clone();
        let mode = self.mode.clone();
        let trigger_key = self.trigger_key;
        log::info!(
            "Spawning repeat thread (vk={}, mode={:?})",
            trigger_key,
            mode
        );
        *handle = Some(std::thread::spawn(move || {
            log::debug!(
                "Repeat thread started (vk={}, mode={:?})",
                trigger_key,
                mode
            );
            loop {
                let (lock, cvar) = &*repeat_state;
                {
                    let mut state = lock.lock().unwrap();
                    while !state.active && !state.should_stop {
                        state = cvar.wait(state).unwrap();
                    }
                    if state.should_stop {
                        log::debug!(
                            "Repeat thread stopping (vk={}, should_stop=true)",
                            trigger_key
                        );
                        break;
                    }
                    state.is_executing = true;
                    log::trace!("Repeat thread activated (vk={})", trigger_key);
                }
                match mode {
                    Mode::Once => {
                        log::debug!("Executing Once action (vk={})", trigger_key);
                        action();
                        let mut state = lock.lock().unwrap();
                        state.active = false;
                        state.is_executing = false;
                        log::trace!("Once execution completed (vk={})", trigger_key);
                    }
                    Mode::Hold | Mode::Toggle => {
                        let mut iteration = 0;
                        loop {
                            {
                                let state = lock.lock().unwrap();
                                if !state.active || state.should_stop {
                                    break;
                                }
                            }
                            iteration += 1;
                            if iteration == 1 || iteration % 100 == 0 {
                                log::trace!("Repeat iteration {} (vk={})", iteration, trigger_key);
                            }
                            action();
                            {
                                let state = lock.lock().unwrap();
                                if !state.active || state.should_stop {
                                    break;
                                }
                            }
                        }
                        log::debug!(
                            "Repeat loop ended after {} iterations (vk={})",
                            iteration,
                            trigger_key
                        );
                        lock.lock().unwrap().is_executing = false;
                    }
                }
                if lock.lock().unwrap().should_stop {
                    log::debug!(
                        "Repeat thread exiting (vk={}, should_stop=true)",
                        trigger_key
                    );
                    break;
                }
            }
            log::debug!("Repeat thread terminated (vk={})", trigger_key);
        }));
    }

    pub(super) fn stop_repeat_thread(&mut self) {
        log::info!("Stopping repeat thread (vk={})", self.trigger_key);
        {
            let (lock, cvar) = &*self.repeat_state;
            let mut state = lock.lock().unwrap();
            state.active = false;
            state.should_stop = true;
            cvar.notify_one();
        }
        if let Some(handle) = self.thread_handle.lock().unwrap().take() {
            log::debug!(
                "Waiting for repeat thread to join (vk={})",
                self.trigger_key
            );
            let _ = handle.join();
            log::debug!(
                "Repeat thread joined successfully (vk={})",
                self.trigger_key
            );
        } else {
            log::trace!("No repeat thread to stop (vk={})", self.trigger_key);
        }
    }
}
