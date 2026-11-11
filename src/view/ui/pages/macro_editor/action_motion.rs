//! Temporary visual order and interruptible row motion; never edits the draft.
use std::time::{Duration, Instant};

const MOVE_DURATION: Duration = Duration::from_millis(150);

pub(super) struct ActionMotion {
    order: Vec<usize>,
    starts: Vec<f32>,
    targets: Vec<f32>,
    started: Instant,
}

impl ActionMotion {
    pub fn new(count: usize, now: Instant) -> Self {
        Self {
            order: (0..count).collect(),
            starts: (0..count).map(|index| index as f32).collect(),
            targets: (0..count).map(|index| index as f32).collect(),
            started: now,
        }
    }

    pub fn original_index(&self, visual_index: usize) -> usize {
        self.order[visual_index]
    }

    pub fn position(&self, original_index: usize, now: Instant) -> f32 {
        let progress = (now.saturating_duration_since(self.started).as_secs_f32()
            / MOVE_DURATION.as_secs_f32())
        .clamp(0., 1.);
        let eased = 1. - (1. - progress).powi(3);
        let start = self.starts[original_index];
        start + (self.targets[original_index] - start) * eased
    }

    pub fn is_animating(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.started) < MOVE_DURATION && self.starts != self.targets
    }

    /// Request the final exact frame too, not just frames before the deadline.
    pub fn tick(&mut self, now: Instant) -> bool {
        let changed = self.starts != self.targets;
        if changed && !self.is_animating(now) {
            self.starts.clone_from(&self.targets);
        }
        changed
    }

    pub fn retarget(&mut self, from: usize, to: usize, now: Instant) {
        let mut order: Vec<_> = (0..self.order.len()).collect();
        let source = order.remove(from);
        order.insert(to, source);
        if order == self.order {
            return;
        }
        // Sample the ongoing transition, rather than jumping back to its start
        // when the pointer changes destination before the 150 ms have elapsed.
        self.starts = (0..order.len())
            .map(|index| self.position(index, now))
            .collect();
        for (visual, &original) in order.iter().enumerate() {
            self.targets[original] = visual as f32;
        }
        self.order = order;
        self.started = now;
    }
}

#[cfg(test)]
#[path = "../../../../../tests/unit/view/ui/action_motion.rs"]
mod tests;
