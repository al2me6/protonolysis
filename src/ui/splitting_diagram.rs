use eframe::egui::plot::{Line, LineStyle, PlotUi};
use eframe::epaint::Color32;

use crate::peak::{FractionalStageIndex, MultipletCascade, Peaklet, SplittingRelationship};

const STAGE_ORIGIN: f64 = 0.;
const MAX_PEAKLET_HEIGHT: f64 = 0.7;

fn base_height_of(stage: usize) -> f64 {
    STAGE_ORIGIN - stage as f64
}

fn tip_height_of(peaklet: &Peaklet, stage: usize, max_integration: f64) -> f64 {
    base_height_of(stage) + (peaklet.integration / max_integration) * MAX_PEAKLET_HEIGHT
}

fn draw_peaklet_marker(
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
            let mut color = Color32::LIGHT_BLUE;
            color[3] = 127;
            color
        } else {
            Color32::DARK_GRAY
        })
        .width(3.),
    );
}

fn draw_group_children_and_connectors(
    plot_ui: &mut PlotUi,
    group: SplittingRelationship,
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

pub(super) fn draw_splitting_diagram(
    plot_ui: &mut PlotUi,
    full_cascade: &MultipletCascade,
    partial_cascade: &MultipletCascade,
    view_stage: FractionalStageIndex,
) {
    draw_peaklet_marker(plot_ui, &full_cascade.base_peaklet(), 0, 1., true);

    let last_full = view_stage.full();
    let maybe_partial = view_stage.partial_and_index();

    for stage in 1..=full_cascade.child_stages_count() {
        let max_integration = full_cascade.max_integration_of_stage(stage);
        let mut enabled = stage <= last_full;
        if let Some((partial_idx, part)) = maybe_partial {
            enabled |= stage == partial_idx
                && (partial_cascade.is_stage_resolved(partial_idx) || part > 0.9);
        }
        for group in full_cascade.iter_nth_stage(stage) {
            draw_group_children_and_connectors(plot_ui, group, stage, max_integration, enabled);
        }
    }
}
