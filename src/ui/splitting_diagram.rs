use eframe::egui::plot::{Line, LineStyle, PlotUi};
use eframe::epaint::Color32;

use crate::peak::{Peaklet, SplittingRelationship};

const STAGE_ORIGIN: f64 = 0.;
const STAGE_HEIGHT: f64 = 50.;
const MAX_PEAKLET_HEIGHT: f64 = 35.;

#[allow(clippy::cast_precision_loss)]
fn base_height_of(stage: usize) -> f64 {
    STAGE_ORIGIN - stage as f64 * STAGE_HEIGHT
}

fn tip_height_of(peaklet: &Peaklet, stage: usize, max_integration: f64) -> f64 {
    base_height_of(stage) + (peaklet.integration / max_integration) * MAX_PEAKLET_HEIGHT
}

pub(super) fn draw_peaklet_marker(
    plot_ui: &mut PlotUi,
    peaklet: &Peaklet,
    stage: usize,
    max_integration: f64,
    enabled: bool,
) {
    plot_ui.line(
        Line::new(vec![
            [peaklet.δ, base_height_of(stage)],
            [peaklet.δ, tip_height_of(peaklet, stage, max_integration)],
        ])
        .color(if enabled {
            Color32::LIGHT_BLUE
        } else {
            Color32::DARK_GRAY
        })
        .width(3.),
    );
}

pub(super) fn draw_group_children_and_connectors(
    plot_ui: &mut PlotUi,
    group: &SplittingRelationship,
    stage: usize,
    max_integration: f64,
    enabled: bool,
) {
    let parent_base = [group.parent.δ, base_height_of(stage - 1)];
    for child in group.children {
        draw_peaklet_marker(plot_ui, child, stage, max_integration, enabled);
        let child_tip = [child.δ, tip_height_of(child, stage, max_integration)];
        let corner = [child.δ, base_height_of(stage) + MAX_PEAKLET_HEIGHT];
        plot_ui.line(
            Line::new(vec![child_tip, corner, parent_base])
                .color(if enabled {
                    Color32::GRAY
                } else {
                    Color32::DARK_GRAY
                })
                .style(LineStyle::dashed_dense())
                .width(1.),
        );
    }
}
