use std::marker::PhantomData;

use super::Peaklet;
use crate::numerics::distribution::distribution_sum::DistributionSum;
use crate::numerics::distribution::RenormalizedDistribution;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SplittingRelationship<'a> {
    pub parent: &'a Peaklet,
    pub children: &'a [Peaklet],
}

#[derive(Clone, PartialEq, Debug)]
/// Splitting patterns resulting from the cumulative contributions of all preceding splitters,
/// starting from the parent singlet.
///
/// _E.g._, s -> q -> qd -> qdd.
pub struct MultipletCascade<D> {
    /// Splitting patterns resulting from contributions of the first n splitters only.
    /// Note that the ordering of peaklets within each stage is meaningful: children of the
    /// same peaklet appear consecutively, and these groups are in the same order as the parent
    /// stage.
    pub(super) stages: Vec<Vec<Peaklet>>,
    /// Full width at half maximum of a single peaklet, in Hz.
    pub(super) fwhm: f64,
    pub(super) _phantom: PhantomData<D>,
}

impl<'a> SplittingRelationship<'a> {
    #[must_use]
    pub fn children_count(&self) -> usize {
        self.children.len()
    }
}

impl<D: RenormalizedDistribution> MultipletCascade<D> {
    #[must_use]
    pub fn base_peaklet(&self) -> Peaklet {
        let base_stage = &self.stages[0];
        assert_eq!(base_stage.len(), 1);
        base_stage[0]
    }

    #[must_use]
    pub fn child_stages_count(&self) -> usize {
        self.stages.len() - 1
    }

    #[must_use]
    pub fn nth_waveform(&self, n: usize, field_strength: f64) -> DistributionSum<D> {
        self.stages[n]
            .iter()
            .map(|peaklet| {
                D::with_fwhm_normalized(
                    super::j_to_ppm(peaklet.δ, field_strength),
                    super::j_to_ppm(self.fwhm, field_strength),
                    peaklet.integration,
                )
            })
            .collect()
    }

    #[must_use]
    pub fn final_waveform(&self, field_strength: f64) -> DistributionSum<D> {
        self.nth_waveform(self.stages.len() - 1, field_strength)
    }

    /// # Panics:
    /// This iterator can only be called on child stages (that is, not the base peaklet).
    pub fn iter_nth_stage(&self, n: usize) -> impl Iterator<Item = SplittingRelationship<'_>> {
        let parent_count = self.stages[n
            .checked_sub(1)
            .expect("should not be called on base stage")]
        .len();
        let children_count = self.stages[n].len();
        assert_eq!(
            children_count % parent_count,
            0,
            "the number of child peaklets should be an integer multiple of the number of parents"
        );
        let group_size = children_count / parent_count;
        self.stages[n]
            .chunks_exact(group_size)
            .enumerate()
            .map(move |(i, group)| SplittingRelationship {
                parent: &self.stages[n - 1][i],
                children: group,
            })
    }

    pub fn max_integration_of_stage(&self, n: usize) -> f64 {
        self.stages[n]
            .iter()
            .map(|peaklet| peaklet.integration)
            .max_by(f64::total_cmp)
            .unwrap()
    }

    #[must_use]
    /// An estimate of whether the splitting _introduced in this stage only_ is visually resolved.
    /// Note that this (intentionally) does not consider whether the peaklet groups (_i.e._, those
    /// contained in a single [`SplittingRelationship`]) in the stage overlap with _each other_.
    pub fn is_stage_resolved(&self, n: usize) -> bool {
        if n == 0 {
            true
        } else {
            // Note that each peaklet group experiences the same splitting, so only check one.
            self.iter_nth_stage(n)
                .next()
                .unwrap()
                .children
                .array_windows()
                .all(|[a, b]| !a.overlaps_with(*b, self.fwhm))
        }
    }
}
