//! 数据库相关的 Tauri 命令：分组、连接与最近连接的增删改查。

use crate::storage::{ConnectionInput, ConnectionOrder, Db, Group, RecentConnection};

/// 列出全部分组（含各自连接）。
#[tauri::command]
pub fn list_groups(state: tauri::State<'_, Db>) -> Result<Vec<Group>, String> {
    let storage = state.0.lock().map_err(|e| e.to_string())?;
    storage.list_groups().map_err(|e| e.to_string())
}

/// 新建分组。
#[tauri::command]
pub fn create_group(state: tauri::State<'_, Db>, name: String) -> Result<Group, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("分组名称不能为空".to_string());
    }
    let mut storage = state.0.lock().map_err(|e| e.to_string())?;
    storage.create_group(&name).map_err(|e| e.to_string())
}

/// 重命名分组。
#[tauri::command]
pub fn rename_group(state: tauri::State<'_, Db>, id: String, name: String) -> Result<(), String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("分组名称不能为空".to_string());
    }
    let mut storage = state.0.lock().map_err(|e| e.to_string())?;
    storage.rename_group(&id, &name).map_err(|e| e.to_string())
}

/// 删除分组（连带删除其下所有连接）。
#[tauri::command]
pub fn delete_group(state: tauri::State<'_, Db>, id: String) -> Result<(), String> {
    let mut storage = state.0.lock().map_err(|e| e.to_string())?;
    storage.delete_group(&id).map_err(|e| e.to_string())
}

/// 新建连接。
#[tauri::command]
pub fn create_connection(
    state: tauri::State<'_, Db>,
    input: ConnectionInput,
) -> Result<crate::storage::Connection, String> {
    let mut storage = state.0.lock().map_err(|e| e.to_string())?;
    storage.create_connection(&input).map_err(|e| e.to_string())
}

/// 更新连接。
#[tauri::command]
pub fn update_connection(
    state: tauri::State<'_, Db>,
    id: String,
    input: ConnectionInput,
) -> Result<(), String> {
    let mut storage = state.0.lock().map_err(|e| e.to_string())?;
    storage
        .update_connection(&id, &input)
        .map_err(|e| e.to_string())
}

/// 删除连接。
#[tauri::command]
pub fn delete_connection(state: tauri::State<'_, Db>, id: String) -> Result<(), String> {
    let mut storage = state.0.lock().map_err(|e| e.to_string())?;
    storage.delete_connection(&id).map_err(|e| e.to_string())
}

/// 保存拖拽后的分组顺序、连接归属与连接顺序。
#[tauri::command]
pub fn reorder_tree(
    state: tauri::State<'_, Db>,
    group_ids: Vec<String>,
    connections: Vec<ConnectionOrder>,
) -> Result<(), String> {
    let mut storage = state.0.lock().map_err(|e| e.to_string())?;
    storage
        .reorder_tree(&group_ids, &connections)
        .map_err(|e| e.to_string())
}

/// 列出最近连接（按使用时间倒序，最多 30 条）。
#[tauri::command]
pub fn list_recent(state: tauri::State<'_, Db>) -> Result<Vec<RecentConnection>, String> {
    let storage = state.0.lock().map_err(|e| e.to_string())?;
    storage.list_recent().map_err(|e| e.to_string())
}

/// 清空最近连接记录。
#[tauri::command]
pub fn clear_recent(state: tauri::State<'_, Db>) -> Result<(), String> {
    let mut storage = state.0.lock().map_err(|e| e.to_string())?;
    storage.clear_recent().map_err(|e| e.to_string())
}
