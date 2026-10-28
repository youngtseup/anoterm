//! 本地文件系统命令：为 SFTP 双栏模式提供本地侧的浏览与基础文件操作。

use std::path::Path;
use std::time::UNIX_EPOCH;

use tauri::Manager;

use crate::ssh::sftp::SftpEntry;

/// 用户主目录（本地栏初始路径）。
#[tauri::command]
pub fn local_home(app: tauri::AppHandle) -> Result<String, String> {
    app.path()
        .home_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|e| e.to_string())
}

/// 列出本地目录（目录排前，按名称不区分大小写排序）。
#[tauri::command]
pub async fn local_list(path: String) -> Result<Vec<SftpEntry>, String> {
    tokio::task::spawn_blocking(move || list_dir(&path))
        .await
        .map_err(|e| e.to_string())?
}

fn list_dir(path: &str) -> Result<Vec<SftpEntry>, String> {
    let mut out = Vec::new();
    let rd = std::fs::read_dir(path).map_err(|e| format!("读取目录失败: {e}"))?;
    for entry in rd.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let full = entry.path();
        let is_symlink = std::fs::symlink_metadata(&full)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false);
        // metadata 跟随符号链接，目录判断与远端行为保持一致
        let md = std::fs::metadata(&full).ok();
        let is_dir = md.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        let size = md.as_ref().filter(|m| m.is_file()).map(|m| m.len());
        let mtime = md
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs().min(u32::MAX as u64) as u32);
        out.push(SftpEntry {
            name,
            path: full.to_string_lossy().into_owned(),
            is_dir,
            is_symlink,
            size,
            mtime,
            permissions: unix_mode(&md),
        });
    }
    out.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(out)
}

#[cfg(unix)]
fn unix_mode(md: &Option<std::fs::Metadata>) -> Option<u32> {
    use std::os::unix::fs::PermissionsExt;
    md.as_ref().map(|m| m.permissions().mode())
}

#[cfg(not(unix))]
fn unix_mode(_md: &Option<std::fs::Metadata>) -> Option<u32> {
    None
}

/// 创建本地目录。
#[tauri::command]
pub async fn local_mkdir(path: String) -> Result<(), String> {
    tokio::fs::create_dir(&path)
        .await
        .map_err(|e| format!("创建目录失败: {e}"))
}

/// 重命名/移动本地文件或目录。
#[tauri::command]
pub async fn local_rename(old_path: String, new_path: String) -> Result<(), String> {
    tokio::fs::rename(&old_path, &new_path)
        .await
        .map_err(|e| format!("重命名失败: {e}"))
}

/// 删除本地文件或目录（目录递归删除）。
#[tauri::command]
pub async fn local_remove(path: String, is_dir: bool) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let p = Path::new(&path);
        if is_dir {
            std::fs::remove_dir_all(p).map_err(|e| format!("删除目录失败: {e}"))
        } else {
            std::fs::remove_file(p).map_err(|e| format!("删除文件失败: {e}"))
        }
    })
    .await
    .map_err(|e| e.to_string())?
}
