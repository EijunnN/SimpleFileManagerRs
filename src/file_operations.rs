use crate::file_manager::{FileEntry, FileManager};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;
use walkdir::WalkDir;

pub fn read_directory(path: &Path) -> Vec<FileEntry> {
    WalkDir::new(path)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| {
            let path = e.path().to_path_buf();
            let metadata = fs::metadata(&path).unwrap();
            let is_dir = metadata.is_dir();
            let size = if is_dir {
                get_dir_size(&path)
            } else {
                metadata.len()
            };
            FileEntry {
                path,
                size,
                modified: metadata.modified().unwrap_or(SystemTime::now()),
                is_dir,
            }
        })
        .collect()
}

pub fn get_dir_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.metadata().ok())
        .filter(|metadata| metadata.is_file())
        .map(|metadata| metadata.len())
        .sum()
}

pub fn delete_file(file_manager: &mut FileManager, path: &Path) {
    if path.is_dir() {
        if let Err(e) = fs::remove_dir_all(path) {
            eprintln!("Error deleting directory: {}", e);
        }
    } else {
        if let Err(e) = fs::remove_file(path) {
            eprintln!("Error deleting file: {}", e);
        }
    }
    file_manager.update_entries();
    file_manager.selected_file = None;
}

pub fn copy_file(source: &Path, destination: &Path) {
    if source.is_dir() {
        if let Err(e) = fs::create_dir_all(destination) {
            eprintln!("Error creating directory: {}", e);
            return;
        }
        for entry in fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            let new_dest = destination.join(entry.file_name());
            copy_file(&entry.path(), &new_dest);
        }
    } else {
        if let Err(e) = fs::copy(source, destination) {
            eprintln!("Error copying file: {}", e);
        }
    }
}

pub fn paste_file(file_manager: &mut FileManager) {
    if let Some(source) = &file_manager.clipboard {
        let destination = file_manager.current_dir.join(source.file_name().unwrap());
        copy_file(source, &destination);
        file_manager.update_entries();
    }
}

pub fn open_file(path: &Path) {
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", "start", "", &path.to_string_lossy()])
            .spawn()
            .expect("Failed to open file");
    } else if cfg!(target_os = "macos") {
        Command::new("open")
            .arg(path)
            .spawn()
            .expect("Failed to open file");
    } else {
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .expect("Failed to open file");
    }
}

pub fn open_terminal(path: &Path) {
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", "start", "cmd.exe", "/K", &format!("cd /d {:?}", path)])
            .spawn()
            .expect("Failed to open terminal");
    } else if cfg!(target_os = "macos") {
        Command::new("open")
            .args(&["-a", "Terminal", path.to_str().unwrap()])
            .spawn()
            .expect("Failed to open terminal");
    } else {
        Command::new("x-terminal-emulator")
            .arg("--working-directory")
            .arg(path)
            .spawn()
            .expect("Failed to open terminal");
    }
}

pub fn get_disk_usage(path: &Path) -> (u64, u64) {
    let available = fs2::available_space(path).unwrap_or(0);
    let total = fs2::total_space(path).unwrap_or(0);
    (total - available, total)
}
