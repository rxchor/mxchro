//! Repository-port fake. No PoloDB, filesystem, hooks or OS input.
use super::app::{
    application::ports::{CollectionRepository, MacroRepository, Repositories},
    domain::types::{Collection, Macro},
};
use std::{
    collections::HashMap,
    sync::atomic::{AtomicBool, Ordering},
    sync::{Arc, Mutex},
};

#[derive(Default)]
struct Data {
    macros: HashMap<String, Macro>,
    collections: HashMap<String, Collection>,
}
struct Store(Arc<Mutex<Data>>, Arc<AtomicBool>);

pub fn repositories() -> Arc<Repositories> {
    repositories_with_hotkey_failures(Arc::new(AtomicBool::new(false)))
}

pub fn repositories_with_hotkey_failures(fail: Arc<AtomicBool>) -> Arc<Repositories> {
    let data = Arc::new(Mutex::new(Data::default()));
    Arc::new(Repositories {
        macro_repo: Arc::new(Store(data.clone(), fail.clone())),
        collection_repo: Arc::new(Store(data, fail)),
    })
}

impl MacroRepository for Store {
    fn load_all(&self) -> Vec<Macro> {
        self.0.lock().unwrap().macros.values().cloned().collect()
    }
    fn insert(&self, value: &Macro) -> Result<(), String> {
        self.0
            .lock()
            .unwrap()
            .macros
            .insert(value.trigger_key.clone(), value.clone());
        Ok(())
    }
    fn delete(&self, key: &str) -> Result<(), String> {
        self.0.lock().unwrap().macros.remove(key);
        Ok(())
    }
    fn set_enabled(&self, key: &str, enabled: bool) -> Result<(), String> {
        if let Some(value) = self.0.lock().unwrap().macros.get_mut(key) {
            value.enabled = enabled;
        }
        Ok(())
    }
    fn replace(&self, old_key: &str, value: &Macro) -> Result<(), String> {
        let mut data = self.0.lock().unwrap();
        data.macros.remove(old_key);
        data.macros.insert(value.trigger_key.clone(), value.clone());
        Ok(())
    }
    fn wipe_data(&self) -> Result<(), String> {
        self.0.lock().unwrap().macros.clear();
        Ok(())
    }
    fn set_toggle_hotkey(&self, key: &str, hotkey: Option<String>) -> Result<(), String> {
        if self.1.load(Ordering::Acquire) {
            return Err("Simulated hotkey write failure".into());
        }
        if let Some(value) = self.0.lock().unwrap().macros.get_mut(key) {
            value.toggle_enabled_hotkey = hotkey;
        }
        Ok(())
    }
}

impl CollectionRepository for Store {
    fn load_all(&self) -> Vec<Collection> {
        self.0
            .lock()
            .unwrap()
            .collections
            .values()
            .cloned()
            .collect()
    }
    fn insert(&self, value: &Collection) -> Result<(), String> {
        self.0
            .lock()
            .unwrap()
            .collections
            .insert(value.id.clone(), value.clone());
        Ok(())
    }
    fn delete(&self, id: &str) -> Result<(), String> {
        let mut data = self.0.lock().unwrap();
        data.collections.remove(id);
        for value in data
            .macros
            .values_mut()
            .filter(|m| m.collection_id.as_deref() == Some(id))
        {
            value.collection_id = None;
        }
        Ok(())
    }
    fn set_enabled_with_macros(&self, id: &str, enabled: bool) -> Result<(), String> {
        let mut data = self.0.lock().unwrap();
        if let Some(value) = data.collections.get_mut(id) {
            value.enabled = enabled;
        }
        for value in data
            .macros
            .values_mut()
            .filter(|m| m.collection_id.as_deref() == Some(id))
        {
            value.enabled = enabled;
        }
        Ok(())
    }
    fn update_name(&self, id: &str, name: &str) -> Result<(), String> {
        if let Some(value) = self.0.lock().unwrap().collections.get_mut(id) {
            value.name = name.into();
        }
        Ok(())
    }
    fn reorder(&self, ids: &[String]) -> Result<(), String> {
        let mut data = self.0.lock().unwrap();
        for (order, id) in ids.iter().enumerate() {
            if let Some(value) = data.collections.get_mut(id) {
                value.order = order as i32;
            }
        }
        Ok(())
    }
    fn get_max_order(&self) -> i32 {
        self.0
            .lock()
            .unwrap()
            .collections
            .values()
            .map(|c| c.order)
            .max()
            .unwrap_or(-1)
    }
    fn wipe_data(&self) -> Result<(), String> {
        self.0.lock().unwrap().collections.clear();
        Ok(())
    }
    fn set_toggle_hotkey(&self, id: &str, hotkey: Option<String>) -> Result<(), String> {
        if self.1.load(Ordering::Acquire) {
            return Err("Simulated hotkey write failure".into());
        }
        if let Some(value) = self.0.lock().unwrap().collections.get_mut(id) {
            value.toggle_enabled_hotkey = hotkey;
        }
        Ok(())
    }
}
