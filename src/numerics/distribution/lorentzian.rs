use std::f64::consts::FRAC_1_PI;

use super::RenormalizedDistribution;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lorentzian {
    x0: f64,
    γ: f64,
    normalize: f64,
}

impl RenormalizedDistribution for Lorentzian {
    fn with_fwhm_normalized(μ: f64, fwhm: f64, normalize: f64) -> Self {
        Self {
            x0: μ,
            γ: fwhm / 2.,
            normalize,
        }
    }

    fn μ(&self) -> f64 {
        self.x0
    }

    fn fwhm(&self) -> f64 {
        self.γ * 2.
    }

    fn evaluate(&self, x: f64) -> f64 {
        FRAC_1_PI * self.γ / ((x - self.x0) * (x - self.x0) + self.γ * self.γ) * self.normalize
    }

    fn evaluate_cdf(&self, x: f64) -> f64 {
        (FRAC_1_PI * ((x - self.x0) / self.γ).atan() + 0.5) * self.normalize
    }
}
