//! 应用装配入口：初始化插件、数据目录与全局状态，并注册全部 Tauri 命令。
//!
//! 模块划分：
//! - [`db_commands`]：数据库（分组/连接/最近连接）命令
//! - [`ssh`]：SSH/SFTP 会话管理（连接、终端、文件操作）
//! - [`local_fs`]：本地文件系统命令（SFTP 双栏模式的本地侧）
//! - [`storage`]：SQLite 数据层

mod crypto;
mod db_commands;
mod local_fs;
mod ssh;
mod storage;

use std::sync::Mutex;

use db_commands::{
    clear_recent, create_connection, create_group, delete_connection, delete_group, list_groups,
    list_recent, rename_group, reorder_tree, update_connection,
};
use local_fs::{local_home, local_list, local_mkdir, local_remove, local_rename};
use ssh::connect::{ssh_confirm_host_key, ssh_connect, ssh_test_connection};
use ssh::sftp::{
    sftp_cancel_transfer, sftp_canonicalize, sftp_download_items, sftp_list, sftp_mkdir,
    sftp_remove, sftp_rename, sftp_upload_items, Transfers,
};
use ssh::shell::{ssh_disconnect, ssh_resize, ssh_send};
use ssh::{HostKeyState, SshManager};
use storage::{Db, Storage};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

/// 应用入口：数据库固定存放于系统默认应用数据目录（与安装位置无关）。
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 开发构建使用独立 identifier，隔离开发数据（不污染正式数据目录）
    #[allow(unused_mut)]
    let mut context = tauri::generate_context!();
    #[cfg(debug_assertions)]
    {
        context.config_mut().identifier = "top.youngtse.anoterm.dev".to_string();
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // 初始化数据库存储与会话管理器，注册为全局状态
            let data_dir = app.path().app_data_dir()?;
            let storage =
                Storage::open(&data_dir).map_err(|e| std::io::Error::other(e.to_string()))?;
            app.manage(Db(Mutex::new(storage)));
            app.manage(SshManager(Default::default()));
            app.manage(HostKeyState::default());
            app.manage(Transfers::default());

            // 主窗口在代码中创建：配置窗口不支持开启 WebView 剪贴板访问
            WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                .title("Anoterm")
                .inner_size(1325.0, 805.0)
                .center()
                .decorations(false)
                .enable_clipboard_access()
                .build()?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 数据库命令
            list_groups,
            create_group,
            rename_group,
            delete_group,
            create_connection,
            update_connection,
            delete_connection,
            reorder_tree,
            list_recent,
            clear_recent,
            // SSH 命令
            ssh_connect,
            ssh_test_connection,
            ssh_confirm_host_key,
            ssh_send,
            ssh_resize,
            ssh_disconnect,
            // SFTP 命令
            sftp_list,
            sftp_canonicalize,
            sftp_mkdir,
            sftp_rename,
            sftp_remove,
            sftp_upload_items,
            sftp_download_items,
            sftp_cancel_transfer,
            // 本地文件系统命令（双栏模式）
            local_home,
            local_list,
            local_mkdir,
            local_rename,
            local_remove
        ])
        .run(context)
        .expect("error while running tauri application");
}
