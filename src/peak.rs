use std::borrow::Cow;
use std::collections::VecDeque;

use crate::numerics::{self, Gaussian, GaussianSum};

#[must_use]
#[allow(clippy::doc_markdown)]
/// Convert an NMR instrument frequency (MHz) to the corresponding magnetic field strength (T).
pub fn mhz_to_tesla(frequency: f64) -> f64 {
    #[allow(non_upper_case_globals)]
    const γ_PROTON: f64 = 42.577_478_518; // MHz/T
    frequency / γ_PROTON
}

#[must_use]
#[allow(clippy::doc_markdown)]
/// Convert an absolute shift in Hz to ppm at a given field strength in (MHz).
pub fn j_to_ppm(j: f64, frequency: f64) -> f64 {
    j / frequency
}

#[derive(Clone, Copy, PartialEq, Debug)]
/// A single type of proton coupled to a [`Peak`].
pub struct Splitter {
    /// Number of chemically equivalent protons.
    pub n: u32,
    /// Coupling constant in Hz.
    pub j: f64,
}

#[derive(Clone, Copy, PartialEq, Debug)]
/// An atomic component of a multiplet peak.
pub struct Peaklet {
    /// Shift relative to the center of the root peak, in Hz.
    pub δ: f64,
    /// The fraction of the whole multiplet contained herein.
    pub integration: f64,
}

#[derive(Clone, PartialEq, Debug)]
/// Splitting patterns resulting from the cumulative contributions of all preceding splitters,
/// starting from the parent singlet.
///
/// _E.g._, s -> q -> qd -> qdd.
pub struct MultipletCascade {
    /// Splitting patterns resulting from contributions of the first n splitters only.
    stages: Vec<Vec<Peaklet>>,
    /// Full width at half maximum of a single peaklet, in Hz.
    fwhm: f64,
}

#[derive(Clone, PartialEq, Debug)]
/// A descriptor of a peak corresponding to a single proton type coupled to arbitrary [`Splitter`]s.
pub struct Peak {
    /// List of coupled proton types.
    pub splitters: Vec<Splitter>,
    /// Full width at half maximum of the peak, in Hz.
    pub fwhm: f64,
}

impl Default for Splitter {
    fn default() -> Self {
        Self { n: 1, j: 5.0 }
    }
}

impl Splitter {
    pub const PATTERN_ABBREVIATIONS: [&str; 6] = ["s", "d", "t", "q", "p", "h"];

    #[must_use]
    pub fn resultant_peaklet_count(&self) -> u32 {
        self.n + 1
    }

    #[must_use]
    pub fn name_pattern(&self) -> Cow<'static, str> {
        // N.b. indexing: peak count = n + 1, but 0-indexing subtracts 1.
        Self::PATTERN_ABBREVIATIONS
            .get(self.n as usize)
            .copied()
            .map_or_else(
                || self.resultant_peaklet_count().to_string().into(),
                Cow::Borrowed,
            )
    }
}

impl Peaklet {
    pub const PARENT_SINGLET: Peaklet = Self {
        δ: 0.,
        integration: 1.,
    };
}

impl MultipletCascade {
    #[must_use]
    pub fn len(&self) -> usize {
        self.stages.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &[Peaklet]> {
        self.stages.iter().map(AsRef::as_ref)
    }

    #[must_use]
    pub fn nth_waveform(&self, n: usize, field_strength: f64) -> GaussianSum {
        self.stages[n]
            .iter()
            .map(|peaklet| {
                Gaussian::with_fwhm(
                    j_to_ppm(self.fwhm, field_strength),
                    j_to_ppm(peaklet.δ, field_strength),
                    peaklet.integration,
                )
            })
            .collect()
    }

    #[must_use]
    pub fn final_waveform(&self, field_strength: f64) -> GaussianSum {
        self.nth_waveform(self.len() - 1, field_strength)
    }
}

impl Default for Peak {
    fn default() -> Self {
        Self {
            splitters: vec![],
            fwhm: 1.,
        }
    }
}

impl Peak {
    #[must_use]
    pub fn name(&self) -> String {
        let splitter_count = self.splitters.len();
        if splitter_count == 0 {
            return Splitter::PATTERN_ABBREVIATIONS[0].to_owned();
        }
        self.splitters.iter().map(Splitter::name_pattern).collect()
    }

    #[must_use]
    pub fn build_multiplet_cascade(&self) -> MultipletCascade {
        let mut cascade = MultipletCascade {
            stages: itertools::repeat_n(vec![], self.splitters.len() + 1).collect(),
            fwhm: self.fwhm,
        };

        let mut queue: VecDeque<(Peaklet, &[Splitter])> = VecDeque::new();
        queue.push_back((Peaklet::PARENT_SINGLET, &self.splitters));

        while let Some((peaklet, splitters)) = queue.pop_front() {
            let peaklet_stage = self.splitters.len() - splitters.len();
            cascade.stages[peaklet_stage].push(peaklet);

            let [splitter, child_splitters @ ..] = splitters else {
                continue;
            };

            let peak_count = splitter.resultant_peaklet_count();
            let mut δ = peaklet.δ - f64::from(peak_count - 1) * splitter.j / 2.;
            for a in numerics::normalized_pascals_triangle(splitter.n) {
                let child_peaklet = Peaklet {
                    δ,
                    integration: peaklet.integration * a,
                };
                δ += splitter.j;
                queue.push_back((child_peaklet, child_splitters));
            }
        }

        cascade
    }
}
