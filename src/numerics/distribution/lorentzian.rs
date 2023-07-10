use std::f64::consts::FRAC_1_PI;

use super::RenormalizedDistribution;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lorentzian {
    pub x0: f64,
    pub γ: f64,
    pub normalization: f64,
}

impl RenormalizedDistribution for Lorentzian {
    fn with_fwhm_normalized(μ: f64, fwhm: f64, normalization: f64) -> Self {
        Self {
            x0: μ,
            γ: fwhm / 2.,
            normalization,
        }
    }

    fn μ(&self) -> f64 {
        self.x0
    }

    fn fwhm(&self) -> f64 {
        self.γ * 2.
    }

    fn normalization(&self) -> f64 {
        self.normalization
    }

    fn evaluate(&self, x: f64) -> f64 {
        FRAC_1_PI * self.γ / ((x - self.x0) * (x - self.x0) + self.γ * self.γ) * self.normalization
    }

    fn evaluate_cdf(&self, x: f64) -> f64 {
        (FRAC_1_PI * ((x - self.x0) / self.γ).atan() + 0.5) * self.normalization
    }
}
