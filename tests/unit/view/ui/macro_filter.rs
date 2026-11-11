use super::MacroFilter;
use crate::{application::dto::MacroDTO, domain::types::Mode};

#[test]
fn filters_preserve_all_uncategorized_and_collection_views() {
    let mut config = MacroDTO {
        trigger_key: "A".into(),
        mode: Mode::Once,
        actions_count: 0,
        block_original_event: false,
        ignore_injected_keys: true,
        enabled: false,
        collection_id: None,
        toggle_enabled_hotkey: None,
    };
    assert!(MacroFilter::All.matches(&config));
    assert!(MacroFilter::Uncategorized.matches(&config));
    assert!(!MacroFilter::Collection("work".into()).matches(&config));
    config.collection_id = Some("work".into());
    assert!(MacroFilter::All.matches(&config));
    assert!(!MacroFilter::Uncategorized.matches(&config));
    assert!(MacroFilter::Collection("work".into()).matches(&config));
    assert!(!MacroFilter::Collection("other".into()).matches(&config));
}
