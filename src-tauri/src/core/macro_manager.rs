use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::core::types::{Action, ActionCallback, Macro, MacroView, Mode};
use crate::core::persistence::Persistence;
use crate::infrastructure::keyboard::mapper::KeyMapper;
use crate::infrastructure::keyboard::simulator::Keyboard;

pub struct RepeatState {
    pub active: bool,
    pub should_stop: bool,
    pub is_executing: bool,  // Para Mode::Once: indica si la acción se está ejecutando
}

struct MacroInstance {
    trigger_key: u16,
    mode: Mode,
    action: ActionCallback,
    actions_config: Vec<Action>, // Acciones originales para editar
    block_original_event: bool,
    ignore_injected_keys: bool,
    enabled: bool,
    actions_count: usize,
    repeat_state: Arc<(Mutex<RepeatState>, Condvar)>,
    thread_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

/// Resultado de procesar un evento de tecla.
pub struct KeyEventResult {
    pub should_block: bool,
}

pub struct MacroManager {
    macros: RwLock<HashMap<u16, MacroInstance>>,
    key_mapper: &'static KeyMapper,
    persistence: Persistence,
}

impl MacroManager {
    pub fn new() -> Self {
        let manager = MacroManager {
            macros: RwLock::new(HashMap::new()),
            key_mapper: KeyMapper::instance(),
            persistence: Persistence::new(),
        };
        
        // Cargar macros desde el archivo
        manager.load_from_file();
        
        manager
    }

    pub fn load_from_file(&self) {
        let configs = self.persistence.load_all();
        let mut macros = self.macros.write().unwrap();
        
        for config in configs {
            if let Some(vk) = self.key_mapper.string_to_vk(&config.trigger_key) {
                if !macros.contains_key(&vk) {
                    let mut instance = self.create_instance(config);
                    if instance.enabled {
                        instance.spawn_repeat_thread();
                    }
                    macros.insert(vk, instance);
                }
            }
        }
        
        log::info!("Loaded {} macros from persistence", macros.len());
    }

    fn trigger_key_str(&self, vk: &u16) -> String {
        self.key_mapper.vk_to_string(vk).unwrap_or_default().to_string()
    }

    fn build_config(&self, instance: &MacroInstance) -> Macro {
        Macro {
            trigger_key: self.trigger_key_str(&instance.trigger_key),
            mode: instance.mode.clone(),
            actions: instance.actions_config.clone(),
            block_original_event: instance.block_original_event,
            ignore_injected_keys: instance.ignore_injected_keys,
            enabled: instance.enabled,
        }
    }

    pub fn wipe_data(&self) -> Result<(), String> {
        // Detener todos los hilos de repetición
        let mut macros = self.macros.write().unwrap();
        for (_, mut instance) in macros.drain() {
            instance.stop_repeat_thread();
        }
        
        // Limpiar el archivo
        self.persistence.wipe_data()?;
        
        log::info!("Wiped all macro data");
        Ok(())
    }

    fn create_instance(&self, macro_entity: Macro) -> MacroInstance {
        let vk = self.key_mapper.string_to_vk(&macro_entity.trigger_key).unwrap();
        let actions = self.build_actions(&macro_entity.actions);
        let actions_count = actions.len();
        let combined_action = self.combine_actions(actions);

        let repeat_state = Arc::new((
            Mutex::new(RepeatState {
                active: false,
                should_stop: false,
                is_executing: false,
            }),
            Condvar::new(),
        ));

        MacroInstance {
            trigger_key: vk,
            mode: macro_entity.mode.clone(),
            action: combined_action,
            actions_config: macro_entity.actions.clone(),
            block_original_event: macro_entity.block_original_event,
            ignore_injected_keys: macro_entity.ignore_injected_keys,
            enabled: macro_entity.enabled,
            actions_count,
            repeat_state,
            thread_handle: Arc::new(Mutex::new(None)),
        }
    }

    pub fn add_macro(&self, macro_entity: Macro) -> bool {
        let vk = match self.key_mapper.string_to_vk(&macro_entity.trigger_key) {
            Some(vk) => vk,
            None => {
                log::warn!("add_macro: trigger key '{}' not recognized", macro_entity.trigger_key);
                return false;
            }
        };

        let mut macros = self.macros.write().unwrap();
        if macros.contains_key(&vk) {
            log::warn!("add_macro: trigger key '{}' (vk={}) already exists", macro_entity.trigger_key, vk);
            return false;
        }

        let mut instance = self.create_instance(macro_entity);
        if instance.enabled {
            instance.spawn_repeat_thread();
        }

        let config = self.build_config(&instance);
        log::info!("Macro added: trigger={} (vk={})", instance.trigger_key, vk);
        macros.insert(vk, instance);
        drop(macros);

        if let Err(e) = self.persistence.insert(&config) {
            log::error!("Failed to persist new macro: {}", e);
        }
        true
    }

    pub fn get_macro(&self, trigger_key: &str) -> Option<Macro> {
        let vk = match self.key_mapper.string_to_vk(trigger_key) {
            Some(vk) => vk,
            None => return None,
        };

        let macros = self.macros.read().unwrap();
        macros.get(&vk).map(|instance| Macro {
            trigger_key: self.key_mapper.vk_to_string(&instance.trigger_key).unwrap_or(trigger_key).to_string(),
            mode: instance.mode.clone(),
            actions: instance.actions_config.clone(),
            block_original_event: instance.block_original_event,
            ignore_injected_keys: instance.ignore_injected_keys,
            enabled: instance.enabled,
        })
    }

    pub fn update_macro(&self, old_trigger_key: &str, macro_entity: Macro) -> bool {
        let old_vk = match self.key_mapper.string_to_vk(old_trigger_key) {
            Some(vk) => vk,
            None => {
                log::warn!("update_macro: old trigger key '{}' not recognized", old_trigger_key);
                return false;
            }
        };

        let new_vk = match self.key_mapper.string_to_vk(&macro_entity.trigger_key) {
            Some(vk) => vk,
            None => {
                log::warn!("update_macro: new trigger key '{}' not recognized", macro_entity.trigger_key);
                return false;
            }
        };

        let mut macros = self.macros.write().unwrap();
        
        // Si cambia la tecla trigger, verificar que la nueva no exista
        if old_vk != new_vk && macros.contains_key(&new_vk) {
            log::warn!("update_macro: new trigger key '{}' (vk={}) already exists", macro_entity.trigger_key, new_vk);
            return false;
        }

        // Eliminar la macro antigua
        if let Some(mut old_instance) = macros.remove(&old_vk) {
            old_instance.stop_repeat_thread();
            log::info!("Macro updated: old trigger={} (vk={})", old_trigger_key, old_vk);
        } else {
            log::warn!("update_macro: macro with trigger key '{}' (vk={}) not found", old_trigger_key, old_vk);
            return false;
        }

        // Crear la nueva macro
        let actions = self.build_actions(&macro_entity.actions);
        let actions_count = actions.len();
        let combined_action = self.combine_actions(actions);

        let repeat_state = Arc::new((
            Mutex::new(RepeatState {
                active: false,
                should_stop: false,
                is_executing: false,
            }),
            Condvar::new(),
        ));

        let mut instance = MacroInstance {
            trigger_key: new_vk,
            mode: macro_entity.mode.clone(),
            action: combined_action,
            actions_config: macro_entity.actions.clone(),
            block_original_event: macro_entity.block_original_event,
            ignore_injected_keys: macro_entity.ignore_injected_keys,
            enabled: macro_entity.enabled,
            actions_count,
            repeat_state,
            thread_handle: Arc::new(Mutex::new(None)),
        };

        if instance.enabled {
            instance.spawn_repeat_thread();
        }

        log::info!("Macro updated: new trigger={} (vk={}), mode={:?}, actions={}, enabled={}",
            macro_entity.trigger_key, new_vk, instance.mode, actions_count, instance.enabled);

        macros.insert(new_vk, instance);
        drop(macros);

        if let Err(e) = self.persistence.replace(old_trigger_key, &macro_entity) {
            log::error!("Failed to persist macro update: {}", e);
        }
        true
    }

    pub fn delete_macro(&self, trigger_key: &str) -> bool {
        let vk = match self.key_mapper.string_to_vk(trigger_key) {
            Some(vk) => vk,
            None => {
                log::warn!("delete_macro: trigger key '{}' not recognized", trigger_key);
                return false;
            }
        };

        let mut macros = self.macros.write().unwrap();
        if let Some(mut instance) = macros.remove(&vk) {
            log::info!("Macro deleted: trigger={} (vk={})", trigger_key, vk);
            instance.stop_repeat_thread();
            drop(macros);

            if let Err(e) = self.persistence.delete(trigger_key) {
                log::error!("Failed to persist macro deletion: {}", e);
            }
            true
        } else {
            log::warn!("delete_macro: trigger key '{}' (vk={}) not found", trigger_key, vk);
            false
        }
    }

    pub fn enable_macro(&self, trigger_key: &str) -> bool {
        let vk = match self.key_mapper.string_to_vk(trigger_key) {
            Some(vk) => vk,
            None => {
                log::warn!("enable_macro: trigger key '{}' not recognized", trigger_key);
                return false;
            }
        };

        let mut macros = self.macros.write().unwrap();
        if let Some(instance) = macros.get_mut(&vk) {
            log::info!("Macro enabled: trigger={} (vk={})", trigger_key, vk);
            instance.enable();
            drop(macros);

            if let Err(e) = self.persistence.set_enabled(trigger_key, true) {
                log::error!("Failed to persist enable for '{}': {}", trigger_key, e);
            }
            true
        } else {
            log::warn!("enable_macro: trigger key '{}' (vk={}) not found", trigger_key, vk);
            false
        }
    }

    pub fn disable_macro(&self, trigger_key: &str) -> bool {
        let vk = match self.key_mapper.string_to_vk(trigger_key) {
            Some(vk) => vk,
            None => {
                log::warn!("disable_macro: trigger key '{}' not recognized", trigger_key);
                return false;
            }
        };

        let mut macros = self.macros.write().unwrap();
        if let Some(instance) = macros.get_mut(&vk) {
            log::info!("Macro disabled: trigger={} (vk={})", trigger_key, vk);
            instance.disable();
            drop(macros);

            if let Err(e) = self.persistence.set_enabled(trigger_key, false) {
                log::error!("Failed to persist disable for '{}': {}", trigger_key, e);
            }
            true
        } else {
            log::warn!("disable_macro: trigger key '{}' (vk={}) not found", trigger_key, vk);
            false
        }
    }

    pub fn list_macros(&self) -> Vec<MacroView> {
        let macros = self.macros.read().unwrap();
        macros
            .values()
            .map(|m| MacroView {
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

    /// Procesa un evento de tecla en una sola adquisición del lock,
    /// evitando deadlocks por múltiples reads anidados.
    ///
    /// Retorna un `KeyEventResult` indicando si la tecla debe bloquearse.
    pub fn process_key_event(&self, vk: u16, is_key_down: bool, is_key_up: bool, is_injected: bool) -> KeyEventResult {
        let macros = self.macros.read().unwrap();

        let instance = match macros.get(&vk) {
            Some(inst) => inst,
            None => return KeyEventResult { should_block: false },
        };

        log::trace!("process_key_event: vk={}, down={}, up={}, injected={}, mode={:?}", 
            vk, is_key_down, is_key_up, is_injected, instance.mode);

        // Solo bloquear eventos originales (físicos), no los inyectados.
        // Si bloqueáramos los inyectados, las acciones de la propia macro no llegarían a la aplicación.
        let should_block = instance.enabled && instance.block_original_event && !is_injected;

        if !instance.enabled {
            log::trace!("Macro disabled, skipping");
            return KeyEventResult { should_block };
        }

        if is_injected {
            if instance.ignore_injected_keys {
                log::trace!("Ignoring injected event (ignore_injected_keys=true)");
                return KeyEventResult { should_block };
            }
            // Protección contra self-triggering: incluso si ignore_injected_keys es false,
            // ignoramos ciertos eventos inyectados para evitar que la macro se interfiera consigo misma.
            match instance.mode {
                // KeyUp inyectado desactivaría el Hold prematuramente (la macro envía KeyUp de la misma tecla)
                Mode::Hold if is_key_up => {
                    log::debug!("Self-triggering prevented: ignoring injected KeyUp for Hold macro (vk={})", vk);
                    return KeyEventResult { should_block };
                },
                // KeyDown inyectado togglería el macro off (la macro envía KeyDown de la misma tecla)
                Mode::Toggle if is_key_down => {
                    log::debug!("Self-triggering prevented: ignoring injected KeyDown for Toggle macro (vk={})", vk);
                    return KeyEventResult { should_block };
                },
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
                    log::debug!("Toggle macro state changed (vk={}, active={})", vk, state.active);
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

        KeyEventResult { should_block }
    }

    fn build_actions(&self, events: &[Action]) -> Vec<ActionCallback> {
        events
            .iter()
            .filter_map(|event| match event {
                Action::Wait { duration_ms } => {
                    let duration = *duration_ms;
                    Some(Arc::new(move || {
                        std::thread::sleep(Duration::from_millis(duration));
                    }) as ActionCallback)
                }
                Action::Key { key, press } => {
                    let vk = self.key_mapper.string_to_vk(key)?;
                    let press = *press;

                    Some(Arc::new(move || {
                        if press {
                            Keyboard::key_down(vk);
                        } else {
                            Keyboard::key_up(vk);
                        }
                    }) as ActionCallback)
                }
            })
            .collect()
    }

    fn combine_actions(&self, actions: Vec<ActionCallback>) -> ActionCallback {
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
        log::debug!("Enabling macro (vk={})", self.trigger_key);
        self.spawn_repeat_thread();
    }

    fn disable(&mut self) {
        self.enabled = false;
        log::debug!("Disabling macro (vk={})", self.trigger_key);
        self.stop_repeat_thread();
    }

    fn spawn_repeat_thread(&mut self) {
        let mut handle = self.thread_handle.lock().unwrap();
        if handle.is_some() {
            log::trace!("Repeat thread already exists (vk={})", self.trigger_key);
            return;
        }

        let repeat_state = self.repeat_state.clone();
        let action = self.action.clone();
        let mode = self.mode.clone();
        let trigger_key = self.trigger_key;

        log::info!("Spawning repeat thread (vk={}, mode={:?})", trigger_key, mode);

        let join_handle = std::thread::spawn(move || {
            log::debug!("Repeat thread started (vk={}, mode={:?})", trigger_key, mode);
            loop {
                let (lock, cvar) = &*repeat_state;

                // Esperar hasta que se active o se deba detener
                {
                    let mut state = lock.lock().unwrap();
                    while !state.active && !state.should_stop {
                        state = cvar.wait(state).unwrap();
                    }
                    if state.should_stop {
                        log::debug!("Repeat thread stopping (vk={}, should_stop=true)", trigger_key);
                        break;
                    }
                    state.is_executing = true;
                    log::trace!("Repeat thread activated (vk={})", trigger_key);
                }

                match mode {
                    Mode::Once => {
                        // Ejecutar una sola vez y resetear
                        log::debug!("Executing Once action (vk={})", trigger_key);
                        action();

                        let mut state = lock.lock().unwrap();
                        state.active = false;
                        state.is_executing = false;
                        log::trace!("Once execution completed (vk={})", trigger_key);
                    }
                    Mode::Hold | Mode::Toggle => {
                        // Repetir continuamente mientras esté activo
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

                            std::thread::sleep(Duration::from_millis(10));
                        }

                        log::debug!("Repeat loop ended after {} iterations (vk={})", iteration, trigger_key);
                        let mut state = lock.lock().unwrap();
                        state.is_executing = false;
                    }
                }

                {
                    let state = lock.lock().unwrap();
                    if state.should_stop {
                        log::debug!("Repeat thread exiting (vk={}, should_stop=true)", trigger_key);
                        break;
                    }
                }
            }
            log::debug!("Repeat thread terminated (vk={})", trigger_key);
        });

        *handle = Some(join_handle);
    }

    fn stop_repeat_thread(&mut self) {
        log::info!("Stopping repeat thread (vk={})", self.trigger_key);
        {
            let (lock, cvar) = &*self.repeat_state;
            let mut state = lock.lock().unwrap();
            state.active = false;
            state.should_stop = true;
            cvar.notify_one();
        }

        if let Some(handle) = self.thread_handle.lock().unwrap().take() {
            log::debug!("Waiting for repeat thread to join (vk={})", self.trigger_key);
            let _ = handle.join();
            log::debug!("Repeat thread joined successfully (vk={})", self.trigger_key);
        } else {
            log::trace!("No repeat thread to stop (vk={})", self.trigger_key);
        }
    }
}
