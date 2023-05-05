use std::borrow::Cow;
use std::collections::VecDeque;

use crate::numerics::{self, Gaussian};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Splitter {
    /// Number of chemically equivalent protons.
    pub n: u32,
    /// Splitting constant in Hz.
    pub j: f64,
}

impl Splitter {
    pub const PATTERN_ABBREVIATIONS: [&str; 6] = ["s", "d", "t", "q", "p", "h"];

    #[must_use]
    pub fn resultant_peaklet_count(&self) -> u32 {
        self.n + 1
    }

    #[must_use]
    pub fn name_pattern(&self) -> Option<&'static str> {
        // N.b. indexing: peak count = n + 1, but 0-indexing subtracts 1.
        Self::PATTERN_ABBREVIATIONS.get(self.n as usize).copied()
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Peaklet {
    // Shift relative to center of root peak, in Hz.
    pub δ: f64,
    pub integration: f64,
}

#[derive(Clone, Debug)]
struct ProtoPeaklet<'a> {
    peaklet: Peaklet,
    splitters: &'a [Splitter],
}

#[derive(Clone, PartialEq, Debug)]
pub struct Peak {
    /// List of coupled chemically equivalent proton types.
    pub splitters: Vec<Splitter>,
    /// Full width at half maximum, in Hz.
    pub fwhm: f64,
}

impl Peak {
    #[must_use]
    pub fn name(&self) -> String {
        let splitter_count = self.splitters.len();
        if splitter_count == 0 {
            return Splitter::PATTERN_ABBREVIATIONS[0].to_owned();
        }
        self.splitters
            .iter()
            .map(|splitter| {
                splitter
                    .name_pattern()
                    .map_or_else(|| Cow::from(splitter.n.to_string()), Cow::Borrowed)
            })
            .collect()
    }

    #[must_use]
    pub fn build_peaklets(&self) -> Vec<Peaklet> {
        let mut peaklets = vec![];
        let mut queue = VecDeque::new();
        queue.push_back(ProtoPeaklet {
            peaklet: Peaklet {
                integration: 1.,
                δ: 0.,
            },
            splitters: &self.splitters,
        });
        while !queue.is_empty() {
            let ProtoPeaklet { peaklet, splitters } = queue.pop_front().unwrap();
            let Some((splitter, child_splitters)) = splitters.split_first() else {
                peaklets.push(peaklet);
                continue;
            };
            let peak_count = splitter.resultant_peaklet_count();
            let mut δ = peaklet.δ - f64::from(peak_count - 1) * splitter.j / 2.;
            for a in numerics::normalized_pascals_triangle(peak_count - 1) {
                let child_peaklet = Peaklet {
                    δ,
                    integration: peaklet.integration * a,
                };
                δ += splitter.j;
                queue.push_back(ProtoPeaklet {
                    peaklet: child_peaklet,
                    splitters: child_splitters,
                });
            }
        }
        peaklets.sort_by(|a, b| a.δ.total_cmp(&b.δ));
        peaklets
    }

    #[must_use]
    pub fn build_waveform(&self) -> Vec<Gaussian> {
        self.build_peaklets()
            .into_iter()
            .map(|peaklet| Gaussian::with_fwhm(self.fwhm, peaklet.δ, peaklet.integration))
            .collect()
    }
}
