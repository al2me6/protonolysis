pub const FRAC_1_SQRT_2PI: f64 = 0.398_942_280_4;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Gaussian {
    pub μ: f64,
    pub σ: f64,
    pub normalize: f64,
}

impl Gaussian {
    #[must_use]
    pub fn with_fwhm(fwhm: f64, μ: f64, normalize: f64) -> Self {
        #[allow(non_upper_case_globals)]
        /// 2 sqrt(2 ln 2). <https://mathworld.wolfram.com/GaussianFunction.html>, eqn. 8.
        pub(crate) const FWHM_FOR_σ: f64 = 2.354_820_045;
        Gaussian {
            μ,
            σ: fwhm / FWHM_FOR_σ,
            normalize,
        }
    }

    #[must_use]
    #[inline]
    pub fn evaluate(&self, x: f64) -> f64 {
        let σ_inv = self.σ.recip();
        self.normalize
            * FRAC_1_SQRT_2PI
            * σ_inv
            * (-0.5 * σ_inv * σ_inv * (x - self.μ) * (x - self.μ)).exp()
    }
}
