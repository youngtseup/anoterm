//! SSH / SFTP 会话管理模块。
//!
//! 负责维护每个 tab 会话（`SshManager`）的状态，并拆分为三个子模块：
//! - [`connect`]：建立 SSH 连接（认证、打开 shell/SFTP 通道）
//! - [`shell`]：SSH 终端的数据泵（事件转发）与终端命令
//! - [`sftp`]：SFTP 文件操作命令

pub mod connect;
pub mod sftp;
pub mod shell;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use russh_sftp::client::SftpSession;
use tauri::async_runtime::JoinHandle;
use tokio::sync::oneshot;
use tokio::sync::Mutex as AsyncMutex;

use crate::ssh::shell::ChannelWrite;

/// 单个 SSH 会话的终端通道句柄。
///
/// - `write`：终端写入端（发送按键输入、调整窗口大小）
/// - `task`：后台数据泵任务（把远端输出转发为前端事件），关闭会话时用于中止
pub(crate) struct SshHandle {
    pub(crate) write: Arc<AsyncMutex<ChannelWrite>>,
    pub(crate) task: JoinHandle<()>,
}

/// 一个 tab 会话的全部状态：可选 shell 通道与 SFTP 会话。
pub(crate) struct ConnState {
    pub(crate) shell: Option<SshHandle>,
    pub(crate) sftp: Option<Arc<SftpSession>>,
}

/// 全局会话管理器：以 `conn_id`（前端 tabId）为键维护所有活跃会话。
#[derive(Default)]
pub struct SshManager(pub Arc<Mutex<HashMap<String, ConnState>>>);

/// 主机密钥确认状态：首次连接或指纹变更时，后端等待前端用户决策。
#[derive(Default)]
pub struct HostKeyState {
    /// conn_id → 用户决策发送端（接受 / 拒绝）
    pub(crate) pending: Mutex<HashMap<String, oneshot::Sender<bool>>>,
    /// conn_id → 拒绝原因（供连接错误信息映射）
    pub(crate) rejections: Mutex<HashMap<String, String>>,
}
