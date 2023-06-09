mod multiplet_cascade;

use std::borrow::Cow;
use std::collections::VecDeque;

pub use self::multiplet_cascade::{MultipletCascade, SplittingRelationship};
use crate::numerics;

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

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct FractionalStageIndex(f64);

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
    pub const PATTERN_ABBREVIATIONS: [&str; 7] = ["s", "d", "t", "q", "p", "h", "hept"];
    pub const PATTERN_NAMES: [&str; 7] = [
        "singlet", "doublet", "triplet", "quartet", "pentet", "hextet", "heptet",
    ];

    #[must_use]
    pub fn resultant_peaklet_count(&self) -> u32 {
        self.n + 1
    }

    #[must_use]
    pub fn abbreviate_pattern(&self) -> Cow<'static, str> {
        // N.b. indexing: peak count = n + 1, but 0-indexing subtracts 1.
        Self::PATTERN_ABBREVIATIONS
            .get(self.n as usize)
            .copied()
            .map_or_else(
                || self.resultant_peaklet_count().to_string().into(),
                Cow::Borrowed,
            )
    }

    #[must_use]
    pub fn name_pattern(&self) -> Option<&'static str> {
        Self::PATTERN_NAMES.get(self.n as usize).copied()
    }

    pub fn peak_ratios(&self) -> impl Iterator<Item = u32> {
        numerics::pascals_triangle(self.n)
    }
}

impl Peaklet {
    pub const PARENT_SINGLET: Peaklet = Self {
        δ: 0.,
        integration: 1.,
    };

    #[must_use]
    pub fn overlaps_with(&self, b: Peaklet, fwhm: f64) -> bool {
        (self.δ - b.δ).abs() < fwhm * 1.1
    }
}

impl FractionalStageIndex {
    #[must_use]
    pub fn new(index: f64) -> FractionalStageIndex {
        assert!(index >= 0.0);
        Self(index)
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    #[must_use]
    pub fn full(&self) -> usize {
        self.0.floor() as usize
    }

    #[must_use]
    pub fn has_significant_partial(&self) -> bool {
        approx::abs_diff_ne!(self.0.fract(), 0.0)
    }

    #[must_use]
    pub fn partial_and_index(&self) -> Option<(usize, f64)> {
        if self.has_significant_partial() {
            Some((self.full() + 1, self.0.fract()))
        } else {
            None
        }
    }

    #[must_use]
    pub fn total_stage_count(&self) -> usize {
        self.full() + usize::from(self.has_significant_partial())
    }
}

impl Default for Peak {
    fn default() -> Self {
        Self {
            splitters: vec![],
            fwhm: 0.5,
        }
    }
}

impl Peak {
    pub fn total_peaklet_count(&self) -> u32 {
        self.splitters
            .iter()
            .map(Splitter::resultant_peaklet_count)
            .product()
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn stage_count(&self) -> u32 {
        self.splitters.len() as u32 + 1
    }

    #[must_use]
    pub fn name(&self) -> Option<String> {
        let splitter_count = self.splitters.len();
        if splitter_count == 0 {
            return Some(Splitter::PATTERN_ABBREVIATIONS[0].to_owned());
        }
        self.splitters
            .iter()
            .map(Splitter::abbreviate_pattern)
            .map(|name| (name.len() == 1 && matches!(name, Cow::Borrowed(_))).then_some(name))
            .collect()
    }

    pub fn sort_by_j(&mut self) {
        self.splitters.sort_by(|a, b| b.j.total_cmp(&a.j));
    }

    #[must_use]
    pub fn nth_partial_peak(&self, n: FractionalStageIndex) -> Self {
        let mut clone = self.clone();
        clone.splitters.truncate(n.total_stage_count());
        if let Some((idx, part)) = n.partial_and_index() {
            // Note that the splitters do not contain the base stage.
            clone.splitters[idx - 1].j *= part;
        }
        clone
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
