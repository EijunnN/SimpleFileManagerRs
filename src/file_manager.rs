use crate::file_operations;
use eframe::egui;
use image;
use std::path::PathBuf;
use std::time::{Instant, SystemTime};

pub struct FileManager {
    pub current_dir: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected_file: Option<PathBuf>,
    pub clipboard: Option<PathBuf>,
    pub search_query: String,
    pub search_results: Vec<FileEntry>,
    pub last_search_time: Instant,
    pub dark_mode: bool,
    pub background_image: Option<egui::TextureHandle>,
}

pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
    pub modified: SystemTime,
    pub is_dir: bool,
}

impl FileManager {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let current_dir = std::env::current_dir().unwrap();
        let entries = file_operations::read_directory(&current_dir);
        Self {
            current_dir,
            entries,
            selected_file: None,
            clipboard: None,
            search_query: String::new(),
            search_results: Vec::new(),
            last_search_time: Instant::now(),
            dark_mode: false,
            background_image: None,
        }
    }

    pub fn update_entries(&mut self) {
        self.entries = file_operations::read_directory(&self.current_dir);
    }

    pub fn change_directory(&mut self, new_dir: PathBuf) {
        if new_dir.is_dir() {
            self.current_dir = new_dir;
            self.update_entries();
        }
    }

    pub fn get_disk_usage(&self) -> (u64, u64) {
        file_operations::get_disk_usage(&self.current_dir)
    }

    pub fn set_background_image(&mut self, ctx: &egui::Context, path: &str) {
        let image = image::open(path).expect("Failed to open image");
        let size = [image.width() as _, image.height() as _];
        let image_buffer = image.to_rgba8();
        let pixels = image_buffer.as_flat_samples();
        let texture = ctx.load_texture(
            "background",
            egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()),
            egui::TextureOptions::LINEAR,
        );
        self.background_image = Some(texture);
    }

    pub fn rename_file(&mut self, old_path: &PathBuf, new_name: &str) {
        let new_path = old_path.with_file_name(new_name);
        if let Err(e) = std::fs::rename(old_path, &new_path) {
            eprintln!("Error renaming file: {}", e);
        } else {
            self.update_entries();
        }
    }
}

impl eframe::App for FileManager {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
        crate::ui::render(self, ctx);
    }
}
