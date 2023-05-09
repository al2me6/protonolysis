use eframe::egui::plot::{self, Plot, PlotPoints};
use eframe::egui::{self, Button, FontData, FontDefinitions, Slider};
use eframe::epaint::FontFamily;

use crate::peak::{self, Peak, Splitter};

pub struct Protonolysis {
    field_strength: f64,
    peak: Peak,
    view_stage: usize,
}

impl Protonolysis {
    #[must_use]
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        const FONT: &str = "Inter";

        let mut fonts = FontDefinitions::empty();
        fonts.font_data.insert(
            FONT.to_owned(),
            FontData::from_static(include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/Inter-Regular.otf"
            ))),
        );
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .push(FONT.to_owned());
        cc.egui_ctx.set_fonts(fonts);

        Self {
            field_strength: 600.,
            peak: Peak {
                splitters: vec![Splitter { n: 2, j: 6.0 }],
                fwhm: 1.,
            },
            view_stage: 1,
        }
    }

    fn try_increment_view_stage(&mut self) -> bool {
        if self.view_stage < self.peak.splitters.len() {
            self.view_stage += 1;
            true
        } else {
            false
        }
    }

    fn clamp_view_stage(&mut self) {
        self.view_stage = self.view_stage.min(self.peak.splitters.len());
    }
}

impl eframe::App for Protonolysis {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let controls = |ui: &mut egui::Ui| {
            ui.heading("Protonolysis");

            ui.separator();

            egui::Grid::new("controls_sliders")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Instrument frequency:");
                    ui.add(
                        Slider::new(&mut self.field_strength, 40.0..=1200.0)
                            .fixed_decimals(0)
                            .step_by(10.)
                            .suffix(" MHz"),
                    );
                    ui.end_row();

                    ui.label("Field strength:");
                    ui.add_enabled(
                        false,
                        egui::DragValue::new(&mut peak::mhz_to_tesla(self.field_strength))
                            .max_decimals(1)
                            .suffix(" T"),
                    );
                    ui.end_row();
                });

            ui.separator();

            ui.label("Configure coupled protons:");
            ui.horizontal(|ui| {
                if ui.button("Add").clicked() {
                    self.peak.splitters.push(Splitter::default());
                    self.try_increment_view_stage();
                }
                if ui.button("Sort by J").clicked() {
                    self.peak.canonicalize();
                }
            });

            // Fixme spread out
            let table = egui_extras::TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::auto());
            // FIXME: auto size.
            table
                .header(20., |mut header| {
                    header.col(|_ui| {});
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
                    while i < self.peak.splitters.len() {
                        let row = |mut row: egui_extras::TableRow| {
                            row.col(|ui| {
                                ui.label(i.to_string());
                            });
                            let splitter = &mut self.peak.splitters[i];
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
                                    self.peak.splitters.swap(i - 1, i);
                                }
                                let is_last = i == self.peak.splitters.len() - 1;
                                if ui
                                    .add_enabled(!is_last, Button::new("↓"))
                                    .on_hover_text("Move down")
                                    .clicked()
                                {
                                    self.peak.splitters.swap(i, i + 1);
                                }
                                // FIXME: positioning of x needs OTF feature `case`.
                                if ui.button("×").on_hover_text("Delete").clicked() {
                                    self.peak.splitters.remove(i);
                                    self.clamp_view_stage();
                                }
                            });
                        };
                        body.row(18.0, row);
                        i += 1;
                    }
                });

            ui.separator();

            egui::Grid::new("controls_sliders")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Peak FWHM:")
                        .on_hover_text("Full width at half maximum (i.e., broadness) of peaks.");
                    ui.add(
                        Slider::new(&mut self.peak.fwhm, 0.1..=4.0)
                            .fixed_decimals(1)
                            .suffix(" Hz"),
                    );
                    ui.end_row();

                    ui.label("Apply splitting up to:");
                    ui.add(Slider::new(
                        &mut self.view_stage,
                        0..=self.peak.splitters.len(),
                    ));
                    ui.end_row();
                });
        };

        egui::SidePanel::right("controls")
            .resizable(false)
            .show(ctx, controls);

        egui::CentralPanel::default().show(ctx, |ui| {
            let waveform = self
                .peak
                .build_multiplet_cascade()
                .nth_waveform(self.view_stage, self.field_strength);
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
