use std::ops::RangeInclusive;

use itertools::Itertools;

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
        const FWHM_FOR_σ: f64 = 2.354_820_045;
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

pub fn pascals_triangle(n: u32) -> impl Iterator<Item = u32> {
    let mut prev = 1;
    (0..=n).map(move |k| {
        if k > 0 {
            prev = prev * (n + 1 - k) / k;
        }
        prev
    })
}

/// The `n`-th row of Pascal's triangle, with the sum of the row normalized to unity.
pub fn normalized_pascals_triangle(n: u32) -> impl Iterator<Item = f64> {
    // Sum of (0-indexed) n-th row is 2^n.
    let row_sum = f64::from(1 << n);
    pascals_triangle(n).map(move |a| f64::from(a) / row_sum)
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    #[test]
    fn pascals_triangle() {
        const EXPECTED: [&[u32]; 7] = [
            &[1],
            &[1, 1],
            &[1, 2, 1],
            &[1, 3, 3, 1],
            &[1, 4, 6, 4, 1],
            &[1, 5, 10, 10, 5, 1],
            &[1, 6, 15, 20, 15, 6, 1],
        ];
        for (n, &expected) in EXPECTED.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            let computed = super::pascals_triangle(n as _).collect_vec();
            assert!(computed == expected);
        }
    }

    #[test]
    fn normalized_pascals_triangle() {
        for n in 0..=6 {
            let sum = super::normalized_pascals_triangle(n).sum::<f64>();
            approx::assert_abs_diff_eq!(sum, 1.);
        }
    }
}
