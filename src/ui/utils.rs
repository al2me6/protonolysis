use eframe::egui::plot::{PlotBounds, PlotUi};
use eframe::egui::{self, CursorIcon, Margin, TopBottomPanel, Ui};
use eframe::epaint::Vec2;

/// Apply custom zoom and pan interactions for peak plots.
pub(super) fn peak_viewer_interactions(plot_ui: &mut PlotUi, x_axis: &mut (f64, f64)) {
    // FIXME: touch support.

    if !plot_ui.plot_hovered() {
        return;
    }

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
    let (mut bounds_min, mut bounds_max) = (bounds.min(), bounds.max());
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

pub(super) fn inner_bottom_panel(
    id: &'static str,
    ui: &mut Ui,
    add_contents: impl FnOnce(&mut Ui),
) {
    TopBottomPanel::bottom(id)
        .show_separator_line(false)
        .frame(
            egui::Frame::side_top_panel(ui.style())
                .inner_margin(Margin::symmetric(0.0, ui.style().spacing.item_spacing.y)),
        )
        .show_inside(ui, add_contents);
}
