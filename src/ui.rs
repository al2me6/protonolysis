mod animation;
mod splitting_diagram;
pub mod utils;

use std::collections::HashMap;
use std::sync::LazyLock;

use eframe::egui::plot::{Line, PlotBounds, PlotPoints, PlotUi};
use eframe::egui::{
    self, Align, Button, CentralPanel, ComboBox, Context, DragValue, FontData, FontDefinitions,
    FontTweak, Layout, RichText, ScrollArea, SidePanel, Slider, TextStyle, Ui,
};
use eframe::epaint::{Color32, FontFamily, Rect, Vec2};
use egui_extras::{Column, TableBuilder};
use itertools::Itertools;
use maplit::hashmap;

use self::animation::CyclicallyAnimatedF64;
use crate::peak::{self, FractionalStageIndex, MultipletCascade, Peak, Splitter};
use crate::utils::StoreOnNthCall;

macro_rules! load_font {
    ($name:literal) => {
        FontData::from_static(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/",
            $name
        )))
    };
}

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
    view_stage: CyclicallyAnimatedF64,
    show_integral: bool,
    show_splitting_diagram: bool,
    show_peaklets: bool,
    linked_x_axis: (f64, f64),
    side_panel_width: StoreOnNthCall<2, f32>,
    cached_partial_cascade: MultipletCascade,
}

impl Protonolysis {
    const ANIMATION_TIME_PER_STAGE: f64 = 2.0;
    const DEFAULT_PATTERN: &str = "Et₂O (CH₂)";
    const DEFAULT_X: f64 = 0.15;
    const DEFAULT_Y: f64 = 300.;
    const MAX_PROTON_COUNT: u32 = 9;
    const MAX_SPLITTERS: usize = 4;
    const SAMPLES: usize = 5000;
    const TOO_COMPLEX_THRESHOLD: u32 = 100;

    #[must_use]
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // cc.egui_ctx.set_debug_on_hover(true);

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

        let peak = Peak {
            splitters: PEAK_PRESETS[Self::DEFAULT_PATTERN].clone(),
            fwhm: 1.0,
        };
        let cached_partial_cascade = peak.build_multiplet_cascade();
        Self {
            field_strength: 600.,
            selected_preset: Self::DEFAULT_PATTERN,
            peak,
            view_stage: CyclicallyAnimatedF64::new(1., 0.0..=1.0, Self::ANIMATION_TIME_PER_STAGE),
            show_integral: true,
            show_splitting_diagram: true,
            show_peaklets: false,
            linked_x_axis: (-Self::DEFAULT_X, Self::DEFAULT_X),
            side_panel_width: StoreOnNthCall::default(),
            cached_partial_cascade,
        }
    }
}

impl Protonolysis {
    fn can_modify_configuration(&self) -> bool {
        !self.view_stage.is_animating()
    }

    fn update_animation_parameters(&mut self) {
        self.view_stage
            .set_range_clamping(0.0..=(self.peak.splitters.len() as f64));
        self.view_stage
            .set_duration(Self::ANIMATION_TIME_PER_STAGE * f64::from(self.peak.stage_count()));
    }

    fn is_preset_modified(&self) -> bool {
        self.peak.splitters != PEAK_PRESETS[self.selected_preset]
    }

    fn apply_preset(&mut self) {
        self.peak.splitters = PEAK_PRESETS[self.selected_preset].clone();
        self.update_animation_parameters();
        self.view_stage.set_value_clamping(f64::INFINITY);
    }
}

impl Protonolysis {
    fn controls(&mut self, ui: &mut Ui) {
        let enabled = self.can_modify_configuration();

        utils::vertical_space(ui);
        ui.heading("¹H-NMR Splitting Patterns");
        ui.separator();

        utils::two_column_grid("controls_instrument", ui, |ui| {
            ui.label("Instrument frequency:");
            ui.add_enabled(
                enabled,
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

        utils::two_column_grid("controls_peak", ui, |ui| {
            ui.label("Peak FWHM:")
                .on_hover_text("Full width at half maximum (i.e., broadness) of peaks");
            ui.add_enabled(
                enabled,
                Slider::new(&mut self.peak.fwhm, 0.5..=4.0)
                    .fixed_decimals(1)
                    .smart_aim(false)
                    .suffix(" Hz"),
            );
            ui.end_row();

            ui.label("Configure coupled protons:");
            ui.end_row();
        });

        ui.indent("controls_splitters", |ui| {
            ui.horizontal(|ui| {
                ui.label("Apply preset:");
                ui.add_enabled_ui(enabled, |ui| {
                    ComboBox::from_id_source("presets_selector")
                        .selected_text(self.selected_preset)
                        .show_ui(ui, |ui| {
                            for &preset in PEAK_PRESETS.keys().sorted() {
                                ui.selectable_value(&mut self.selected_preset, preset, preset);
                            }
                        });
                });

                let modified = self.is_preset_modified();
                let apply_button = &ui.add_enabled(
                    enabled && modified,
                    Button::new(if modified { "Apply" } else { "Applied" }),
                );
                if apply_button.clicked() {
                    self.apply_preset();
                }
            });

            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        enabled && self.peak.splitters.len() < Self::MAX_SPLITTERS,
                        Button::new("Add"),
                    )
                    .on_hover_text("Add new coupled proton type")
                    .clicked()
                {
                    self.peak.splitters.push(Splitter::default());
                    self.update_animation_parameters();
                    self.view_stage.set_value_clamping(*self.view_stage + 1.);
                }
                if ui
                    .add_enabled(enabled, Button::new("Sort by J"))
                    .on_hover_text("Sort by splitting constant in ascending order")
                    .clicked()
                {
                    self.peak.sort_by_j();
                }
            });

            let row_height = ui.text_style_height(&TextStyle::Body) + ui.spacing().item_spacing.y;
            let table = TableBuilder::new(ui)
                .striped(true)
                .cell_layout(Layout::left_to_right(Align::Center))
                .columns(Column::auto_with_initial_suggestion(20.), 5)
                .header(row_height, |mut header| {
                    let mut col = |text: &str| {
                        header.col(|ui| {
                            ui.label(RichText::new(text).underline());
                        });
                    };
                    col("");
                    col("Count");
                    col("J (Hz)");
                    col("Pattern");
                    col("Actions");
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
                            ui.add_enabled(
                                enabled,
                                Slider::new(&mut splitter.n, 1..=Self::MAX_PROTON_COUNT),
                            );
                        });
                        row.col(|ui| {
                            ui.add_enabled(
                                enabled,
                                Slider::new(&mut splitter.j, 0.2..=20.0)
                                    .fixed_decimals(1)
                                    .smart_aim(false),
                            );
                        });
                        row.col(|ui| {
                            let label = ui.label(splitter.abbreviate_pattern());
                            if let Some(name) = splitter.name_pattern() {
                                label.on_hover_text(name);
                            }
                        });
                        row.col(|ui| {
                            let mut button = |enabled2, text, hover| {
                                ui.add_enabled(enabled && enabled2, Button::new(text))
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
                            if button(self.peak.splitters.len() > 1, "\u{2717}", "Delete") {
                                self.peak.splitters.remove(i);
                                self.view_stage.set_value_clamping(*self.view_stage - 1.);
                                self.update_animation_parameters();
                            }
                        });
                    };
                    body.row(row_height, row);
                    i += 1;
                }
            });

            utils::vertical_space(ui);

            ui.label(format!(
                "Resulting pattern: {}",
                self.peak.name().unwrap_or("<complex>".to_owned())
            ));

            if self.peak.total_peaklet_count() > Self::TOO_COMPLEX_THRESHOLD {
                ui.label(
                    "⚠ The requested splitting pattern is highly complex and may result \
                    in performance degradation!",
                );
            }
        });

        ui.separator();

        utils::two_column_grid("controls_view", ui, |ui| {
            ui.label("Apply splitting up to level:").on_hover_text(
                "Draw the peak as if only the first n proton types were present. A fractional \
                    value indicates partial application of the last splitting constant.",
            );
            ui.horizontal(|ui| {
                self.view_stage.tick(ui);
                ui.style_mut().spacing.slider_width = 200.;
                ui.add(
                    Slider::from_get_set(self.view_stage.range(), |value| {
                        if let Some(value) = value {
                            self.view_stage.set_value_clamping(value);
                        }
                        *self.view_stage
                    })
                    .custom_formatter(|x, _| {
                        if approx::abs_diff_eq!(x, x.round(), epsilon = 8e-3) {
                            format!("{x:.0}")
                        } else {
                            format!("{x:.2}")
                        }
                    }),
                );
                let animate_text = if self.view_stage.is_animating() {
                    "Stop"
                } else {
                    "Animate"
                };
                if ui.button(animate_text).clicked() {
                    self.view_stage.toggle_animation();
                }
            });
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

        self.cached_partial_cascade = self
            .peak
            .nth_partial_peak(FractionalStageIndex::new(*self.view_stage))
            .build_multiplet_cascade();
    }

    fn peak_viewer(&mut self, ui: &mut Ui) {
        utils::inner_bottom_panel("plot_interaction", ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Controls:");
                ui.code("drag");
                ui.label("to pan,");
                ui.code("scroll");
                ui.label("to zoom vertically,");
                ui.code("ctrl+scroll");
                ui.label("to zoom horizontally.");
            });
        });

        let Vec2 {
            x: available_width,
            y: available_height,
        } = ui.available_size();
        let placement_origin = ui.next_widget_position();
        let plot_height =
            available_height - ui.text_style_height(&TextStyle::Body) - ui.spacing().item_spacing.y;

        let waveform = self
            .cached_partial_cascade
            .final_waveform(self.field_strength);

        let peak_plot = utils::make_noninteractable_plot("peak_plot")
            .include_x(-Self::DEFAULT_X)
            .include_x(Self::DEFAULT_X)
            .include_y(Self::DEFAULT_Y * -0.05)
            .include_y(Self::DEFAULT_Y * 1.1)
            .allow_double_click_reset(true)
            .height(plot_height);
        peak_plot.show(ui, |plot_ui| {
            utils::peak_viewer_interactions(plot_ui, &mut self.linked_x_axis);

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

        if !self.show_integral {
            return;
        }

        let integral_plot = utils::make_noninteractable_plot("integral_plot")
            .include_x(-Self::DEFAULT_X)
            .include_x(Self::DEFAULT_X)
            .include_y(-0.05)
            .include_y(1.05)
            .show_axes([false; 2])
            .show_background(false);
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
                (placement_origin.x, placement_origin.y + plot_height * 0.025).into(),
                (available_width, plot_height * 0.2).into(),
            ),
            draw_integral_plot,
        );
    }

    fn splitting_diagram(&self, ui: &mut Ui) {
        let plot = utils::make_noninteractable_plot("splitting_diagram")
            .show_axes([false; 2])
            .show_background(false)
            .show_x(false)
            .show_y(false)
            .auto_bounds_x()
            .auto_bounds_y()
            .height(
                ui.available_height()
                    .min(100. * self.peak.stage_count() as f32),
            )
            .data_aspect(15.);

        plot.show(ui, |plot_ui| {
            splitting_diagram::draw_splitting_diagram(
                plot_ui,
                &self.peak.build_multiplet_cascade(),
                &self.cached_partial_cascade,
                FractionalStageIndex::new(*self.view_stage),
            );
        });
    }

    fn side_panel_contents(&mut self, ui: &mut Ui) {
        self.controls(ui);

        ui.separator();

        if self.show_splitting_diagram {
            ui.label("Splitting diagram:");
            self.splitting_diagram(ui);
        }
    }

    fn footer(ui: &mut Ui) {
        utils::inner_bottom_panel("about_footer", ui, |ui| {
            // Right-alignment disabled due to `exact_width` bug.
            // ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.horizontal(|ui| {
                ui.hyperlink_to("Source", env!("CARGO_PKG_REPOSITORY"));
                ui.separator();
                ui.label(concat!(app_name!(), " v", version!()));
            });
        });
    }

    fn full_layout(&mut self, ctx: &Context) {
        let mut side_panel = SidePanel::right("controls").resizable(false);
        if let Some(width) = self.side_panel_width.get() {
            side_panel = side_panel.exact_width(width.max(ctx.available_rect().width() * 0.25));
        }
        let response = side_panel.show(ctx, |ui| {
            Self::footer(ui);
            self.side_panel_contents(ui);
        });
        // Note that the table contained within does sizing on the first frame. Thus we take the
        // computed size from the second frame.
        self.side_panel_width.set(response.response.rect.width());

        CentralPanel::default()
            .frame({
                let mut frame = egui::Frame::central_panel(ctx.style().as_ref());
                frame.inner_margin.bottom = 2.0;
                frame
            })
            .show(ctx, |ui| {
                self.peak_viewer(ui);
            });
    }

    fn compressed_layout(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::both().show(ui, |ui| {
                // TODO: make this usable on phones.
                // FIXME: somehow make the splitting diagram respond to page scrolls.
                ui.scope(|ui| {
                    ui.set_max_height(f32::INFINITY);
                    self.side_panel_contents(ui);
                });
                ui.separator();
                ui.scope(|ui| {
                    ui.set_height(ctx.screen_rect().width() * 0.8);
                    self.peak_viewer(ui);
                });
                Self::footer(ui);
            });
        });
    }
}

impl eframe::App for Protonolysis {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if ctx.screen_rect().width() < 850. {
            self.compressed_layout(ctx);
        } else {
            self.full_layout(ctx);
        }
    }
}
