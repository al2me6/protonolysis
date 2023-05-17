use std::ops::{Deref, RangeInclusive};

use eframe::egui::Ui;

use crate::numerics;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AnimationDirection {
    Forward,
    Reverse,
}

#[derive(Clone, Debug)]
pub(super) struct CyclicallyAnimatedF64 {
    value: f64,
    range: (f64, f64),
    duration: f64,
    direction: AnimationDirection,
    anim_factor: Option<f64>,
}

impl AnimationDirection {
    fn flip(&mut self) {
        *self = match self {
            Self::Forward => Self::Reverse,
            Self::Reverse => Self::Forward,
        }
    }
}

impl Deref for CyclicallyAnimatedF64 {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl CyclicallyAnimatedF64 {
    pub(super) fn new(value: f64, range: RangeInclusive<f64>, duration: f64) -> Self {
        let mut ret = Self {
            value,
            range: range.into_inner(),
            duration,
            direction: AnimationDirection::Forward,
            anim_factor: None,
        };
        ret.set_value_clamping(value);
        ret
    }

    pub(super) fn range(&self) -> RangeInclusive<f64> {
        self.range.0..=self.range.1
    }

    fn set_value_inner(&mut self, value: f64) {
        self.value = value.clamp(self.range.0, self.range.1);
    }

    pub(super) fn set_value_clamping(&mut self, value: f64) {
        self.set_value_inner(value);
        self.stop_animating();
    }

    pub(super) fn set_range_clamping(&mut self, range: RangeInclusive<f64>) {
        self.range = range.into_inner();
        self.set_value_inner(self.value);
    }

    pub(super) fn start_animating(&mut self) {
        if self.anim_factor.is_none() {
            self.anim_factor = Some(numerics::ease_transition_inverse(
                (self.value - self.range.0) / (self.range.1 - self.range.0),
            ));
        }
    }

    pub(super) fn stop_animating(&mut self) {
        self.anim_factor = None;
    }

    pub(super) fn is_animating(&self) -> bool {
        self.anim_factor.is_some()
    }

    pub(super) fn toggle_animation(&mut self) {
        if self.is_animating() {
            self.stop_animating();
        } else {
            self.start_animating();
        }
    }

    pub(super) fn tick(&mut self, ui: &mut Ui) {
        let Some(factor) = &mut self.anim_factor else {
            return;
        };

        let dt = ui.ctx().input(|i| f64::from(i.stable_dt)).min(0.1)
            * match self.direction {
                AnimationDirection::Forward => 1.0,
                AnimationDirection::Reverse => -1.0,
            };
        *factor += dt / self.duration;

        let new_normalized = numerics::ease_transition(factor.clamp(0.0, 1.0));
        self.value = new_normalized * (self.range.1 - self.range.0) + self.range.0;

        let reached_end = !(0.0..=1.0).contains(factor);
        if reached_end {
            self.direction.flip();
        }

        ui.ctx().request_repaint();
    }
}
