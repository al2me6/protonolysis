use std::f64::consts::FRAC_1_SQRT_2;

use super::RenormalizedDistribution;
use crate::numerics::error_function::erfc;

pub const FRAC_1_SQRT_2PI: f64 = 0.398_942_280_4;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Gaussian {
    pub μ: f64,
    pub σ: f64,
    pub normalize: f64,
}

impl Gaussian {
    #[allow(non_upper_case_globals)]
    /// 2 sqrt(2 ln 2). <https://mathworld.wolfram.com/GaussianFunction.html>, eqn. 8.
    const FWHM_FOR_σ: f64 = 2.354_820_045;
}

impl RenormalizedDistribution for Gaussian {
    fn with_fwhm_normalized(μ: f64, fwhm: f64, normalize: f64) -> Self {
        Self {
            μ,
            σ: fwhm / Self::FWHM_FOR_σ,
            normalize,
        }
    }

    fn μ(&self) -> f64 {
        self.μ
    }

    fn fwhm(&self) -> f64 {
        self.σ * Self::FWHM_FOR_σ
    }

    #[inline]
    fn evaluate(&self, x: f64) -> f64 {
        let σ_inv = self.σ.recip();
        self.normalize
            * FRAC_1_SQRT_2PI
            * σ_inv
            * (-0.5 * σ_inv * σ_inv * (x - self.μ) * (x - self.μ)).exp()
    }

    #[inline]
    fn evaluate_cdf(&self, x: f64) -> f64 {
        0.5 * erfc(-(x - self.μ) / self.σ * FRAC_1_SQRT_2) * self.normalize
    }
}
