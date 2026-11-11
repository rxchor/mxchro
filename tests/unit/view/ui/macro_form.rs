use super::MacroOption;
use crate::{application::dto::MacroEditDTO, domain::types::Mode};

#[test]
fn controlled_options_update_only_the_requested_flag() {
    let mut config = MacroEditDTO {
        trigger_key: "A".into(),
        mode: Mode::Once,
        actions: vec![],
        block_original_event: false,
        ignore_injected_keys: true,
        enabled: false,
        collection_id: None,
        toggle_enabled_hotkey: None,
    };
    MacroOption::Enabled.set(&mut config, true);
    assert!(MacroOption::Enabled.get(&config));
    assert!(!MacroOption::BlockOriginal.get(&config));
    assert!(MacroOption::IgnoreInjected.get(&config));
    MacroOption::BlockOriginal.set(&mut config, true);
    MacroOption::IgnoreInjected.set(&mut config, false);
    assert!(config.enabled && config.block_original_event);
    assert!(!config.ignore_injected_keys);
}
