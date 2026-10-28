//! SSH 连接建立：认证、按会话类型打开 shell / SFTP 通道并登记会话。

use std::sync::Arc;
use std::time::Duration;

use russh::client;
use russh::keys::{
    load_secret_key, HashAlg, PrivateKeyWithHashAlg, PublicKey, PublicKeyOrCertificate,
};
use russh_sftp::client::SftpSession;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use uuid::Uuid;

use crate::crypto::decrypt_password;
use crate::ssh::shell::pump;
use crate::ssh::{ConnState, HostKeyState, SshHandle, SshManager};
use crate::storage::Db;

/// 主机密钥确认事件负载（首次连接 / 指纹变更）。
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct HostKeyVerifyPayload {
    conn_id: String,
    host: String,
    port: u16,
    key_type: String,
    fingerprint: String,
    public_key: String,
    /// 是否与已保存记录不一致（变更警告）
    changed: bool,
    old_fingerprint: Option<String>,
}

/// 提取服务器公钥的类型、指纹与 OpenSSH 文本（证书取内层公钥）。
fn public_key_info(key: &PublicKeyOrCertificate) -> (String, String, String) {
    let pk: PublicKey = match key {
        PublicKeyOrCertificate::PublicKey { key, .. } => key.clone(),
        PublicKeyOrCertificate::Certificate(cert) => PublicKey::new(cert.public_key().clone(), ""),
    };
    (
        pk.algorithm().as_str().to_string(),
        pk.fingerprint(HashAlg::Sha256).to_string(),
        pk.to_openssh().unwrap_or_default(),
    )
}

/// russh 客户端回调：TOFU 主机密钥校验（首次确认保存，变更时警告确认）。
struct SshHandler {
    app: AppHandle,
    conn_id: String,
    host: String,
    port: u16,
}

impl SshHandler {
    fn new(app: AppHandle, conn_id: String, host: String, port: u16) -> Self {
        Self {
            app,
            conn_id,
            host,
            port,
        }
    }

    /// 记录拒绝原因，供连接错误信息映射。
    fn set_rejection(&self, reason: &str) {
        if let Ok(mut map) = self.app.state::<HostKeyState>().rejections.lock() {
            map.insert(self.conn_id.clone(), reason.to_string());
        }
    }
}

impl client::Handler for SshHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &PublicKeyOrCertificate,
    ) -> Result<bool, Self::Error> {
        let (key_type, fingerprint, public_key) = public_key_info(server_public_key);

        // 查询已保存的主机公钥：完全匹配则直接信任
        let known = match self.app.state::<Db>().0.lock() {
            Ok(storage) => storage
                .get_known_host(&self.host, self.port, &key_type)
                .ok()
                .flatten(),
            Err(_) => None,
        };
        let (changed, old_fingerprint) = match &known {
            Some((stored_key, _)) if stored_key == &public_key => return Ok(true),
            Some((_, stored_fp)) => (true, Some(stored_fp.clone())),
            None => (false, None),
        };

        // 未知或已变更：发事件等待前端用户确认（ssh_confirm_host_key）
        let (tx, rx) = tokio::sync::oneshot::channel::<bool>();
        match self.app.state::<HostKeyState>().pending.lock() {
            Ok(mut map) => {
                map.insert(self.conn_id.clone(), tx);
            }
            Err(_) => return Ok(false),
        }
        let _ = self.app.emit(
            "ssh-host-key-verify",
            HostKeyVerifyPayload {
                conn_id: self.conn_id.clone(),
                host: self.host.clone(),
                port: self.port,
                key_type: key_type.clone(),
                fingerprint: fingerprint.clone(),
                public_key: public_key.clone(),
                changed,
                old_fingerprint,
            },
        );

        // 等待用户决策（超时视为取消；外层连接超时应大于此值）
        let decision = tokio::time::timeout(Duration::from_secs(110), rx).await;
        if let Ok(mut map) = self.app.state::<HostKeyState>().pending.lock() {
            map.remove(&self.conn_id);
        }

        match decision {
            Ok(Ok(true)) => {
                // 用户接受：保存/更新主机公钥（best-effort）
                if let Ok(storage) = self.app.state::<Db>().0.lock() {
                    let _ = storage.upsert_known_host(
                        &self.host,
                        self.port,
                        &key_type,
                        &public_key,
                        &fingerprint,
                    );
                }
                Ok(true)
            }
            Ok(Ok(false)) => {
                self.set_rejection("已取消主机密钥确认，连接已终止");
                Ok(false)
            }
            _ => {
                self.set_rejection("主机密钥确认超时，连接已终止");
                Ok(false)
            }
        }
    }
}

/// 从文件头识别加密私钥（PKCS8 / PKCS5 / PPK）；
/// OpenSSH 格式无法从头部判断（返回 None，由加载结果推断）。
fn detect_encrypted_by_header(content: &str) -> Option<bool> {
    if content.contains("-----BEGIN ENCRYPTED PRIVATE KEY-----") {
        return Some(true);
    }
    if content.contains("Proc-Type: 4,ENCRYPTED") {
        return Some(true);
    }
    if content.trim_start().starts_with("PuTTY-User-Key-File-") {
        for line in content.lines() {
            if let Some(v) = line.strip_prefix("Encryption:") {
                return Some(v.trim() != "none");
            }
        }
        return Some(false);
    }
    None
}

/// 私钥加载错误映射为可读提示（纯函数，便于单测）。
fn describe_key_error(
    path: &str,
    passphrase_provided: bool,
    encrypted: bool,
    e: &russh::keys::Error,
) -> String {
    use russh::keys::Error;
    if matches!(e, Error::KeyIsEncrypted) {
        return "私钥已加密，请在连接配置中填写私钥口令".to_string();
    }
    if encrypted {
        return if passphrase_provided {
            "私钥口令错误，请检查后重试".to_string()
        } else {
            "私钥已加密，请在连接配置中填写私钥口令".to_string()
        };
    }
    match e {
        Error::UnsupportedKeyType { .. } => {
            "不支持的私钥格式（支持 OpenSSH / PEM / PPK）".to_string()
        }
        Error::IO(_) => format!("私钥文件无法读取（权限不足或文件被占用）：{path}"),
        _ => format!("私钥读取失败：{path}（格式无法识别或文件已损坏）"),
    }
}

/// 加载私钥文件并生成友好错误信息。
fn load_private_key(path: &str, passphrase: &str) -> Result<russh::keys::PrivateKey, String> {
    let provided = !passphrase.is_empty();
    match load_secret_key(path, provided.then_some(passphrase)) {
        Ok(key) => Ok(key),
        Err(e) => {
            // 判定是否为加密密钥：库直接告知 / 文件头特征 /
            // 带口令失败但无口令时报 KeyIsEncrypted（覆盖 OpenSSH 格式口令错误）
            let encrypted = matches!(e, russh::keys::Error::KeyIsEncrypted)
                || std::fs::read_to_string(path)
                    .ok()
                    .and_then(|c| detect_encrypted_by_header(&c))
                    .unwrap_or(false)
                || (provided
                    && matches!(
                        load_secret_key(path, None),
                        Err(russh::keys::Error::KeyIsEncrypted)
                    ));
            Err(describe_key_error(path, provided, encrypted, &e))
        }
    }
}

/// 展开路径开头的 `~` / `~/` 为用户主目录。
fn expand_tilde(app: &AppHandle, path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = app.path().home_dir() {
            return home.join(rest).to_string_lossy().to_string();
        }
    }
    path.to_string()
}

/// 建立 SSH 连接并完成认证（不打开任何通道），供连接与测试连接复用。
#[allow(clippy::too_many_arguments)]
async fn establish(
    app: &AppHandle,
    conn_id: &str,
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    auth_type: &str,
    private_key_path: &str,
    key_passphrase: &str,
) -> Result<client::Handle<SshHandler>, String> {
    let addr = format!("{host}:{port}");

    // 快速可达性探测（DNS + TCP），保证不可达主机仍快速失败
    tokio::time::timeout(
        Duration::from_secs(10),
        tokio::net::TcpStream::connect(addr.as_str()),
    )
    .await
    .map_err(|_| "连接超时".to_string())?
    .map_err(|e| format!("连接失败: {e}"))?;

    // 连接配置：30s 保活（防 NAT/防火墙静默断链）、低延迟
    let config = Arc::new(client::Config {
        keepalive_interval: Some(Duration::from_secs(30)),
        keepalive_max: 3,
        nodelay: true,
        ..Default::default()
    });

    let handler = SshHandler::new(app.clone(), conn_id.to_string(), host.to_string(), port);

    // 握手包含可能的主机密钥确认交互，使用宽松超时
    let mut session = tokio::time::timeout(
        Duration::from_secs(120),
        client::connect(config, addr.as_str(), handler),
    )
    .await
    .map_err(|_| "连接超时".to_string())?
    .map_err(|e| {
        // 主机密钥确认被取消/超时：返回具体原因而非底层错误
        if let Ok(mut map) = app.state::<HostKeyState>().rejections.lock() {
            if let Some(reason) = map.remove(conn_id) {
                return reason;
            }
        }
        format!("连接失败: {e}")
    })?;

    // 清理可能残留的拒绝原因
    if let Ok(mut map) = app.state::<HostKeyState>().rejections.lock() {
        map.remove(conn_id);
    }

    if auth_type == "key" {
        // 私钥认证：读取私钥文件（口令同样以加密值存储，先解密）
        let path = expand_tilde(app, private_key_path.trim());
        if path.is_empty() {
            return Err("未配置私钥文件".to_string());
        }
        if !std::path::Path::new(&path).is_file() {
            return Err(format!("私钥文件不存在：{path}"));
        }
        let passphrase = decrypt_password(key_passphrase)?;
        let key = load_private_key(&path, &passphrase)?;
        let hash_alg = session
            .best_supported_rsa_hash()
            .await
            .map_err(|e| format!("认证协商失败: {e}"))?
            .flatten();
        let auth = tokio::time::timeout(
            Duration::from_secs(10),
            session.authenticate_publickey(
                username.to_string(),
                PrivateKeyWithHashAlg::new(Arc::new(key), hash_alg),
            ),
        )
        .await
        .map_err(|_| "认证超时".to_string())?
        .map_err(|e| format!("认证失败: {e}"))?;
        if !auth.success() {
            return Err("私钥未被服务器接受（请确认公钥已加入服务器 authorized_keys）".to_string());
        }
    } else {
        // 密码认证（前端可能传加密存储值 enc:v1:...，先解密；旧明文直接使用）
        let password = decrypt_password(password)?;
        let auth = tokio::time::timeout(
            Duration::from_secs(10),
            session.authenticate_password(username.to_string(), password),
        )
        .await
        .map_err(|_| "认证超时".to_string())?
        .map_err(|e| format!("认证失败: {e}"))?;
        if !auth.success() {
            return Err("用户名或密码错误".to_string());
        }
    }

    Ok(session)
}

/// 建立 SSH 会话。
///
/// 按 `session_type` 打开对应通道：
/// - `sftp`：仅打开 SFTP 子系统
/// - `ssh`：仅打开 shell 终端
/// - 其他（`ssh_sftp`）：两者都打开
///
/// 认证方式按 `auth_type` 二选一：`password`（密码）或 `key`（私钥文件）。
/// 认证成功后会将连接记录到"最近连接"（best-effort，失败不影响连接）。
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn ssh_connect(
    app: AppHandle,
    state: State<'_, SshManager>,
    conn_id: String,
    connection_id: String,
    host: String,
    port: u16,
    username: String,
    password: String,
    auth_type: String,
    private_key_path: String,
    key_passphrase: String,
    name: String,
    remark: String,
    session_type: String,
) -> Result<(), String> {
    // 若该 tab 已存在会话（重连），先中止旧的数据泵任务
    {
        let mut map = state.0.lock().map_err(|e| e.to_string())?;
        if let Some(old) = map.remove(&conn_id) {
            if let Some(shell) = old.shell {
                shell.task.abort();
            }
        }
    }

    let session = establish(
        &app,
        &conn_id,
        &host,
        port,
        &username,
        &password,
        &auth_type,
        &private_key_path,
        &key_passphrase,
    )
    .await?;

    // 认证成功，记录最近连接（失败仅忽略，不影响本次连接）
    if let Some(db) = app.try_state::<Db>() {
        if let Ok(mut db) = db.0.lock() {
            let _ = db.record_recent(
                &connection_id,
                &name,
                &session_type,
                &host,
                port as i64,
                &username,
                &remark,
            );
        }
    }

    let open_shell = session_type != "sftp";
    let open_sftp = session_type != "ssh";

    let mut state_out = ConnState {
        shell: None,
        sftp: None,
    };

    // 打开 shell 终端通道并启动输出泵
    if open_shell {
        let channel = session
            .channel_open_session()
            .await
            .map_err(|e| e.to_string())?;
        let (read, write) = channel.split();

        write
            .request_pty(false, "xterm-256color", 80, 24, 0, 0, &[])
            .await
            .map_err(|e| format!("请求 PTY 失败: {e}"))?;
        write
            .request_shell(true)
            .await
            .map_err(|e| format!("启动 shell 失败: {e}"))?;

        let write = Arc::new(tokio::sync::Mutex::new(write));
        let task = tauri::async_runtime::spawn(pump(
            app.clone(),
            SshManager(state.0.clone()),
            conn_id.clone(),
            read,
            write.clone(),
        ));
        state_out.shell = Some(SshHandle { write, task });
    }

    // 打开 SFTP 子系统通道
    if open_sftp {
        let channel = session
            .channel_open_session()
            .await
            .map_err(|e| format!("SFTP 通道打开失败: {e}"))?;
        channel
            .request_subsystem(true, "sftp")
            .await
            .map_err(|e| format!("SFTP 子系统启动失败: {e}"))?;
        let sftp = SftpSession::new(channel.into_stream())
            .await
            .map_err(|e| format!("SFTP 初始化失败: {e}"))?;
        state_out.sftp = Some(Arc::new(sftp));
    }

    // 登记会话
    let mut map = state.0.lock().map_err(|e| e.to_string())?;
    map.insert(conn_id, state_out);
    Ok(())
}

/// 测试连接：完成 TCP/握手/认证后立即断开（不打开通道、不记录最近连接）。
///
/// 编辑模式下密码/私钥口令留空时会使用数据库中已保存的值（掩码语义）。
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn ssh_test_connection(
    app: AppHandle,
    db: State<'_, Db>,
    host: String,
    port: u16,
    username: String,
    password: String,
    auth_type: String,
    private_key_path: String,
    key_passphrase: String,
    connection_id: Option<String>,
) -> Result<(), String> {
    // 编辑模式：补全未修改的密码/口令（先取库存值并释放锁，再进入网络等待）
    let mut password = password;
    let mut key_passphrase = key_passphrase;
    if let Some(id) = connection_id.as_deref() {
        let need_password = password.is_empty();
        let need_passphrase = auth_type == "key" && key_passphrase.is_empty();
        if need_password || need_passphrase {
            let (db_password, db_passphrase) = {
                let storage = db.0.lock().map_err(|e| e.to_string())?;
                storage
                    .connection_secret(id)
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| "连接不存在或已被删除".to_string())?
            };
            if need_password {
                password = db_password;
            }
            if need_passphrase {
                key_passphrase = db_passphrase;
            }
        }
    }

    // 测试连接同样需要主机密钥确认；用临时 id 关联前端弹窗
    let confirm_id = format!("test-{}", Uuid::new_v4());
    let session = establish(
        &app,
        &confirm_id,
        &host,
        port,
        &username,
        &password,
        &auth_type,
        &private_key_path,
        &key_passphrase,
    )
    .await?;

    // 优雅断开测试会话
    let _ = session
        .disconnect(russh::Disconnect::ByApplication, "", "")
        .await;
    Ok(())
}

/// 前端对主机密钥确认弹窗的响应（接受 / 取消）。
#[tauri::command]
pub fn ssh_confirm_host_key(
    state: State<'_, HostKeyState>,
    conn_id: String,
    accept: bool,
) -> Result<(), String> {
    let sender = state
        .pending
        .lock()
        .map_err(|e| e.to_string())?
        .remove(&conn_id);
    if let Some(tx) = sender {
        let _ = tx.send(accept);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use russh::keys::Error;

    #[test]
    fn describe_encrypted_key_without_passphrase() {
        let msg = describe_key_error("/k", false, false, &Error::KeyIsEncrypted);
        assert!(msg.contains("已加密"));
    }

    #[test]
    fn describe_wrong_passphrase() {
        let msg = describe_key_error("/k", true, true, &Error::CouldNotReadKey);
        assert_eq!(msg, "私钥口令错误，请检查后重试");
    }

    #[test]
    fn describe_encrypted_key_missing_passphrase() {
        let msg = describe_key_error("/k", false, true, &Error::CouldNotReadKey);
        assert!(msg.contains("已加密"));
    }

    #[test]
    fn describe_corrupt_key() {
        let msg = describe_key_error("/k", true, false, &Error::CouldNotReadKey);
        assert!(msg.contains("格式无法识别"));
    }

    #[test]
    fn describe_unsupported_and_io() {
        let e = Error::UnsupportedKeyType {
            key_type_string: "DSA".into(),
            key_type_raw: vec![],
        };
        assert!(describe_key_error("/k", false, false, &e).contains("不支持的私钥格式"));

        let io = Error::IO(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "denied",
        ));
        assert!(describe_key_error("/k", false, false, &io).contains("权限不足"));
    }

    #[test]
    fn detect_encrypted_headers() {
        assert_eq!(
            detect_encrypted_by_header("-----BEGIN ENCRYPTED PRIVATE KEY-----\nxxx"),
            Some(true)
        );
        assert_eq!(
            detect_encrypted_by_header("Proc-Type: 4,ENCRYPTED\nDEK-Info: AES-128-CBC,00"),
            Some(true)
        );
        assert_eq!(
            detect_encrypted_by_header("PuTTY-User-Key-File-3: ssh-ed25519\nEncryption: none"),
            Some(false)
        );
        assert_eq!(
            detect_encrypted_by_header(
                "PuTTY-User-Key-File-3: ssh-ed25519\nEncryption: aes256-cbc"
            ),
            Some(true)
        );
        assert_eq!(
            detect_encrypted_by_header("-----BEGIN OPENSSH PRIVATE KEY-----\nxxx"),
            None
        );
    }
}
