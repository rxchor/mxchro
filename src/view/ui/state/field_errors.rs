use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EditorField {
    Trigger,
    Mode,
    Collection,
    Hotkey,
    Actions,
    ActionKey,
    Wait,
}

#[derive(Clone, Default)]
pub struct FieldErrors(BTreeMap<EditorField, String>);

impl FieldErrors {
    pub fn get(&self, field: EditorField) -> Option<String> {
        self.0.get(&field).cloned()
    }

    pub fn set(&mut self, field: EditorField, error: Option<String>) {
        if let Some(error) = error {
            self.0.insert(field, error);
        } else {
            self.0.remove(&field);
        }
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}
