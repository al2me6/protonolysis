use std::ops::RangeInclusive;

use itertools::Itertools;

use super::gaussian::Gaussian;

#[derive(Clone, PartialEq, Debug)]
pub struct GaussianSum(Vec<Gaussian>);

impl FromIterator<Gaussian> for GaussianSum {
    fn from_iter<T: IntoIterator<Item = Gaussian>>(iter: T) -> Self {
        let mut gaussians = iter.into_iter().collect_vec();
        gaussians.sort_by(|a, b| a.μ.total_cmp(&b.μ));
        Self(gaussians)
    }
}

impl GaussianSum {
    /// Iterate over the individual Gaussians of the sum.
    pub fn components(&self) -> impl Iterator<Item = &Gaussian> {
        self.0.iter()
    }

    #[must_use]
    pub fn evaluate(&self, x: f64) -> f64 {
        self.components().map(|g| g.evaluate(x)).sum()
    }

    #[must_use]
    /// The overall extent of the sum, or the union of the extents of the individual components
    /// (where each extent comprises the interval `σ` standard deviations out from the mean).
    pub fn extent(&self, σ: f64) -> RangeInclusive<f64> {
        self.components()
            .map(|g| (g.μ - g.σ * σ, g.μ + g.σ * σ))
            .reduce(|(l1, r1), (l2, r2)| (l1.min(l2), r1.max(r2)))
            .map_or(0.0..=0.0, |(l, r)| l..=r)
    }

    #[must_use]
    /// Give an _estimate_ of the max value of the sum, by evaluating the sum at the maxima
    /// (i.e., means) of the components.
    pub fn max(&self) -> f64 {
        self.components()
            .map(|g| self.evaluate(g.μ))
            .reduce(f64::max)
            .unwrap_or(0.)
    }
}
