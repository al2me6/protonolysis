#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_precision_loss,
    clippy::len_without_is_empty,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::too_many_lines
)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(drain_filter, lazy_cell)]

macro_rules! version {
    () => {
        concat!(
            env!("CARGO_PKG_VERSION_MAJOR"),
            ".",
            env!("CARGO_PKG_VERSION_MINOR"),
            ".",
            env!("CARGO_PKG_VERSION_PATCH"),
        )
    };
}

macro_rules! app_name {
    () => {
        "Protonolysis"
    };
}

pub mod numerics;
pub mod peak;
pub mod ui;
pub mod utils;

#[cfg(not(target_arch = "wasm32"))]
pub const ICON: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo32.png"));

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let icon = image::load_from_memory(ICON).unwrap().into_rgba8();
    let (icon_width, icon_height) = icon.dimensions();
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(eframe::epaint::Vec2 { x: 1200., y: 600. }),
        icon_data: Some(eframe::IconData {
            rgba: icon.into_raw(),
            width: icon_width,
            height: icon_height,
        }),
        ..Default::default()
    };
    tracing_subscriber::fmt::init();
    eframe::run_native(
        app_name!(),
        native_options,
        Box::new(|cc| Box::new(ui::Protonolysis::new(cc))),
    )?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "canvas",
            web_options,
            Box::new(|cc| Box::new(ui::Protonolysis::new(cc))),
        )
        .await
        .expect("failed to start eframe");
    });
}
