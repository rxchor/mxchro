use super::move_action;
use super::{collection_options, mode_options, selected_mode};
use crate::application::dto::CollectionDTO;
use crate::domain::types::{Action, Mode};

#[test]
fn arbitrary_moves_insert_instead_of_swapping_and_preserve_the_other_actions() {
    for count in 1..8 {
        for from in 0..count {
            for to in 0..count {
                let mut actions: Vec<_> = (0..count)
                    .map(|n| Action::Wait {
                        duration_ms: n as u64 + 1,
                    })
                    .collect();
                let mut expected: Vec<_> = (1..=count as u64).collect();
                let moved = expected.remove(from);
                expected.insert(to, moved);
                assert_eq!(move_action(&mut actions, from, to), from != to);
                let result: Vec<_> = actions
                    .iter()
                    .map(|action| match action {
                        Action::Wait { duration_ms } => *duration_ms,
                        _ => unreachable!(),
                    })
                    .collect();
                assert_eq!(result, expected);
            }
        }
    }
    assert!(!move_action(&mut [], 0, 0));
    let mut actions = [Action::Wait { duration_ms: 1 }];
    assert!(!move_action(&mut actions, 0, 1));
    assert!(!move_action(&mut actions, 1, 0));
}

#[test]
fn moving_a_key_does_not_merge_its_press_and_release() {
    let mut actions = [
        Action::Key {
            key: "A".into(),
            press: true,
        },
        Action::Key {
            key: "A".into(),
            press: false,
        },
        Action::Wait { duration_ms: 10 },
    ];
    assert!(move_action(&mut actions, 0, 2));
    assert!(matches!(&actions[0], Action::Key { key, press: false } if key == "A"));
    assert!(matches!(&actions[1], Action::Wait { duration_ms: 10 }));
    assert!(matches!(&actions[2], Action::Key { key, press: true } if key == "A"));
}

#[test]
fn collection_options_keep_ids_even_with_duplicate_names() {
    let collections = ["first", "second"].map(|id| CollectionDTO {
        id: id.into(),
        name: "Juegos".into(),
        enabled: false,
        macros_count: 0,
        order: 0,
        toggle_enabled_hotkey: None,
    });
    let options = collection_options(&collections);
    assert_eq!(options.len(), 2);
    assert_eq!(options[0].value, "first");
    assert_eq!(options[1].value, "second");
    assert_eq!(options[0].label.as_ref(), "Juegos");
    assert!(collection_options(&[]).is_empty());
}

#[test]
fn mode_options_round_trip_without_cycling() {
    let options = mode_options();
    assert_eq!(options.len(), 3);
    for (option, expected) in options.iter().zip([Mode::Once, Mode::Hold, Mode::Toggle]) {
        assert_eq!(selected_mode(Some(&option.value)), Ok(expected));
        assert_eq!(option.label.as_ref(), option.value);
    }
    assert_eq!(
        selected_mode(None).unwrap_err(),
        "Select a mode for the macro."
    );
    assert!(selected_mode(Some("invalid")).is_err());
}
