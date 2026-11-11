use super::{ActionMotion, MOVE_DURATION};
use std::time::{Duration, Instant};

fn order(motion: &ActionMotion, count: usize) -> Vec<usize> {
    (0..count)
        .map(|index| motion.original_index(index))
        .collect()
}

#[test]
fn visual_gap_moves_both_ways_without_swapping_intermediate_rows() {
    let now = Instant::now();
    let mut motion = ActionMotion::new(5, now);
    motion.retarget(0, 3, now);
    assert_eq!(order(&motion, 5), [1, 2, 3, 0, 4]);
    assert_eq!(motion.position(1, now), 1.);
    let halfway = motion.position(1, now + MOVE_DURATION / 2);
    assert!(halfway > 0. && halfway < 1.);
    assert_eq!(motion.position(1, now + MOVE_DURATION), 0.);
    assert_eq!(motion.position(4, now + MOVE_DURATION), 4.);
    assert!(!motion.is_animating(now + MOVE_DURATION));
    assert!(motion.tick(now + MOVE_DURATION));
    assert!(!motion.tick(now + MOVE_DURATION + Duration::from_millis(16)));

    let mut motion = ActionMotion::new(5, now);
    motion.retarget(4, 0, now);
    assert_eq!(order(&motion, 5), [4, 0, 1, 2, 3]);
    assert_eq!(motion.position(0, now + MOVE_DURATION), 1.);
    assert_eq!(motion.position(3, now + MOVE_DURATION), 4.);
}

#[test]
fn changing_destination_preserves_current_position_and_unchanged_targets_do_not_restart() {
    let now = Instant::now();
    let mut motion = ActionMotion::new(5, now);
    motion.retarget(0, 3, now);
    let later = now + Duration::from_millis(50);
    let position = motion.position(1, later);
    motion.retarget(0, 0, later);
    assert_eq!(motion.position(1, later), position);
    assert_eq!(order(&motion, 5), [0, 1, 2, 3, 4]);
    motion.retarget(0, 0, later + Duration::from_millis(100));
    assert_eq!(motion.position(1, later + MOVE_DURATION), 1.);
    assert!(!motion.is_animating(later + MOVE_DURATION));
}

#[test]
fn virtual_indices_remain_global_and_single_row_is_stationary() {
    let now = Instant::now();
    let mut motion = ActionMotion::new(500, now);
    motion.retarget(200, 250, now);
    assert_eq!(motion.original_index(199), 199);
    assert_eq!(motion.original_index(200), 201);
    assert_eq!(motion.original_index(250), 200);
    assert_eq!(motion.original_index(251), 251);
    assert_eq!(motion.position(201, now + MOVE_DURATION), 200.);
    let mut motion = ActionMotion::new(1, now);
    motion.retarget(0, 0, now);
    assert!(!motion.is_animating(now));
    assert_eq!(motion.position(0, now), 0.);
}
