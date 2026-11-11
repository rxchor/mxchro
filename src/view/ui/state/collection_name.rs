use crate::view::{i18n::text, ui::components::app_input_field::text_input};
use gpui_kit::{component::input::InputState, *};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CollectionNameTarget {
    Create,
    Rename(String),
}

#[derive(Clone)]
pub struct CollectionNameDraft {
    pub input: Entity<InputState>,
    pub target: Option<CollectionNameTarget>,
    pub error: Option<String>,
}

impl CollectionNameDraft {
    pub fn new(window: &mut Window, cx: &mut App) -> Self {
        Self {
            input: text_input("", &text("sidebar.collection_name"), window, cx),
            target: None,
            error: None,
        }
    }

    pub fn close(&mut self) {
        self.target = None;
        self.error = None;
    }
}
