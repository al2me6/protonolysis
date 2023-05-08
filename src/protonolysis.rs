use eframe::egui::plot::{self, Plot, PlotPoints};
use eframe::egui::{self, Button, Slider};

use crate::peak::{self, Peak, Splitter};

pub struct Protonolysis {
    field_strength: f64,
    peak: Peak,
}

impl Protonolysis {
    #[must_use]
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            field_strength: 600.,
            peak: Peak {
                splitters: vec![Splitter { n: 2, j: 6.0 }],
                fwhm: 1.,
            },
        }
    }
}

impl eframe::App for Protonolysis {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            field_strength,
            peak,
        } = self;

        let controls = |ui: &mut egui::Ui| {
            ui.heading("Protonolysis");

            egui::Grid::new("controls_sliders")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Instrument frequency:");
                    ui.add(
                        Slider::new(field_strength, 40.0..=1200.0)
                            .fixed_decimals(0)
                            .step_by(10.)
                            .suffix(" MHz"),
                    );
                    ui.end_row();

                    ui.label("Field strength:");
                    ui.add_enabled(
                        false,
                        egui::DragValue::new(&mut peak::mhz_to_tesla(*field_strength)).suffix(" T"),
                    );
                    ui.end_row();
                });

            // Fixme spread out
            let table = egui_extras::TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::auto());
            // FIXME: auto size.
            table
                .header(20., |mut header| {
                    header.col(|ui| {
                        ui.strong("Count");
                    });
                    header.col(|ui| {
                        ui.strong("J (Hz)");
                    });
                    header.col(|ui| {
                        ui.strong("Pattern");
                    });
                    header.col(|ui| {
                        ui.strong("Action");
                    });
                })
                .body(|mut body| {
                    let mut i = 0;
                    while i < peak.splitters.len() {
                        let row = |mut row: egui_extras::TableRow| {
                            let splitter = &mut peak.splitters[i];
                            row.col(|ui| {
                                ui.add(Slider::new(&mut splitter.n, 1..=10));
                            });
                            row.col(|ui| {
                                ui.add(Slider::new(&mut splitter.j, 0.0..=20.0).fixed_decimals(1));
                            });
                            row.col(|ui| {
                                ui.label(splitter.name_pattern());
                            });
                            row.col(|ui| {
                                let is_first = i == 0;
                                if ui
                                    .add_enabled(!is_first, Button::new("↑"))
                                    .on_hover_text("Move up")
                                    .clicked()
                                {
                                    peak.splitters.swap(i - 1, i);
                                }
                                let is_last = i == peak.splitters.len() - 1;
                                if ui
                                    .add_enabled(!is_last, Button::new("↓"))
                                    .on_hover_text("Move down")
                                    .clicked()
                                {
                                    peak.splitters.swap(i, i + 1);
                                }
                                if ui.button("×").on_hover_text("Delete").clicked() {
                                    peak.splitters.remove(i);
                                }
                            });
                        };
                        body.row(18.0, row);
                        i += 1;
                    }
                });

            if ui.button("Add splitter").clicked() {
                peak.splitters.push(Splitter::default());
            }
        };

        egui::SidePanel::right("controls")
            .min_width(350.)
            .resizable(false)
            .show(ctx, controls);

        egui::CentralPanel::default().show(ctx, |ui| {
            let waveform = peak
                .build_multiplet_cascade()
                .final_waveform(*field_strength);
            let extent = waveform.extent(10.0);

            let plot = Plot::new("main")
                .include_x(-0.5)
                .include_x(0.5)
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
