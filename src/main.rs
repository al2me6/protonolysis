#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc, clippy::module_name_repetitions)]

pub mod numerics;
pub mod peak;

use eframe::egui;
use eframe::egui::plot::{Plot, PlotPoints};
use egui::plot;

use self::peak::{Peak, Splitter};

#[derive(Default)]
struct Protonolysis;

impl Protonolysis {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
}

impl eframe::App for Protonolysis {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Protonolysis");

            let peak = Peak {
                splitters: vec![Splitter { n: 2, j: 5. }, Splitter { n: 3, j: 2. }],
                fwhm: 0.5,
            };

            let waveform = peak.build_waveform();

            let plot = Plot::new("main");
            let line = plot::Line::new(PlotPoints::from_explicit_callback(
                move |x| numerics::evaluate_gaussian_sum(&waveform, x),
                ..,
                1000,
            ));
            plot.show(ui, |plot_ui| {
                plot_ui.line(line);
            })
        });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Protonolysis",
        native_options,
        Box::new(|cc| Box::new(Protonolysis::new(cc))),
    )?;
    Ok(())
}
