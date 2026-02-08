use std::path::PathBuf;
use std::sync::Arc;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::config::Settings;
use super::stream::OmniusClientState;

#[derive(Debug, Serialize)]
pub struct StorageInfo {
    pub download_path: String,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub file_count: usize,
}

#[tauri::command]
pub async fn get_storage_info(app: AppHandle) -> Result<StorageInfo, String> {
    let download_path = get_app_data_dir(&app)?;

    println!("[get_storage_info] Checking path: {:?}", download_path);

    let (used_bytes, file_count) = calculate_dir_size(&download_path)?;

    println!("[get_storage_info] Found {} bytes in {} files", used_bytes, file_count);

    let free_bytes = get_free_space(&download_path)?;

    Ok(StorageInfo {
        download_path: download_path.to_string_lossy().to_string(),
        used_bytes,
        free_bytes,
        file_count,
    })
}

#[tauri::command]
pub async fn clear_cache(app: AppHandle) -> Result<u64, String> {
    let download_path = get_app_data_dir(&app)?;

    println!("[clear_cache] Clearing path: {:?}", download_path);

    let (size_cleared, _) = calculate_dir_size(&download_path)?;

    if download_path.exists() {
        for entry in std::fs::read_dir(&download_path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();

            // Don't delete config.json
            if path.file_name().map(|n| n == "config.json").unwrap_or(false) {
                continue;
            }

            if path.is_dir() {
                std::fs::remove_dir_all(&path).map_err(|e| e.to_string())?;
            } else {
                std::fs::remove_file(&path).map_err(|e| e.to_string())?;
            }
        }
    }

    println!("[clear_cache] Cleared {} bytes", size_cleared);
    Ok(size_cleared)
}

#[tauri::command]
pub async fn get_download_path(app: AppHandle) -> Result<String, String> {
    let path = get_app_data_dir(&app)?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn get_server_url(
    settings: State<'_, Arc<Settings>>,
) -> Result<String, String> {
    Ok(settings.server_url())
}

#[tauri::command]
pub async fn set_server_url(
    settings: State<'_, Arc<Settings>>,
    client: State<'_, OmniusClientState>,
    url: String,
) -> Result<(), String> {
    settings.set_server_url(&url);
    let mut client = client.write().await;
    client.set_base_url(&url);
    println!("[set_server_url] Server URL changed to: {}", url);
    Ok(())
}

fn get_app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path().app_data_dir().map_err(|e| format!("Failed to get app data dir: {}", e))
}

fn calculate_dir_size(path: &PathBuf) -> Result<(u64, usize), String> {
    let mut total_size: u64 = 0;
    let mut file_count: usize = 0;

    if !path.exists() {
        return Ok((0, 0));
    }

    fn visit_dirs(dir: &PathBuf, size: &mut u64, count: &mut usize) -> std::io::Result<()> {
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dirs(&path, size, count)?;
                } else {
                    *size += entry.metadata()?.len();
                    *count += 1;
                }
            }
        }
        Ok(())
    }

    visit_dirs(path, &mut total_size, &mut file_count).map_err(|e| e.to_string())?;
    Ok((total_size, file_count))
}

fn get_free_space(path: &PathBuf) -> Result<u64, String> {
    #[cfg(unix)]
    {
        use std::mem;

        let check_path = if path.exists() {
            path.clone()
        } else if let Some(parent) = path.parent() {
            if parent.exists() {
                parent.to_path_buf()
            } else {
                PathBuf::from("/")
            }
        } else {
            PathBuf::from("/")
        };

        let path_str = check_path.to_string_lossy();
        let c_path = match std::ffi::CString::new(path_str.as_bytes()) {
            Ok(p) => p,
            Err(_) => return Ok(0),
        };

        unsafe {
            let mut stat: libc::statvfs = mem::zeroed();
            if libc::statvfs(c_path.as_ptr(), &mut stat) == 0 {
                Ok(stat.f_bavail as u64 * stat.f_bsize as u64)
            } else {
                Ok(0)
            }
        }
    }

    #[cfg(windows)]
    {
        Ok(0)
    }
}
