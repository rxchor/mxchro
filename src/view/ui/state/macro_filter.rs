use crate::application::dto::{CollectionDTO, MacroDTO};
use crate::view::i18n::text;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MacroFilter {
    All,
    Uncategorized,
    Collection(String),
}

impl MacroFilter {
    pub fn collection_id(&self) -> Option<&str> {
        match self {
            Self::Collection(id) => Some(id),
            _ => None,
        }
    }

    pub fn matches(&self, config: &MacroDTO) -> bool {
        match self {
            Self::All => true,
            Self::Uncategorized => config.collection_id.is_none(),
            Self::Collection(id) => config.collection_id.as_ref() == Some(id),
        }
    }

    pub fn title(&self, collections: &[CollectionDTO]) -> String {
        match self {
            Self::All => text("sidebar.all_macros"),
            Self::Uncategorized => text("common.no_collection"),
            Self::Collection(id) => collections
                .iter()
                .find(|c| &c.id == id)
                .map(|c| c.name.clone())
                .unwrap_or_default(),
        }
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/view/ui/macro_filter.rs"]
mod tests;
