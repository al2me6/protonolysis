mod plotting_utils;
mod splitting_diagram;

use std::collections::HashMap;
use std::sync::LazyLock;

use eframe::egui::plot::{Line, Plot, PlotBounds, PlotPoints, PlotUi};
use eframe::egui::{
    Align, Button, CentralPanel, ComboBox, Context, DragValue, FontData, FontDefinitions,
    FontTweak, Grid, Layout, SidePanel, Slider, TextStyle, Ui,
};
use eframe::epaint::{Color32, FontFamily, Rect, Vec2};
use egui_extras::{Column, TableBuilder};
use maplit::hashmap;

use crate::peak::{self, Peak, Splitter};

pub static PEAK_PRESETS: LazyLock<HashMap<&str, Vec<Splitter>>> = LazyLock::new(|| {
    hashmap! {
        "Et₂O (CH₂)" => vec![Splitter { j: 7., n: 3 }],
        "Et₂O (CH₃)" => vec![Splitter { j: 7., n: 2 }],
    }
});

pub struct Protonolysis {
    field_strength: f64,
    selected_preset: &'static str,
    peak: Peak,
    view_stage: f64,
    show_integral: bool,
    show_splitting_diagram: bool,
    show_peaklets: bool,
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

impl Protonolysis {
    fn try_increment_view_stage(&mut self) {
        self.view_stage += 1.;
        self.clamp_view_stage();
    }

    fn clamp_view_stage(&mut self) {
        self.view_stage = self.view_stage.clamp(0.0, self.peak.splitters.len() as f64);
    }

    fn is_preset_modified(&self) -> bool {
        self.peak.splitters != PEAK_PRESETS[self.selected_preset]
    }

    fn apply_preset(&mut self) {
        self.peak.splitters = PEAK_PRESETS[self.selected_preset].clone();
        self.clamp_view_stage();
    }
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

        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing.y = 5.;
        style.spacing.slider_width = 120.;
        style.spacing.combo_width = 120.;
        cc.egui_ctx.set_style(style);

        let (&selected_preset, splitters) = PEAK_PRESETS.iter().next().unwrap();
        Self {
            field_strength: 600.,
            selected_preset,
            peak: Peak {
                splitters: splitters.clone(),
                fwhm: 1.0,
            },
            view_stage: splitters.len() as f64,
            show_integral: true,
            show_splitting_diagram: true,
            show_peaklets: false,
            linked_x_axis: (-Self::DEFAULT_X, Self::DEFAULT_X),
        }
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
                        .step_by(20.)
                        .suffix(" MHz"),
                );
                ui.end_row();

                ui.label("Field strength:")
                    .on_hover_text("Strength of magnetic field of instrument");
                ui.add_enabled(
                    false,
                    DragValue::new(&mut peak::mhz_to_tesla(self.field_strength))
                        .max_decimals(1)
                        .suffix(" T"),
                );
                ui.end_row();
            });

        ui.separator();

        Grid::new("controls_sliders_peak")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Peak FWHM:")
                    .on_hover_text("Full width at half maximum (i.e., broadness) of peaks");
                ui.add(
                    Slider::new(&mut self.peak.fwhm, 0.5..=4.0)
                        .fixed_decimals(1)
                        .smart_aim(false)
                        .suffix(" Hz"),
                );
                ui.end_row();

                ui.label("Configure coupled protons:");
                ui.end_row();
            });

        ui.indent("controls_splitter", |ui| {
            ui.horizontal(|ui| {
                ui.label("Apply preset:");
                ComboBox::from_id_source("presets_selector")
                    .selected_text(self.selected_preset)
                    .show_ui(ui, |ui| {
                        for &preset in PEAK_PRESETS.keys() {
                            ui.selectable_value(&mut self.selected_preset, preset, preset);
                        }
                    });

                let modified = self.is_preset_modified();
                let apply_button = &ui.add_enabled(
                    modified,
                    Button::new(if modified { "Apply" } else { "Applied" }),
                );
                if apply_button.clicked() {
                    self.apply_preset();
                }
            });

            ui.horizontal(|ui| {
                if ui
                    .add_enabled(self.peak.splitters.len() < 5, Button::new("Add"))
                    .on_hover_text("Add new coupled proton type")
                    .clicked()
                {
                    self.peak.splitters.push(Splitter::default());
                    self.try_increment_view_stage();
                }
                if ui
                    .button("Sort by J")
                    .on_hover_text("Sort by splitting constant in ascending order")
                    .clicked()
                {
                    self.peak.canonicalize();
                }
            });

            let row_height = ui.text_style_height(&TextStyle::Body) + ui.spacing().item_spacing.y;
            let table = TableBuilder::new(ui)
                .striped(true)
                .cell_layout(Layout::left_to_right(Align::Center))
                .column(Column::auto_with_initial_suggestion(20.))
                .columns(Column::auto(), 3)
                .column(Column::remainder())
                .header(row_height, |mut header| {
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
                            ui.style_mut().spacing.slider_width = 80.;
                            ui.add(Slider::new(&mut splitter.n, 1..=10));
                        });
                        row.col(|ui| {
                            ui.add(
                                Slider::new(&mut splitter.j, 0.0..=20.0)
                                    .fixed_decimals(1)
                                    .smart_aim(false),
                            );
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
                    body.row(row_height, row);
                    i += 1;
                }
            });
        });

        ui.separator();

        Grid::new("controls_sliders_view")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Apply splitting up to level:").on_hover_text(
                    "Draw the peak as if only the first n proton types were present. A fractional \
                    value indicates partial application of the last splitting constant.",
                );
                ui.add(
                    Slider::new(
                        &mut self.view_stage,
                        0.0..=(self.peak.splitters.len() as f64),
                    )
                    .custom_formatter(|x, _| {
                        if approx::relative_eq!(x, x.round()) {
                            format!("{x:.0}")
                        } else {
                            format!("{x:.2}")
                        }
                    }),
                );
                ui.end_row();

                ui.label("Show:");
                ui.checkbox(&mut self.show_integral, "Peak integral");
                ui.end_row();

                ui.label("");
                ui.checkbox(&mut self.show_splitting_diagram, "Splitting diagram");
                ui.end_row();

                ui.label("");
                ui.checkbox(&mut self.show_peaklets, "Individual contributions")
                    .on_hover_text(
                        "Draw the individual peaks making up the multiplet to elucidate overlap",
                    );
                ui.end_row();
            });
    }

    fn peak_viewer(&mut self, ui: &mut Ui) {
        let Vec2 {
            x: available_width,
            y: available_height,
        } = ui.available_size();
        let placement_origin = ui.next_widget_position();
        let plot_height = available_height
            - (ui.text_style_height(&TextStyle::Body) + ui.spacing().item_spacing.y) * 2.;

        let waveform = self
            .peak
            .nth_partial_peak(self.view_stage)
            .build_multiplet_cascade()
            .final_waveform(self.field_strength);

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
            plotting_utils::peak_viewer_interactions(plot_ui, &mut self.linked_x_axis);

            let waveform_clone = waveform.clone();
            plot_ui.line(
                Line::new(PlotPoints::from_explicit_callback(
                    move |x| waveform_clone.evaluate(x),
                    ..,
                    Self::SAMPLES,
                ))
                .width(2.)
                .fill(0.),
            );

            if !self.show_peaklets {
                return;
            }
            for &peaklet in waveform.components() {
                plot_ui.line(
                    Line::new(PlotPoints::from_explicit_callback(
                        move |x| peaklet.evaluate(x),
                        peaklet.extent(4.),
                        Self::SAMPLES / 10,
                    ))
                    .color(Color32::LIGHT_BLUE),
                );
            }
        });
        ui.vertical_centered(|ui| ui.label("δ (ppm)"));

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
        let draw_integral_plot = |ui: &mut Ui| {
            integral_plot
                .show(ui, |plot_ui: &mut PlotUi| {
                    let extent = waveform.extent(10.);
                    plot_ui.line(
                        Line::new(PlotPoints::from_explicit_callback(
                            move |x| waveform.evaluate_integral(x),
                            extent,
                            Self::SAMPLES / 2,
                        ))
                        .width(2.)
                        .color(Color32::LIGHT_GREEN),
                    );

                    let bounds = plot_ui.plot_bounds();
                    let (mut bounds_min, mut bounds_max) = (bounds.min(), bounds.max());
                    (bounds_min[0], bounds_max[0]) = self.linked_x_axis;
                    plot_ui.set_plot_bounds(PlotBounds::from_min_max(bounds_min, bounds_max));
                })
                .response
        };
        ui.put(
            Rect::from_min_size(
                placement_origin,
                (available_width, plot_height * 0.2).into(),
            ),
            draw_integral_plot,
        );
    }

    fn splitting_diagram(&self, ui: &mut Ui) {
        let cascade = self.peak.build_multiplet_cascade();

        let plot = Plot::new("splitting_diagram")
            .show_axes([false; 2])
            .show_background(false)
            .show_x(false)
            .show_y(false)
            .allow_boxed_zoom(false)
            .allow_double_click_reset(false)
            .allow_drag(false)
            .allow_scroll(false)
            .allow_zoom(false)
            .auto_bounds_x()
            .auto_bounds_y()
            .height(
                ui.available_height()
                    .min(100. * (self.peak.splitters.len() + 1) as f32),
            )
            .data_aspect(15.);

        plot.show(ui, |plot_ui| {
            splitting_diagram::draw_peaklet_marker(plot_ui, &cascade.base_peaklet(), 0, 1., true);
            for i in 1..=cascade.child_stages_count() {
                let enabled = (i as f64) <= self.view_stage;
                let max_integration = cascade.max_integration_of_stage(i);
                for group in cascade.iter_nth_stage(i) {
                    splitting_diagram::draw_group_children_and_connectors(
                        plot_ui,
                        &group,
                        i,
                        max_integration,
                        enabled,
                    );
                }
            }
        });
    }
}

impl eframe::App for Protonolysis {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        SidePanel::right("controls")
            .min_width(ctx.available_rect().width() * 0.25)
            .resizable(false)
            .show(ctx, |ui| {
                self.controls(ui);
                ui.separator();
                if !self.show_splitting_diagram {
                    return;
                }
                ui.label("Splitting diagram:");
                self.splitting_diagram(ui);
            });

        CentralPanel::default().show(ctx, |ui| {
            self.peak_viewer(ui);
        });
    }
}
