// src/main.rs
#![windows_subsystem = "windows"]
use eframe;

mod file_manager;
mod file_operations;
mod search;
mod ui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(400.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Aimet Manager",
        options,
        Box::new(|cc| Box::new(file_manager::FileManager::new(cc))),
    )
}
