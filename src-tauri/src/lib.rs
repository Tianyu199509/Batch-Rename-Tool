use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize)]
struct FileInfo {
    name: String,
    path: String,
    is_dir: bool,
}

#[derive(Deserialize)]
struct RenameTask {
    old_path: String,
    new_path: String,
}

#[derive(Serialize)]
struct RenameResult {
    old_path: String,
    new_path: String,
    success: bool,
    error: Option<String>,
}

#[tauri::command]
fn list_files(dir_path: &str) -> Result<Vec<FileInfo>, String> {
    let mut files = Vec::new();
    let entries = fs::read_dir(dir_path).map_err(|e| e.to_string())?;

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = path.is_dir();

            // 过滤掉目录，只显示文件
            if !is_dir {
                files.push(FileInfo {
                    name,
                    path: path.to_string_lossy().to_string(),
                    is_dir,
                });
            }
        }
    }
    
    files.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(files)
}

#[tauri::command]
fn rename_files(tasks: Vec<RenameTask>) -> Result<Vec<RenameResult>, String> {
    let mut results = Vec::new();

    for task in tasks {
        let success;
        let mut error = None;

        match fs::rename(&task.old_path, &task.new_path) {
            Ok(_) => success = true,
            Err(e) => {
                success = false;
                error = Some(e.to_string());
            }
        }

        results.push(RenameResult {
            old_path: task.old_path,
            new_path: task.new_path,
            success,
            error,
        });
    }

    Ok(results)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![list_files, rename_files])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
