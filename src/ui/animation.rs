use std::ops::RangeInclusive;
use std::time::Instant;

use eframe::egui::Ui;

use crate::numerics;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AnimationDirection {
    Forward,
    Reverse,
}

#[derive(Clone, Debug)]
struct AnimationState {
    direction: AnimationDirection,
    initial_t: Instant,
}

#[derive(Clone, Debug)]
pub(super) struct AnimationManager {
    value: f64,
    range: RangeInclusive<f64>,
    duration: f64,
    anim_state: Option<AnimationState>,
}

impl AnimationManager {
    pub(super) fn new(value: f64, range: RangeInclusive<f64>, duration: f64) -> Self {
        let mut ret = Self {
            value,
            range,
            duration,
            anim_state: None,
        };
        ret.set_value_clamping(value);
        ret
    }

    pub(super) fn value(&self) -> f64 {
        self.value
    }

    pub(super) fn range(&self) -> RangeInclusive<f64> {
        self.range.clone()
    }

    fn set_value_inner(&mut self, value: f64) {
        self.value = value.clamp(*self.range.start(), *self.range.end());
    }

    pub(super) fn set_value_clamping(&mut self, value: f64) {
        self.set_value_inner(value);
        self.stop_animating();
    }

    pub(super) fn set_range_clamping(&mut self, range: RangeInclusive<f64>) {
        self.range = range;
        self.set_value_inner(self.value);
    }

    pub(super) fn animate(&mut self) {
        self.anim_state = Some(AnimationState {
            direction: if self.value < *self.range.end() {
                AnimationDirection::Forward
            } else {
                AnimationDirection::Reverse
            },
            initial_t: Instant::now(),
        });
    }

    pub(super) fn stop_animating(&mut self) {
        self.anim_state = None;
    }

    pub(super) fn tick(&mut self, ui: &mut Ui) -> bool {
        let Some(state) = &self.anim_state else {
            return false;
        };

        let dt = state.initial_t.elapsed().as_secs_f64();
        let factor = dt / self.duration;
        let new_normalized = numerics::ease_transition(match state.direction {
            AnimationDirection::Forward => factor,
            AnimationDirection::Reverse => 1.0 - factor,
        });
        let reached_end = !(0.0..=1.0).contains(&new_normalized);
        self.value = new_normalized.clamp(0.0, 1.0) * (self.range.end() - self.range.start()) + self.range.start();
        if reached_end {
            self.anim_state = None;
        } else {
            ui.ctx().request_repaint();
        }
        reached_end
    }
}
