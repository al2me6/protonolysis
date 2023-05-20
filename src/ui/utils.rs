use eframe::egui::plot::{PlotBounds, PlotUi};
use eframe::egui::{self, CursorIcon, InputState, Margin, TopBottomPanel, Ui};
use eframe::epaint::Vec2;

/// Apply custom zoom and pan interactions for peak plots.
pub(super) fn peak_viewer_interactions(plot_ui: &mut PlotUi, x_axis: &mut (f64, f64)) {
    if !plot_ui.plot_hovered() {
        return;
    }

    let multitouch = plot_ui.ctx().input(InputState::multi_touch);

    // Custom zoom:
    let bounds = plot_ui.plot_bounds();
    let (mut bounds_min, mut bounds_max) = (bounds.min(), bounds.max());
    // scroll wheel y zoom:
    let raw_scroll_y = plot_ui.ctx().input(|i| f64::from(i.scroll_delta.y));
    if raw_scroll_y != 0. {
        let scroll_y = (raw_scroll_y / 200.).exp();
        bounds_min[1] /= scroll_y;
        bounds_max[1] /= scroll_y;
    }
    // ctrl-scroll x-zoom or pinch-to-zoom (x and y):
    // This seems to eat the raw scroll delta in the former case.
    let zoom_delta = plot_ui.ctx().input(InputState::zoom_delta_2d);
    let zoom_x = f64::from(zoom_delta.x);
    bounds_min[0] /= zoom_x;
    bounds_max[0] /= zoom_x;
    if multitouch.is_some() {
        let pinch_y = f64::from(zoom_delta.y);
        bounds_min[1] /= pinch_y;
        bounds_max[1] /= pinch_y;
    }
    *x_axis = (bounds_min[0], bounds_max[0]);
    plot_ui.set_plot_bounds(PlotBounds::from_min_max(bounds_min, bounds_max));

    // Custom pan:
    let drag = plot_ui
        .ctx()
        .input(|i| i.pointer.primary_down().then(|| i.pointer.delta()));
    if let Some(drag) = drag {
        // Don't allow drag-to-pan while in pinch-to-zoom.
        if multitouch.is_none() {
            plot_ui.ctx().set_cursor_icon(CursorIcon::ResizeHorizontal);
            plot_ui.translate_bounds(Vec2 { x: -drag.x, y: 0. });
        }
    }
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
