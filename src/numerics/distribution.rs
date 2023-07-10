pub mod distribution_sum;
pub mod gaussian;
pub mod lorentzian;

use std::ops::RangeInclusive;

/// A probability density function which has been renormalized by some factor.
pub trait RenormalizedDistribution: Copy + PartialEq {
    fn with_fwhm_normalized(μ: f64, fwhm: f64, normalization: f64) -> Self;
    fn μ(&self) -> f64;
    fn fwhm(&self) -> f64;
    fn normalization(&self) -> f64;
    fn evaluate(&self, x: f64) -> f64;
    fn evaluate_cdf(&self, x: f64) -> f64;
    fn extent_by_fwhm(&self, n: f64) -> RangeInclusive<f64> {
        let μ = self.μ();
        let fwhm = self.fwhm();
        (μ - fwhm * n)..=(μ + fwhm * n)
    }
}
