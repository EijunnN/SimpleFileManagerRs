// src/ui.rs
use crate::file_manager::FileManager;
use crate::file_operations;
use crate::search;
use eframe::egui;
use std::path::PathBuf;
use std::time::SystemTime;

pub fn render(file_manager: &mut FileManager, ctx: &egui::Context) {
    if let Some(background) = &file_manager.background_image {
        let screen_rect = ctx.screen_rect();
        let painter = ctx.layer_painter(egui::LayerId::background());
        painter.image(
            background.id(),
            screen_rect,
            screen_rect,
            egui::Color32::WHITE,
        );
    }

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("File Manager");
            ui.checkbox(&mut file_manager.dark_mode, "Dark Mode");
            if ui.button("Set Background").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    file_manager.set_background_image(ctx, path.to_str().unwrap());
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label("Current Directory:");
            if ui.button("📂").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    file_manager.change_directory(path);
                }
            }
            ui.label(file_manager.current_dir.to_string_lossy());
        });

        if ui.button("⬆️ Up").clicked() && file_manager.current_dir.parent().is_some() {
            file_manager.current_dir.pop();
            file_manager.update_entries();
        }

        ui.horizontal(|ui| {
            ui.label("Search:");
            if ui
                .text_edit_singleline(&mut file_manager.search_query)
                .changed()
            {
                search::search_files(file_manager);
            }
        });

        let (used_space, total_space) = file_manager.get_disk_usage();
        ui.label(format!(
            "Disk Usage: {:.2} GB / {:.2} GB",
            used_space as f64 / 1_000_000_000.0,
            total_space as f64 / 1_000_000_000.0
        ));

        render_file_list(file_manager, ui);
        render_file_operations(file_manager, ui);
    });
}

fn render_file_list(file_manager: &mut FileManager, ui: &mut egui::Ui) {
    let mut to_select: Option<PathBuf> = None;
    let mut to_navigate: Option<PathBuf> = None;
    let mut to_open: Option<PathBuf> = None;
    let mut to_copy: Option<PathBuf> = None;
    let mut to_paste: bool = false;
    let mut to_delete: Option<PathBuf> = None;
    let mut to_open_terminal: Option<PathBuf> = None;
    let mut to_create_folder: bool = false;
    let mut to_rename: Option<PathBuf> = None;
    let mut new_name: String = String::new();

    let entries_to_show = if !file_manager.search_query.is_empty() {
        &file_manager.search_results
    } else {
        &file_manager.entries
    };

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label("Name");
            ui.label("Type");
            ui.label("Size");
            ui.label("Modified");
        });

        for entry in entries_to_show {
            let name = entry.path.file_name().unwrap();
            let is_dir = entry.is_dir;
            let file_type = if is_dir { "Directory" } else { "File" };
            let size = if is_dir {
                format!("{:.2} MB", entry.size as f64 / 1_000_000.0)
            } else {
                format!("{:.2} KB", entry.size as f64 / 1024.0)
            };

            ui.horizontal(|ui| {
                let response = ui.selectable_label(
                    file_manager.selected_file.as_ref() == Some(&entry.path),
                    format!("{}", name.to_string_lossy()),
                );

                ui.label(file_type);
                ui.label(size);
                ui.label(format!(
                    "{}",
                    entry
                        .modified
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                ));

                if response.clicked() {
                    if is_dir {
                        to_navigate = Some(entry.path.clone());
                    } else {
                        to_select = Some(entry.path.clone());
                    }
                }

                if response.double_clicked() && !is_dir {
                    to_open = Some(entry.path.clone());
                }

                response.context_menu(|ui| {
                    if ui.button("Copy").clicked() {
                        to_copy = Some(entry.path.clone());
                        ui.close_menu();
                    }
                    if ui.button("Paste").clicked() && file_manager.clipboard.is_some() {
                        to_paste = true;
                        ui.close_menu();
                    }
                    if ui.button("Delete").clicked() {
                        to_delete = Some(entry.path.clone());
                        ui.close_menu();
                    }
                    if ui.button("Open Terminal Here").clicked() {
                        to_open_terminal = Some(entry.path.clone());
                        ui.close_menu();
                    }
                    if ui.button("Create Folder").clicked() {
                        to_create_folder = true;
                        ui.close_menu();
                    }
                    if ui.button("Rename").clicked() {
                        to_rename = Some(entry.path.clone());
                        ui.close_menu();
                    }
                });
            });
        }
    });

    if let Some(path) = to_navigate {
        file_manager.change_directory(path);
    }

    if let Some(path) = to_select {
        file_manager.selected_file = Some(path);
    }

    if let Some(path) = to_rename {
        ui.horizontal(|ui| {
            ui.label("New Name:");
            ui.text_edit_singleline(&mut new_name);
            if ui.button("Rename").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                file_manager.rename_file(&path, &new_name);
            }
        });
    }

    if let Some(path) = to_open {
        file_operations::open_file(&path);
    }

    if let Some(path) = to_copy {
        file_manager.clipboard = Some(path);
    }

    if to_paste {
        file_operations::paste_file(file_manager);
    }

    if let Some(path) = to_delete {
        file_operations::delete_file(file_manager, &path);
    }

    if let Some(path) = to_open_terminal {
        file_operations::open_terminal(&path);
    }

    if to_create_folder {
        let new_folder_path = file_manager.current_dir.join("New Folder");
        if let Err(e) = std::fs::create_dir(&new_folder_path) {
            eprintln!("Error creating folder: {}", e);
        } else {
            file_manager.update_entries();
        }
    }
}
fn render_file_operations(file_manager: &mut FileManager, ui: &mut egui::Ui) {
    let selected_file = file_manager.selected_file.clone();
    let clipboard = file_manager.clipboard.clone();

    if let Some(selected) = selected_file {
        ui.separator();
        ui.label(format!("Selected File: {:?}", selected));
        if ui.button("Open").clicked() {
            file_operations::open_file(&selected);
        }
        if ui.button("Copy").clicked() {
            file_manager.clipboard = Some(selected.clone());
        }
        if ui.button("Delete").clicked() {
            file_operations::delete_file(file_manager, &selected);
        }
        if ui.button("Open Terminal Here").clicked() {
            file_operations::open_terminal(&selected);
        }
    }

    if let Some(clip) = clipboard {
        ui.separator();
        ui.label(format!("Clipboard File: {:?}", clip));
        if ui.button("Paste").clicked() {
            file_operations::paste_file(file_manager);
        }
    }
}
