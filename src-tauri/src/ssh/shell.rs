//! SSH 终端通道：输出数据泵与终端控制命令。
//!
//! `pump` 是一个后台任务，持续读取终端通道的远端输出并
//! 以 `ssh-output` 事件转发给前端；通道关闭时发送 `ssh-closed` 事件。

use std::sync::Arc;

use base64::Engine as _;
use russh::client::Msg;
use russh::{ChannelMsg, ChannelReadHalf, ChannelWriteHalf};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::ssh::SshManager;

/// 终端通道的写入端类型别名。
pub(crate) type ChannelWrite = ChannelWriteHalf<Msg>;

/// 转发给前端的事件负载。
#[derive(Clone, Serialize)]
pub(crate) struct SshOutput {
    /// 会话 id（tabId），前端据此区分事件归属
    pub(crate) conn_id: String,
    /// base64 编码的原始字节（终端输出）
    pub(crate) data: String,
}

/// 将原始字节编码为 base64 字符串。
fn b64(data: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(data)
}

/// 终端输出泵：读取远端输出 → 发事件；通道关闭 → 清理会话并发 `ssh-closed`。
///
/// `write` 参数用于在通道结束时确认当前会话仍持有该通道，避免误删新建的会话。
pub(crate) async fn pump(
    app: AppHandle,
    manager: SshManager,
    conn_id: String,
    mut read: ChannelReadHalf,
    write: Arc<tokio::sync::Mutex<ChannelWrite>>,
) {
    loop {
        let Some(msg) = read.wait().await else { break };
        match msg {
            // 普通输出与扩展输出（如 stderr）都转发给前端
            ChannelMsg::Data { ref data } | ChannelMsg::ExtendedData { ref data, .. } => {
                let event = SshOutput {
                    conn_id: conn_id.clone(),
                    data: b64(data),
                };
                let _ = app.emit("ssh-output", event);
            }
            ChannelMsg::Close | ChannelMsg::ExitStatus { .. } => break,
            _ => {}
        }
    }

    // 通道结束后清理会话（仅当该会话仍指向此通道时）
    if let Ok(mut map) = manager.0.lock() {
        if let Some(c) = map.get(&conn_id) {
            if c.shell
                .as_ref()
                .map(|s| Arc::ptr_eq(&s.write, &write))
                .unwrap_or(false)
            {
                map.remove(&conn_id);
            }
        }
    }

    let _ = app.emit(
        "ssh-closed",
        SshOutput {
            conn_id: conn_id.clone(),
            data: String::new(),
        },
    );
}

/// 向终端写入数据（按键输入 / 粘贴内容）。
#[tauri::command]
pub async fn ssh_send(
    state: State<'_, SshManager>,
    conn_id: String,
    data: String,
) -> Result<(), String> {
    let write = {
        let map = state.0.lock().map_err(|e| e.to_string())?;
        let c = map.get(&conn_id).ok_or("会话不存在")?;
        c.shell.as_ref().ok_or("无 SSH 终端")?.write.clone()
    };
    let w = write.lock().await;
    w.data_bytes(data.into_bytes())
        .await
        .map_err(|e| e.to_string())
}

/// 通知远端调整终端窗口尺寸。
#[tauri::command]
pub async fn ssh_resize(
    state: State<'_, SshManager>,
    conn_id: String,
    cols: u32,
    rows: u32,
) -> Result<(), String> {
    let write = {
        let map = state.0.lock().map_err(|e| e.to_string())?;
        let c = map.get(&conn_id).ok_or("会话不存在")?;
        c.shell.as_ref().ok_or("无 SSH 终端")?.write.clone()
    };
    let w = write.lock().await;
    let _ = w.window_change(cols, rows, 0, 0).await;
    Ok(())
}

/// 断开会话：移除状态并中止数据泵任务。
#[tauri::command]
pub async fn ssh_disconnect(state: State<'_, SshManager>, conn_id: String) -> Result<(), String> {
    let removed = { state.0.lock().map_err(|e| e.to_string())?.remove(&conn_id) };
    if let Some(c) = removed {
        if let Some(shell) = c.shell {
            shell.task.abort();
        }
    }
    Ok(())
}
