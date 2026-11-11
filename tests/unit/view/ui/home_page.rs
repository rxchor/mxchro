use super::{group_macros, has_hotkey_issue, macro_card};
use crate::application::dto::{
    CollectionDTO, HotkeyIssueDTO, HotkeyIssueKind, HotkeyOwnerDTO, MacroDTO,
};
use crate::domain::types::Mode;

fn collection(id: &str, name: &str, order: i32) -> CollectionDTO {
    CollectionDTO {
        id: id.into(),
        name: name.into(),
        order,
        enabled: false,
        macros_count: 999,
        toggle_enabled_hotkey: None,
    }
}

fn config(key: &str, collection: Option<&str>) -> MacroDTO {
    MacroDTO {
        trigger_key: key.into(),
        mode: Mode::Once,
        actions_count: 1,
        enabled: false,
        block_original_event: false,
        ignore_injected_keys: true,
        collection_id: collection.map(str::to_owned),
        toggle_enabled_hotkey: None,
    }
}

#[test]
fn groups_follow_collection_order_and_keep_macro_order_and_duplicate_names() {
    let groups = group_macros(
        vec![
            config("F1", Some("a")),
            config("F2", Some("b")),
            config("F3", Some("a")),
            config("F4", None),
        ],
        &[
            collection("a", "Same name", 2),
            collection("b", "Same name", 1),
            collection("empty", "Empty", 0),
        ],
    );
    assert_eq!(groups.len(), 3);
    assert_eq!(groups[0].collection_id.as_deref(), Some("b"));
    assert_eq!(groups[1].collection_id.as_deref(), Some("a"));
    assert_eq!(groups[0].name, groups[1].name);
    assert_eq!(
        groups[1]
            .macros
            .iter()
            .map(|m| m.trigger_key.as_str())
            .collect::<Vec<_>>(),
        vec!["F1", "F3"]
    );
    assert!(groups[2].collection_id.is_none());
    assert_eq!(groups[2].macros[0].trigger_key, "F4");
}

#[test]
fn orphaned_assignments_remain_visible_without_changing_the_dto() {
    let groups = group_macros(vec![config("F1", Some("missing")), config("F2", None)], &[]);
    assert_eq!(groups.len(), 1);
    assert!(groups[0].collection_id.is_none());
    assert_eq!(groups[0].macros.len(), 2);
    assert_eq!(
        groups[0].macros[0].collection_id.as_deref(),
        Some("missing")
    );
}

#[test]
fn empty_catalogs_and_empty_collections_produce_no_groups() {
    assert!(group_macros(vec![], &[]).is_empty());
    assert!(group_macros(vec![], &[collection("a", "A", 0)]).is_empty());
}

#[test]
fn grouped_headers_keep_their_own_hotkey_not_a_macros_individual_assignment() {
    let mut collection = collection("work", "Work", 0);
    collection.toggle_enabled_hotkey = Some("F8".into());
    let mut grouped = config("F1", Some("work"));
    grouped.toggle_enabled_hotkey = Some("F9".into());
    let groups = group_macros(vec![grouped, config("F2", None)], &[collection]);
    assert_eq!(groups[0].toggle_hotkey.as_deref(), Some("F8"));
    assert_eq!(
        groups[0].macros[0].toggle_enabled_hotkey.as_deref(),
        Some("F9")
    );
    assert!(groups[1].toggle_hotkey.is_none());
}

#[test]
fn inactive_assignments_are_matched_by_typed_owner_not_name_or_shared_key() {
    let issues = vec![HotkeyIssueDTO {
        owner: HotkeyOwnerDTO::Collection("F6".into()),
        hotkey: "F8".into(),
        kind: HotkeyIssueKind::Duplicate,
    }];
    assert!(has_hotkey_issue(
        &issues,
        &HotkeyOwnerDTO::Collection("F6".into())
    ));
    assert!(!has_hotkey_issue(
        &issues,
        &HotkeyOwnerDTO::Macro("F6".into())
    ));
    let card = macro_card(
        config("F6", None),
        &issues,
        false,
        false,
        std::rc::Rc::new(|_, _, _| {}),
    );
    assert!(!card.hotkey_inactive);
    let issues = vec![HotkeyIssueDTO {
        owner: HotkeyOwnerDTO::Macro("F6".into()),
        hotkey: "F8".into(),
        kind: HotkeyIssueKind::InvalidKey,
    }];
    let card = macro_card(
        config("F6", None),
        &issues,
        false,
        false,
        std::rc::Rc::new(|_, _, _| {}),
    );
    assert!(card.hotkey_inactive);
}
