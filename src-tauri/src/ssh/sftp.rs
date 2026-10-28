//! SFTP 文件操作命令：浏览、新建、重命名、删除与带进度/取消的上传下载。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use russh_sftp::client::SftpSession;
use russh_sftp::protocol::OpenFlags;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::ssh::SshManager;

/// 目录项（列表返回给前端，字段为 camelCase）。
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SftpEntry {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) is_dir: bool,
    pub(crate) is_symlink: bool,
    pub(crate) size: Option<u64>,
    pub(crate) mtime: Option<u32>,
    pub(crate) permissions: Option<u32>,
}

/// 从会话中取出 SFTP 会话句柄。
fn get_sftp(state: &State<'_, SshManager>, conn_id: &str) -> Result<Arc<SftpSession>, String> {
    let map = state.0.lock().map_err(|e| e.to_string())?;
    let c = map.get(conn_id).ok_or("会话不存在")?;
    c.sftp
        .clone()
        .ok_or_else(|| "SFTP 未初始化，请使用 SSH&SFTP 或 SFTP 类型连接".to_string())
}

/// 列出目录内容（目录排前，按名称不区分大小写排序）。
#[tauri::command]
pub async fn sftp_list(
    state: State<'_, SshManager>,
    conn_id: String,
    path: String,
) -> Result<Vec<SftpEntry>, String> {
    let sftp = get_sftp(&state, &conn_id)?;
    let entries = sftp
        .read_dir(path.as_str())
        .await
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for entry in entries {
        let name = entry.file_name();
        let entry_path = entry.path();
        let md = entry.metadata();
        // readdir 常不带完整类型/权限，导致 is_dir 判断不可靠；对歧义项（无权限或符号链接）做权威 stat
        let resolved = if md.permissions.is_none() || md.is_symlink() {
            sftp.metadata(entry_path.as_str())
                .await
                .unwrap_or_else(|_| md.clone())
        } else {
            md
        };
        out.push(SftpEntry {
            name,
            path: entry_path,
            is_dir: resolved.is_dir(),
            is_symlink: resolved.is_symlink(),
            size: resolved.size,
            mtime: resolved.mtime,
            permissions: resolved.permissions,
        });
    }
    out.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(out)
}

/// 解析路径的规范形式（解析符号链接、相对路径）。
#[tauri::command]
pub async fn sftp_canonicalize(
    state: State<'_, SshManager>,
    conn_id: String,
    path: String,
) -> Result<String, String> {
    let sftp = get_sftp(&state, &conn_id)?;
    sftp.canonicalize(path.as_str())
        .await
        .map_err(|e| e.to_string())
}

/// 创建远端目录。
#[tauri::command]
pub async fn sftp_mkdir(
    state: State<'_, SshManager>,
    conn_id: String,
    path: String,
) -> Result<(), String> {
    let sftp = get_sftp(&state, &conn_id)?;
    sftp.create_dir(path.as_str())
        .await
        .map_err(|e| e.to_string())
}

/// 重命名/移动远端文件或目录。
#[tauri::command]
pub async fn sftp_rename(
    state: State<'_, SshManager>,
    conn_id: String,
    old_path: String,
    new_path: String,
) -> Result<(), String> {
    let sftp = get_sftp(&state, &conn_id)?;
    sftp.rename(old_path.as_str(), new_path.as_str())
        .await
        .map_err(|e| e.to_string())
}

/// 删除远端文件或目录（按 `is_dir` 选择删除方式）。
#[tauri::command]
pub async fn sftp_remove(
    state: State<'_, SshManager>,
    conn_id: String,
    path: String,
    is_dir: bool,
) -> Result<(), String> {
    let sftp = get_sftp(&state, &conn_id)?;
    if is_dir {
        sftp.remove_dir(path.as_str())
            .await
            .map_err(|e| e.to_string())
    } else {
        sftp.remove_file(path.as_str())
            .await
            .map_err(|e| e.to_string())
    }
}

/// 拼接远端路径（处理根目录与尾部斜杠）。
fn join_remote(parent: &str, name: &str) -> String {
    if parent == "/" || parent.is_empty() {
        format!("/{name}")
    } else {
        format!("{}/{}", parent.trim_end_matches('/'), name)
    }
}

/// 确保远端目录存在（已存在且为目录时忽略）。
async fn ensure_remote_dir(sftp: &SftpSession, path: &str) -> Result<(), String> {
    if sftp
        .metadata(path)
        .await
        .map(|m| m.is_dir())
        .unwrap_or(false)
    {
        return Ok(());
    }
    sftp.create_dir(path)
        .await
        .map_err(|e| format!("创建远端目录失败: {e}"))
}

/// 分块大小（读写与取消检查的粒度）。
const CHUNK_SIZE: usize = 256 * 1024;
/// 进度事件最小间隔，避免高频 IPC。
const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);
/// 进度事件名（前端按 transferId 过滤）。
const PROGRESS_EVENT: &str = "sftp-transfer-progress";

/// 传输项：目录会递归处理；目录的 `dest` 表示要填充的目标目录。
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferItem {
    pub source: String,
    pub dest: String,
    pub is_dir: bool,
}

/// 传输结果：取消属于正常结束，不算错误。
#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TransferOutcome {
    Done,
    Cancelled,
}

/// 进度事件负载。
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TransferProgress {
    transfer_id: String,
    current: String,
    transferred: u64,
    total: u64,
}

/// 取消登记表：transfer_id -> 取消标记。
#[derive(Default)]
pub struct Transfers(pub Mutex<HashMap<String, Arc<AtomicBool>>>);

/// 内部停止原因。
enum Stop {
    Cancelled,
    Error(String),
}

impl From<String> for Stop {
    fn from(value: String) -> Self {
        Stop::Error(value)
    }
}

/// 一次传输的进度上下文：累计字节、节流发送事件、检查取消。
struct TransferCtx<'a> {
    app: &'a AppHandle,
    transfer_id: &'a str,
    current: String,
    transferred: u64,
    total: u64,
    cancel: Arc<AtomicBool>,
    last_emit: Instant,
}

impl<'a> TransferCtx<'a> {
    fn new(app: &'a AppHandle, transfer_id: &'a str, total: u64, cancel: Arc<AtomicBool>) -> Self {
        Self {
            app,
            transfer_id,
            current: String::new(),
            transferred: 0,
            total,
            cancel,
            last_emit: Instant::now(),
        }
    }

    fn emit(&self) {
        let _ = self.app.emit(
            PROGRESS_EVENT,
            TransferProgress {
                transfer_id: self.transfer_id.to_string(),
                current: self.current.clone(),
                transferred: self.transferred,
                total: self.total,
            },
        );
    }

    fn set_current(&mut self, name: &str) {
        self.current = name.to_string();
        self.emit();
    }

    fn check_cancel(&self) -> Result<(), Stop> {
        if self.cancel.load(Ordering::Relaxed) {
            Err(Stop::Cancelled)
        } else {
            Ok(())
        }
    }

    /// 分块复制并节流上报进度；取消时立即返回 Cancelled。
    async fn copy<R, W>(&mut self, reader: &mut R, writer: &mut W) -> Result<(), Stop>
    where
        R: AsyncRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        let mut buf = vec![0u8; CHUNK_SIZE];
        loop {
            self.check_cancel()?;
            let n = reader
                .read(&mut buf)
                .await
                .map_err(|e| Stop::Error(format!("读取失败: {e}")))?;
            if n == 0 {
                return Ok(());
            }
            writer
                .write_all(&buf[..n])
                .await
                .map_err(|e| Stop::Error(format!("写入失败: {e}")))?;
            self.transferred += n as u64;
            if self.last_emit.elapsed() >= PROGRESS_INTERVAL {
                self.last_emit = Instant::now();
                self.emit();
            }
        }
    }
}

fn register_transfer(
    transfers: &State<'_, Transfers>,
    id: &str,
) -> Result<Arc<AtomicBool>, String> {
    let mut map = transfers.0.lock().map_err(|e| e.to_string())?;
    let flag = Arc::new(AtomicBool::new(false));
    map.insert(id.to_string(), flag.clone());
    Ok(flag)
}

fn finish_transfer(transfers: &State<'_, Transfers>, id: &str) {
    if let Ok(mut map) = transfers.0.lock() {
        map.remove(id);
    }
}

/// 本地条目总字节数（目录递归；不可读条目按 0 计）。
fn local_size(path: &Path, cancel: &AtomicBool) -> u64 {
    if cancel.load(Ordering::Relaxed) {
        return 0;
    }
    let md = match std::fs::metadata(path) {
        Ok(md) => md,
        Err(_) => return 0,
    };
    if !md.is_dir() {
        return md.len();
    }
    let mut total = 0;
    if let Ok(rd) = std::fs::read_dir(path) {
        for entry in rd.flatten() {
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            total += local_size(&entry.path(), cancel);
        }
    }
    total
}

async fn remote_total_size(sftp: &SftpSession, items: &[TransferItem], cancel: &AtomicBool) -> u64 {
    let mut total = 0;
    for item in items {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        if item.is_dir {
            total += remote_dir_size(sftp, &item.source, cancel).await;
        } else if let Ok(md) = sftp.metadata(item.source.as_str()).await {
            total += md.size.unwrap_or(0);
        }
    }
    total
}

async fn remote_dir_size(sftp: &SftpSession, dir: &str, cancel: &AtomicBool) -> u64 {
    let mut total = 0;
    let mut stack = vec![dir.to_string()];
    while let Some(current) = stack.pop() {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        let entries = match sftp.read_dir(current.as_str()).await {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries {
            let path = join_remote(&current, &entry.file_name());
            match sftp.metadata(path.as_str()).await {
                Ok(md) if md.is_dir() => stack.push(path),
                Ok(md) => total += md.size.unwrap_or(0),
                Err(_) => {}
            }
        }
    }
    total
}

/// 上传单个本地文件（覆盖写入）；失败/取消时清理远端半成品。
async fn upload_file(
    sftp: &SftpSession,
    local: &Path,
    remote: &str,
    ctx: &mut TransferCtx<'_>,
) -> Result<(), Stop> {
    let name = local
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    ctx.set_current(&name);
    let mut src = tokio::fs::File::open(local)
        .await
        .map_err(|e| Stop::Error(format!("打开本地文件失败: {e}")))?;
    let mut dst = sftp
        .open_with_flags(
            remote,
            OpenFlags::WRITE | OpenFlags::CREATE | OpenFlags::TRUNCATE,
        )
        .await
        .map_err(|e| Stop::Error(format!("打开远端文件失败: {e}")))?;
    let copied = async {
        ctx.copy(&mut src, &mut dst).await?;
        dst.flush()
            .await
            .map_err(|e| Stop::Error(format!("写入失败: {e}")))?;
        dst.shutdown()
            .await
            .map_err(|e| Stop::Error(format!("关闭失败: {e}")))?;
        Ok::<(), Stop>(())
    }
    .await;
    if let Err(e) = copied {
        drop(dst);
        let _ = sftp.remove_file(remote).await;
        return Err(e);
    }
    Ok(())
}

/// 下载单个远端文件；失败/取消时清理本地半成品。
async fn download_file(
    sftp: &SftpSession,
    remote: &str,
    local: &Path,
    ctx: &mut TransferCtx<'_>,
) -> Result<(), Stop> {
    let name = local
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    ctx.set_current(&name);
    let mut src = sftp
        .open_with_flags(remote, OpenFlags::READ)
        .await
        .map_err(|e| Stop::Error(format!("打开远端文件失败: {e}")))?;
    let mut dst = tokio::fs::File::create(local)
        .await
        .map_err(|e| Stop::Error(format!("创建本地文件失败: {e}")))?;
    let copied = async {
        ctx.copy(&mut src, &mut dst).await?;
        dst.flush()
            .await
            .map_err(|e| Stop::Error(format!("写入失败: {e}")))?;
        dst.shutdown()
            .await
            .map_err(|e| Stop::Error(format!("关闭失败: {e}")))?;
        Ok::<(), Stop>(())
    }
    .await;
    if let Err(e) = copied {
        drop(dst);
        let _ = tokio::fs::remove_file(local).await;
        return Err(e);
    }
    Ok(())
}

/// 逐个处理上传条目（目录用栈迭代；目录的 dest 为要填充的目标目录）。
async fn upload_items(
    sftp: &SftpSession,
    items: &[TransferItem],
    ctx: &mut TransferCtx<'_>,
) -> Result<(), Stop> {
    for item in items {
        ctx.check_cancel()?;
        let local = Path::new(&item.source);
        if !item.is_dir {
            upload_file(sftp, local, &item.dest, ctx).await?;
            continue;
        }
        let mut dirs = vec![(local.to_path_buf(), item.dest.clone())];
        while let Some((dir, remote)) = dirs.pop() {
            ensure_remote_dir(sftp, &remote).await?;
            ctx.check_cancel()?;
            let mut rd = tokio::fs::read_dir(&dir)
                .await
                .map_err(|e| Stop::Error(format!("读取本地目录失败: {e}")))?;
            while let Some(ent) = rd
                .next_entry()
                .await
                .map_err(|e| Stop::Error(format!("读取本地目录失败: {e}")))?
            {
                let path = ent.path();
                let remote_child = join_remote(&remote, &ent.file_name().to_string_lossy());
                let md = tokio::fs::metadata(&path)
                    .await
                    .map_err(|e| Stop::Error(format!("读取本地信息失败: {e}")))?;
                if md.is_dir() {
                    dirs.push((path, remote_child));
                } else {
                    upload_file(sftp, &path, &remote_child, ctx).await?;
                }
            }
        }
    }
    Ok(())
}

/// 逐个处理下载条目（目录用栈迭代；目录的 dest 为要填充的目标目录）。
async fn download_items(
    sftp: &SftpSession,
    items: &[TransferItem],
    ctx: &mut TransferCtx<'_>,
) -> Result<(), Stop> {
    for item in items {
        ctx.check_cancel()?;
        if !item.is_dir {
            download_file(sftp, &item.source, Path::new(&item.dest), ctx).await?;
            continue;
        }
        let mut dirs = vec![(item.source.clone(), PathBuf::from(&item.dest))];
        while let Some((remote, local)) = dirs.pop() {
            tokio::fs::create_dir_all(&local)
                .await
                .map_err(|e| Stop::Error(format!("创建本地目录失败: {e}")))?;
            ctx.check_cancel()?;
            let entries = sftp
                .read_dir(remote.as_str())
                .await
                .map_err(|e| Stop::Error(e.to_string()))?;
            for entry in entries {
                let name = entry.file_name();
                let remote_child = join_remote(&remote, &name);
                let local_child = local.join(&name);
                let md = sftp
                    .metadata(remote_child.as_str())
                    .await
                    .map_err(|e| Stop::Error(e.to_string()))?;
                if md.is_dir() {
                    dirs.push((remote_child, local_child));
                } else {
                    download_file(sftp, &remote_child, &local_child, ctx).await?;
                }
            }
        }
    }
    Ok(())
}

/// 批量上传：本地 → 远端，带进度事件与取消。
#[tauri::command]
pub async fn sftp_upload_items(
    app: AppHandle,
    transfers: State<'_, Transfers>,
    state: State<'_, SshManager>,
    transfer_id: String,
    conn_id: String,
    items: Vec<TransferItem>,
) -> Result<TransferOutcome, String> {
    let sftp = get_sftp(&state, &conn_id)?;
    let cancel = register_transfer(&transfers, &transfer_id)?;
    let sources: Vec<String> = items.iter().map(|i| i.source.clone()).collect();
    let scan_flag = cancel.clone();
    let total = tokio::task::spawn_blocking(move || {
        sources
            .iter()
            .map(|p| local_size(Path::new(p), &scan_flag))
            .sum::<u64>()
    })
    .await
    .unwrap_or(0);
    let mut ctx = TransferCtx::new(&app, &transfer_id, total, cancel);
    let result = upload_items(&sftp, &items, &mut ctx).await;
    if result.is_ok() {
        ctx.emit();
    }
    finish_transfer(&transfers, &transfer_id);
    match result {
        Ok(()) => Ok(TransferOutcome::Done),
        Err(Stop::Cancelled) => Ok(TransferOutcome::Cancelled),
        Err(Stop::Error(e)) => Err(e),
    }
}

/// 批量下载：远端 → 本地，带进度事件与取消。
#[tauri::command]
pub async fn sftp_download_items(
    app: AppHandle,
    transfers: State<'_, Transfers>,
    state: State<'_, SshManager>,
    transfer_id: String,
    conn_id: String,
    items: Vec<TransferItem>,
) -> Result<TransferOutcome, String> {
    let sftp = get_sftp(&state, &conn_id)?;
    let cancel = register_transfer(&transfers, &transfer_id)?;
    let total = remote_total_size(&sftp, &items, &cancel).await;
    let mut ctx = TransferCtx::new(&app, &transfer_id, total, cancel);
    let result = download_items(&sftp, &items, &mut ctx).await;
    if result.is_ok() {
        ctx.emit();
    }
    finish_transfer(&transfers, &transfer_id);
    match result {
        Ok(()) => Ok(TransferOutcome::Done),
        Err(Stop::Cancelled) => Ok(TransferOutcome::Cancelled),
        Err(Stop::Error(e)) => Err(e),
    }
}

/// 请求取消指定传输（标记后由传输循环自行退出并清理半成品）。
#[tauri::command]
pub fn sftp_cancel_transfer(
    transfers: State<'_, Transfers>,
    transfer_id: String,
) -> Result<(), String> {
    let map = transfers.0.lock().map_err(|e| e.to_string())?;
    if let Some(flag) = map.get(&transfer_id) {
        flag.store(true, Ordering::Relaxed);
    }
    Ok(())
}
