use eframe::egui::plot::{Line, Plot, PlotBounds, PlotPoints, PlotUi};
use eframe::egui::{
    Align, Button, CentralPanel, Context, CursorIcon, DragValue, FontData, FontDefinitions,
    FontTweak, Grid, Layout, SidePanel, Slider, TextStyle, Ui,
};
use eframe::epaint::{Color32, FontFamily, Rect, Vec2};
use egui_extras::{Column, TableBuilder};

use crate::peak::{self, Peak, Splitter};

pub struct Protonolysis {
    field_strength: f64,
    peak: Peak,
    view_stage: usize,
    show_integral: bool,
    linked_x_axis: (f64, f64),
}

macro_rules! load_font {
    ($name:literal) => {
        FontData::from_static(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/",
            $name
        )))
    };
}

fn peak_viewer_plot(plot_ui: &mut PlotUi, line: Line, x_axis: &mut (f64, f64)) {
    plot_ui.line(line);

    if !plot_ui.plot_hovered() {
        return;
    }

    // FIXME: touch support.

    // Custom pan:
    let drag = plot_ui
        .ctx()
        .input(|i| i.pointer.primary_down().then(|| i.pointer.delta()));
    if let Some(drag) = drag {
        plot_ui.ctx().set_cursor_icon(CursorIcon::ResizeHorizontal);
        plot_ui.translate_bounds(Vec2 { x: -drag.x, y: 0. });
    }

    // Custom zoom:
    let bounds = plot_ui.plot_bounds();
    let mut bounds_min = bounds.min();
    let mut bounds_max = bounds.max();
    // y: zoom:
    let scroll_y = plot_ui.ctx().input(|i| f64::from(i.scroll_delta.y));
    if scroll_y != 0. {
        let zoom_factor = (scroll_y / 200.).exp();
        bounds_min[1] /= zoom_factor;
        bounds_max[1] /= zoom_factor;
    }
    // x zoom (ctrl+scroll):
    // This seems to eat the raw scroll delta.
    let ctrl_scroll_factor = plot_ui.ctx().input(|i| f64::from(i.zoom_delta()));
    bounds_min[0] /= ctrl_scroll_factor;
    bounds_max[0] /= ctrl_scroll_factor;
    // Apply:
    *x_axis = (bounds_min[0], bounds_max[0]);
    plot_ui.set_plot_bounds(PlotBounds::from_min_max(bounds_min, bounds_max));
}

impl Protonolysis {
    const DEFAULT_X: f64 = 0.15;
    const DEFAULT_Y: f64 = 300.;
    const SAMPLES: usize = 5000;

    #[must_use]
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = FontDefinitions::empty();
        fonts
            .font_data
            .insert("Inter".to_owned(), load_font!("Inter-Regular.otf"));
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .push("Inter".to_owned());
        fonts.font_data.insert(
            "SourceCodePro".to_owned(),
            load_font!("SourceCodePro-Regular.ttf").tweak(FontTweak {
                scale: 1.1,
                y_offset_factor: -0.25,
                y_offset: 0.,
            }),
        );
        fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .push("SourceCodePro".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        Self {
            field_strength: 600.,
            peak: Peak {
                splitters: vec![Splitter { n: 2, j: 6.0 }],
                fwhm: 1.,
            },
            view_stage: 1,
            show_integral: true,
            linked_x_axis: (-Self::DEFAULT_X, Self::DEFAULT_X),
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

    fn controls(&mut self, ui: &mut Ui) {
        ui.heading("Protonolysis");

        ui.separator();

        Grid::new("controls_sliders_instrument")
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
                    DragValue::new(&mut peak::mhz_to_tesla(self.field_strength))
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

        let table = TableBuilder::new(ui)
            .striped(true)
            .cell_layout(Layout::left_to_right(Align::Center))
            .column(Column::auto_with_initial_suggestion(20.))
            .columns(Column::auto(), 3)
            .column(Column::remainder())
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
            });
        table.body(|mut body| {
            let mut i = 0;
            while i < self.peak.splitters.len() {
                let row = |mut row: egui_extras::TableRow| {
                    row.col(|ui| {
                        ui.label((i + 1).to_string());
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
                        let mut button = |enabled, text, hover| {
                            ui.add_enabled(enabled, Button::new(text))
                                .on_hover_text(hover)
                                .clicked()
                        };
                        if button(i > 0, "↑", "Move up") {
                            self.peak.splitters.swap(i - 1, i);
                        }
                        if button(i < self.peak.splitters.len() - 1, "↓", "Move down") {
                            self.peak.splitters.swap(i, i + 1);
                        }
                        // U+2717 BALLOT X.
                        if button(true, "\u{2717}", "Delete") {
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

        Grid::new("controls_sliders_view")
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

                ui.label("");
                ui.checkbox(&mut self.show_integral, "Show peak integral");
                ui.end_row();
            });
    }

    fn peak_viewer(&mut self, ui: &mut Ui) {
        let line_height = ui.text_style_height(&TextStyle::Body);
        let Vec2 {
            x: available_width,
            y: available_height,
        } = ui.available_size();
        let plot_height = available_height - line_height;

        let waveform = self
            .peak
            .build_multiplet_cascade()
            .nth_waveform(self.view_stage, self.field_strength);
        let waveform_clone = waveform.clone();

        let peak_plot = Plot::new("peak_plot")
            .include_x(-Self::DEFAULT_X)
            .include_x(Self::DEFAULT_X)
            .include_y(Self::DEFAULT_Y * -0.05)
            .include_y(Self::DEFAULT_Y * 1.1)
            .show_x(false)
            .show_y(false)
            .allow_drag(false)
            .allow_boxed_zoom(false)
            .allow_scroll(false)
            .allow_zoom(false)
            .height(plot_height);
        peak_plot.show(ui, |plot_ui| {
            peak_viewer_plot(
                plot_ui,
                Line::new(PlotPoints::from_explicit_callback(
                    move |x| waveform.evaluate(x),
                    ..,
                    Self::SAMPLES,
                ))
                .width(2.)
                .fill(0.),
                &mut self.linked_x_axis,
            );
        });

        // Interaction info.
        ui.horizontal(|ui| {
            ui.label("Controls:");
            ui.code("drag");
            ui.label("to pan,");
            ui.code("scroll");
            ui.label("to zoom vertically,");
            ui.code("ctrl+scroll");
            ui.label("to zoom horizontally.");
        });

        if !self.show_integral {
            return;
        }

        let integral_plot = Plot::new("integral_plot")
            .include_x(-Self::DEFAULT_X)
            .include_x(Self::DEFAULT_X)
            .include_y(-0.1)
            .include_y(1.5)
            .show_axes([false; 2])
            .show_background(false)
            .show_x(false)
            .show_y(false)
            .allow_boxed_zoom(false)
            .allow_double_click_reset(false)
            .allow_drag(false)
            .allow_scroll(false)
            .allow_zoom(false);
        let extent = waveform_clone.extent(10.);
        let draw_integral_plot = |ui: &mut Ui| {
            integral_plot
                .show(ui, |plot_ui: &mut PlotUi| {
                    plot_ui.line(
                        Line::new(PlotPoints::from_explicit_callback(
                            move |x| waveform_clone.evaluate_integral(x),
                            extent,
                            Self::SAMPLES,
                        ))
                        .width(2.)
                        .color(Color32::GREEN),
                    );

                    let bounds = plot_ui.plot_bounds();
                    let (mut bounds_min, mut bounds_max) = (bounds.min(), bounds.max());
                    (bounds_min[0], bounds_max[0]) = self.linked_x_axis;
                    plot_ui.set_plot_bounds(PlotBounds::from_min_max(bounds_min, bounds_max));
                })
                .response
        };
        ui.put(
            Rect::from_x_y_ranges(0.0..=available_width, 0.0..=(plot_height * 0.2)),
            draw_integral_plot,
        );
    }
}

impl eframe::App for Protonolysis {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        SidePanel::right("controls")
            .resizable(false)
            .show(ctx, |ui| self.controls(ui));

        CentralPanel::default().show(ctx, |ui| {
            self.peak_viewer(ui);
        });
    }
}
