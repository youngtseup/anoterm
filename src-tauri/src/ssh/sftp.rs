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
use tokio::sync::oneshot;

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
/// 同名冲突事件名（前端按 transferId 过滤）。
const CONFLICT_EVENT: &str = "sftp-transfer-conflict";

/// 传输项：目录会递归处理；`dest` 为最终目标路径（含名称）。
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

/// 同名冲突处理策略（前端弹窗选择）。
#[derive(Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ConflictChoice {
    Overwrite,
    Skip,
    KeepBoth,
    Cancel,
}

/// 前端对冲突弹窗的回复。
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictReply {
    choice: ConflictChoice,
    apply_all: bool,
}

/// 冲突事件负载。
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ConflictPayload {
    transfer_id: String,
    direction: String,
    source: String,
    dest: String,
    name: String,
    is_dir: bool,
    existing_is_dir: bool,
}

/// 传输登记表：取消标记与等待中的冲突决策。
#[derive(Default)]
pub struct Transfers {
    /// transfer_id -> 取消标记
    flags: Mutex<HashMap<String, Arc<AtomicBool>>>,
    /// transfer_id -> 冲突决策回信端（同一传输同一时刻最多一个冲突）
    pending: Mutex<HashMap<String, oneshot::Sender<ConflictReply>>>,
}

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

/// 一次传输的进度上下文：累计字节、节流发送事件、检查取消与冲突处理。
struct TransferCtx<'a> {
    app: &'a AppHandle,
    transfers: &'a Transfers,
    transfer_id: &'a str,
    direction: &'static str,
    current: String,
    transferred: u64,
    total: u64,
    cancel: Arc<AtomicBool>,
    last_emit: Instant,
    /// 用户勾选「应用于后续冲突」后的策略
    conflict_policy: Option<ConflictChoice>,
}

impl<'a> TransferCtx<'a> {
    fn new(
        app: &'a AppHandle,
        transfers: &'a Transfers,
        transfer_id: &'a str,
        direction: &'static str,
        total: u64,
        cancel: Arc<AtomicBool>,
        conflict_policy: Option<ConflictChoice>,
    ) -> Self {
        Self {
            app,
            transfers,
            transfer_id,
            direction,
            current: String::new(),
            transferred: 0,
            total,
            cancel,
            last_emit: Instant::now(),
            conflict_policy,
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

    /// 询问用户如何处理同名冲突；等待期间轮询取消标记，取消即可中断。
    async fn ask_conflict(
        &mut self,
        source: &str,
        dest: &str,
        is_dir: bool,
        existing_is_dir: bool,
    ) -> Result<ConflictChoice, Stop> {
        if let Some(policy) = self.conflict_policy {
            return Ok(policy);
        }

        let name = display_name(dest);
        self.set_current(&format!("同名冲突: {name}"));

        let (tx, mut rx) = oneshot::channel();
        {
            let mut pending = self
                .transfers
                .pending
                .lock()
                .map_err(|e| Stop::Error(e.to_string()))?;
            pending.insert(self.transfer_id.to_string(), tx);
        }
        let _ = self.app.emit(
            CONFLICT_EVENT,
            ConflictPayload {
                transfer_id: self.transfer_id.to_string(),
                direction: self.direction.to_string(),
                source: source.to_string(),
                dest: dest.to_string(),
                name,
                is_dir,
                existing_is_dir,
            },
        );

        let result = loop {
            if let Err(e) = self.check_cancel() {
                break Err(e);
            }
            match tokio::time::timeout(Duration::from_millis(100), &mut rx).await {
                Ok(Ok(reply)) => {
                    if reply.apply_all && reply.choice != ConflictChoice::Cancel {
                        self.conflict_policy = Some(reply.choice);
                    }
                    break Ok(reply.choice);
                }
                // 回信端被丢弃（传输结束/取消）：按取消处理
                Ok(Err(_)) => break Err(Stop::Cancelled),
                Err(_) => continue,
            }
        };

        if let Ok(mut pending) = self.transfers.pending.lock() {
            pending.remove(self.transfer_id);
        }
        result
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

fn register_transfer(transfers: &Transfers, id: &str) -> Result<Arc<AtomicBool>, String> {
    let mut map = transfers.flags.lock().map_err(|e| e.to_string())?;
    let flag = Arc::new(AtomicBool::new(false));
    map.insert(id.to_string(), flag.clone());
    Ok(flag)
}

fn finish_transfer(transfers: &Transfers, id: &str) {
    if let Ok(mut map) = transfers.flags.lock() {
        map.remove(id);
    }
    if let Ok(mut pending) = transfers.pending.lock() {
        pending.remove(id);
    }
}

/// 取路径最后一段作为显示名称。
fn display_name(path: &str) -> String {
    path.rsplit(|c| c == '/' || c == '\\')
        .next()
        .unwrap_or(path)
        .to_string()
}

/// 为文件/目录名追加序号；文件把序号插在扩展名前。
fn with_suffix(name: &str, index: u32, is_dir: bool) -> String {
    if is_dir {
        return format!("{name} ({index})");
    }
    match name.rfind('.') {
        Some(pos) if pos > 0 => {
            let (stem, ext) = name.split_at(pos);
            format!("{stem} ({index}){ext}")
        }
        _ => format!("{name} ({index})"),
    }
}

/// 远端路径的“保留两者”候选名称。
fn remote_suffixed(path: &str, index: u32, is_dir: bool) -> String {
    let name = with_suffix(&display_name(path), index, is_dir);
    match path.rfind('/') {
        Some(0) => format!("/{name}"),
        Some(pos) => format!("{}/{name}", &path[..pos]),
        None => name,
    }
}

/// 本地路径的“保留两者”候选名称。
fn local_suffixed(path: &Path, index: u32, is_dir: bool) -> PathBuf {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let new_name = with_suffix(&name, index, is_dir);
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.join(new_name),
        _ => PathBuf::from(new_name),
    }
}

/// 目标是否已存在，返回（是否目录, 是否符号链接）。
async fn remote_existing(sftp: &SftpSession, path: &str) -> Option<(bool, bool)> {
    sftp.symlink_metadata(path)
        .await
        .ok()
        .map(|md| (md.is_dir(), md.is_symlink()))
}

/// 目标是否已存在，返回（是否目录, 是否符号链接）。
async fn local_existing(path: &Path) -> Option<(bool, bool)> {
    tokio::fs::symlink_metadata(path)
        .await
        .ok()
        .map(|md| (md.is_dir(), md.file_type().is_symlink()))
}

async fn unique_remote_dest(sftp: &SftpSession, dest: &str, is_dir: bool) -> Result<String, Stop> {
    for index in 1..=9999u32 {
        let candidate = remote_suffixed(dest, index, is_dir);
        if sftp.symlink_metadata(candidate.as_str()).await.is_err() {
            return Ok(candidate);
        }
    }
    Err(Stop::Error("无法生成不重名的目标名称".to_string()))
}

async fn unique_local_dest(dest: &Path, is_dir: bool) -> Result<PathBuf, Stop> {
    for index in 1..=9999u32 {
        let candidate = local_suffixed(dest, index, is_dir);
        if tokio::fs::symlink_metadata(&candidate).await.is_err() {
            return Ok(candidate);
        }
    }
    Err(Stop::Error("无法生成不重名的目标名称".to_string()))
}

/// 递归删除远端文件或目录（符号链接只删除链接本身）。
async fn remove_remote_recursive(sftp: &SftpSession, path: &str) -> Result<(), Stop> {
    let md = sftp
        .symlink_metadata(path)
        .await
        .map_err(|e| Stop::Error(format!("读取远端信息失败: {e}")))?;
    if !md.is_dir() {
        return sftp
            .remove_file(path)
            .await
            .map_err(|e| Stop::Error(format!("删除远端文件失败: {e}")));
    }

    let mut stack = vec![(path.to_string(), false)];
    while let Some((current, visited)) = stack.pop() {
        if visited {
            sftp.remove_dir(current.as_str())
                .await
                .map_err(|e| Stop::Error(format!("删除远端目录失败: {e}")))?;
            continue;
        }
        stack.push((current.clone(), true));
        let entries = sftp
            .read_dir(current.as_str())
            .await
            .map_err(|e| Stop::Error(format!("读取远端目录失败: {e}")))?;
        for entry in entries {
            let child = join_remote(&current, &entry.file_name());
            match sftp.symlink_metadata(child.as_str()).await {
                Ok(md) if md.is_dir() => stack.push((child, false)),
                Ok(_) => {
                    sftp.remove_file(child.as_str())
                        .await
                        .map_err(|e| Stop::Error(format!("删除远端文件失败: {e}")))?;
                }
                Err(e) => return Err(Stop::Error(format!("读取远端信息失败: {e}"))),
            }
        }
    }
    Ok(())
}

/// 递归删除本地文件或目录（符号链接只删除链接本身）。
async fn remove_local_recursive(path: &Path) -> Result<(), Stop> {
    let md = tokio::fs::symlink_metadata(path)
        .await
        .map_err(|e| Stop::Error(format!("读取本地信息失败: {e}")))?;
    if md.is_dir() {
        tokio::fs::remove_dir_all(path)
            .await
            .map_err(|e| Stop::Error(format!("删除本地目录失败: {e}")))
    } else {
        tokio::fs::remove_file(path)
            .await
            .map_err(|e| Stop::Error(format!("删除本地文件失败: {e}")))
    }
}

/// 冲突处理结果。
enum ConflictAction {
    /// 使用（可能已重命名的）目标路径继续传输
    Proceed(String),
    /// 跳过当前条目
    Skip,
}

/// 目标已存在时询问用户并按选择处理；不存在则直接放行。
async fn resolve_conflict(
    ctx: &mut TransferCtx<'_>,
    sftp: &SftpSession,
    source: &str,
    dest: &str,
    is_dir: bool,
    existing: Option<(bool, bool)>,
) -> Result<ConflictAction, Stop> {
    let Some((existing_is_dir, existing_is_symlink)) = existing else {
        return Ok(ConflictAction::Proceed(dest.to_string()));
    };
    let choice = ctx
        .ask_conflict(source, dest, is_dir, existing_is_dir)
        .await?;
    match choice {
        ConflictChoice::Cancel => Err(Stop::Cancelled),
        ConflictChoice::Skip => Ok(ConflictAction::Skip),
        ConflictChoice::KeepBoth => {
            let unique = if ctx.direction == "upload" {
                unique_remote_dest(sftp, dest, is_dir).await?
            } else {
                unique_local_dest(Path::new(dest), is_dir)
                    .await?
                    .to_string_lossy()
                    .into_owned()
            };
            Ok(ConflictAction::Proceed(unique))
        }
        ConflictChoice::Overwrite => {
            // 普通文件覆盖文件无需删除（写入时截断）；目录/符号链接先删除目标
            let must_remove = existing_is_dir || existing_is_symlink || is_dir;
            if ctx.direction == "upload" {
                if must_remove {
                    remove_remote_recursive(sftp, dest).await?;
                }
            } else if must_remove {
                remove_local_recursive(Path::new(dest)).await?;
            }
            Ok(ConflictAction::Proceed(dest.to_string()))
        }
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

/// 逐个处理上传条目（目录用栈迭代；目录的 dest 为最终目标目录）。
async fn upload_items(
    sftp: &SftpSession,
    items: &[TransferItem],
    ctx: &mut TransferCtx<'_>,
) -> Result<(), Stop> {
    for item in items {
        ctx.check_cancel()?;
        let existing = remote_existing(sftp, &item.dest).await;
        let dest =
            match resolve_conflict(ctx, sftp, &item.source, &item.dest, item.is_dir, existing)
                .await?
            {
                ConflictAction::Skip => continue,
                ConflictAction::Proceed(dest) => dest,
            };
        let local = Path::new(&item.source);
        if !item.is_dir {
            upload_file(sftp, local, &dest, ctx).await?;
            continue;
        }
        // 目录已按“整体替换/重命名”处理，内部不会再遇到既有目标
        let mut dirs = vec![(local.to_path_buf(), dest)];
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

/// 逐个处理下载条目（目录用栈迭代；目录的 dest 为最终目标目录）。
async fn download_items(
    sftp: &SftpSession,
    items: &[TransferItem],
    ctx: &mut TransferCtx<'_>,
) -> Result<(), Stop> {
    for item in items {
        ctx.check_cancel()?;
        let existing = local_existing(Path::new(&item.dest)).await;
        let dest =
            match resolve_conflict(ctx, sftp, &item.source, &item.dest, item.is_dir, existing)
                .await?
            {
                ConflictAction::Skip => continue,
                ConflictAction::Proceed(dest) => dest,
            };
        if !item.is_dir {
            download_file(sftp, &item.source, Path::new(&dest), ctx).await?;
            continue;
        }
        // 目录已按“整体替换/重命名”处理，内部不会再遇到既有目标
        let mut dirs = vec![(item.source.clone(), PathBuf::from(dest))];
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
    conflict_policy: Option<ConflictChoice>,
) -> Result<TransferOutcome, String> {
    let sftp = get_sftp(&state, &conn_id)?;
    let cancel = register_transfer(transfers.inner(), &transfer_id)?;
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
    let mut ctx = TransferCtx::new(
        &app,
        transfers.inner(),
        &transfer_id,
        "upload",
        total,
        cancel,
        conflict_policy,
    );
    let result = upload_items(&sftp, &items, &mut ctx).await;
    if result.is_ok() {
        ctx.emit();
    }
    finish_transfer(transfers.inner(), &transfer_id);
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
    conflict_policy: Option<ConflictChoice>,
) -> Result<TransferOutcome, String> {
    let sftp = get_sftp(&state, &conn_id)?;
    let cancel = register_transfer(transfers.inner(), &transfer_id)?;
    let total = remote_total_size(&sftp, &items, &cancel).await;
    let mut ctx = TransferCtx::new(
        &app,
        transfers.inner(),
        &transfer_id,
        "download",
        total,
        cancel,
        conflict_policy,
    );
    let result = download_items(&sftp, &items, &mut ctx).await;
    if result.is_ok() {
        ctx.emit();
    }
    finish_transfer(transfers.inner(), &transfer_id);
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
    let registry = transfers.inner();
    if let Ok(map) = registry.flags.lock() {
        if let Some(flag) = map.get(&transfer_id) {
            flag.store(true, Ordering::Relaxed);
        }
    }
    // 若正在等待冲突决策，立即唤醒并中止
    if let Ok(mut pending) = registry.pending.lock() {
        if let Some(tx) = pending.remove(&transfer_id) {
            let _ = tx.send(ConflictReply {
                choice: ConflictChoice::Cancel,
                apply_all: false,
            });
        }
    }
    Ok(())
}

/// 前端对同名冲突弹窗的回复（选择 + 是否应用于后续冲突）。
#[tauri::command]
pub fn sftp_resolve_conflict(
    transfers: State<'_, Transfers>,
    transfer_id: String,
    choice: ConflictChoice,
    apply_all: bool,
) -> Result<(), String> {
    let sender = transfers
        .inner()
        .pending
        .lock()
        .map_err(|e| e.to_string())?
        .remove(&transfer_id);
    if let Some(tx) = sender {
        let _ = tx.send(ConflictReply { choice, apply_all });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suffix_file_names() {
        assert_eq!(with_suffix("a.txt", 1, false), "a (1).txt");
        assert_eq!(
            with_suffix("archive.tar.gz", 2, false),
            "archive.tar (2).gz"
        );
        assert_eq!(with_suffix(".bashrc", 1, false), ".bashrc (1)");
        assert_eq!(with_suffix("README", 3, false), "README (3)");
        assert_eq!(with_suffix("folder.name", 1, true), "folder.name (1)");
    }

    #[test]
    fn remote_suffix_keeps_parent() {
        assert_eq!(remote_suffixed("/a/b/c.txt", 1, false), "/a/b/c (1).txt");
        assert_eq!(remote_suffixed("/c.txt", 1, false), "/c (1).txt");
        assert_eq!(remote_suffixed("c.txt", 1, false), "c (1).txt");
        assert_eq!(remote_suffixed("/a/dir", 2, true), "/a/dir (2)");
    }

    #[test]
    fn display_name_handles_both_separators() {
        assert_eq!(display_name("/a/b/c.txt"), "c.txt");
        assert_eq!(display_name("C:\\a\\b"), "b");
    }
}
