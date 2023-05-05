#![warn(clippy::pedantic)]
#![allow(
    clippy::len_without_is_empty,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions
)]

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
                splitters: vec![
                    Splitter { n: 3, j: 8. },
                    Splitter { n: 2, j: 2. },
                    Splitter { n: 2, j: 1. },
                ],
                fwhm: 0.75,
            };

            let waveform = peak.build_multiplet_cascade().nth_waveform(3);
            let extent = waveform.extent(10.0);

            let plot = Plot::new("main")
                .include_y(waveform.max())
                .auto_bounds_x()
                .auto_bounds_y();
            let line = plot::Line::new(PlotPoints::from_explicit_callback(
                move |x| waveform.evaluate(x),
                extent,
                2500,
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
