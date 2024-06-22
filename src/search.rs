use crate::file_manager::{FileEntry, FileManager};
use std::time::{Duration, Instant, SystemTime};
use walkdir::WalkDir;

pub fn search_files(file_manager: &mut FileManager) {
    let now = Instant::now();
    if now.duration_since(file_manager.last_search_time) < Duration::from_millis(400) {
        return;
    }

    file_manager.last_search_time = now;
    file_manager.search_results = WalkDir::new(&file_manager.current_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|s| {
                    s.to_lowercase()
                        .contains(&file_manager.search_query.to_lowercase())
                })
                .unwrap_or(false)
        })
        .map(|e| {
            let metadata = e.metadata().unwrap();
            FileEntry {
                path: e.path().to_path_buf(),
                size: metadata.len(),
                modified: metadata.modified().unwrap_or(SystemTime::now()),
                is_dir: metadata.is_dir(),
            }
        })
        .collect();
}
