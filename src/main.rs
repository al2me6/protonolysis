#![warn(clippy::pedantic)]
#![allow(
    clippy::len_without_is_empty,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::too_many_lines
)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(drain_filter)]

pub mod numerics;
pub mod peak;
pub mod protonolysis;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    tracing_subscriber::fmt::init();
    eframe::run_native(
        "Protonolysis",
        native_options,
        Box::new(|cc| Box::new(protonolysis::Protonolysis::new(cc))),
    )?;
    Ok(())
}
