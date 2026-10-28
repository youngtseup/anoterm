//! SQLite 数据层：分组、连接与最近连接的持久化。
//!
//! 数据库文件位于应用数据目录（`anoterm.db`），开启 WAL 模式；
//! 所有写操作通过立即事务（`Immediate`）保证并发安全。

use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::{params, Connection as SqliteConnection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::crypto::{encrypt_password, is_encrypted};

/// 加密非空口令；空值保持空（表示无口令/未设置）
fn encrypt_optional(plain: &str) -> String {
    if plain.is_empty() {
        String::new()
    } else {
        encrypt_password(plain).unwrap_or_else(|_| plain.to_string())
    }
}

/// 全局数据库状态（Tauri managed state），内部为互斥的 [`Storage`]。
pub struct Db(pub Mutex<Storage>);

/// 连接记录（与数据库表 `connections` 对应）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub host: String,
    pub port: i64,
    pub username: String,
    pub password: String,
    pub remark: String,
    pub group_id: String,
    /// 认证方式：password（默认）| key
    pub auth_type: String,
    /// 私钥文件路径（明文，不复制私钥内容入库）
    pub private_key_path: String,
    /// 私钥口令（加密存储，可为空）
    pub key_passphrase: String,
}

/// 分组（含其下的连接列表）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub connections: Vec<Connection>,
}

/// 新建/更新连接时的输入参数（不含 id）。
#[derive(Debug, Clone, Deserialize)]
pub struct ConnectionInput {
    pub group_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub host: String,
    pub port: i64,
    pub username: String,
    pub password: String,
    pub remark: String,
    #[serde(default = "default_auth_type")]
    pub auth_type: String,
    #[serde(default)]
    pub private_key_path: String,
    #[serde(default)]
    pub key_passphrase: String,
}

/// 认证方式默认值（兼容旧数据/旧前端）
fn default_auth_type() -> String {
    "password".to_string()
}

/// 未分组连接在列表中的虚拟分组 id（仅用于前端展示，不写入 groups 表）。
pub const UNGROUPED_GROUP_ID: &str = "__ungrouped__";

/// 拖拽排序时提交的一条连接归属。
#[derive(Debug, Clone, Deserialize)]
pub struct ConnectionOrder {
    pub id: String,
    pub group_id: String,
}

/// 最近连接记录（连接快照，连接被删除后仍可展示）。
#[derive(Debug, Clone, Serialize)]
pub struct RecentConnection {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub host: String,
    pub port: i64,
    pub username: String,
    pub remark: String,
    pub last_used: i64,
}

/// SQLite 存储实现。
pub struct Storage {
    conn: SqliteConnection,
}

impl Storage {
    /// 打开（必要时创建）数据库并初始化表结构，随后迁移旧版明文密码。
    pub fn open(data_dir: &PathBuf) -> rusqlite::Result<Self> {
        std::fs::create_dir_all(data_dir).ok();
        let path = data_dir.join("anoterm.db");
        let conn = SqliteConnection::open(path)?;
        conn.busy_timeout(std::time::Duration::from_secs(10))?;
        let _ = conn.pragma_update(None, "journal_mode", "WAL");
        let storage = Self { conn };
        storage.init_schema()?;
        storage.migrate_plain_passwords();
        Ok(storage)
    }

    /// 迁移旧版明文密码为加密存储（best-effort：加密失败跳过，不影响启动）。
    fn migrate_plain_passwords(&self) {
        let Ok(plain) = self.list_plain_passwords() else {
            return;
        };
        for (id, password) in plain {
            if is_encrypted(&password) {
                continue;
            }
            if let Ok(enc) = encrypt_password(&password) {
                let _ = self.conn.execute(
                    "UPDATE connections SET password = ?1 WHERE id = ?2",
                    params![enc, id],
                );
            }
        }
    }

    /// 查询全部连接的 id 与密码（仅用于迁移）。
    fn list_plain_passwords(&self) -> rusqlite::Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare("SELECT id, password FROM connections")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.collect()
    }

    /// 建表：groups / connections / recent_connections，并补齐旧库缺失的列。
    fn init_schema(&self) -> rusqlite::Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS groups (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS connections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                type TEXT NOT NULL DEFAULT 'ssh_sftp',
                host TEXT NOT NULL DEFAULT '',
                port INTEGER NOT NULL DEFAULT 22,
                username TEXT NOT NULL DEFAULT '',
                password TEXT NOT NULL DEFAULT '',
                remark TEXT NOT NULL DEFAULT '',
                group_id TEXT NOT NULL,
                auth_type TEXT NOT NULL DEFAULT 'password',
                private_key_path TEXT NOT NULL DEFAULT '',
                key_passphrase TEXT NOT NULL DEFAULT '',
                sort_order INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS recent_connections (
                connection_id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                type TEXT NOT NULL,
                host TEXT NOT NULL,
                port INTEGER NOT NULL DEFAULT 22,
                username TEXT NOT NULL DEFAULT '',
                remark TEXT NOT NULL DEFAULT '',
                last_used INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS known_hosts (
                host TEXT NOT NULL,
                port INTEGER NOT NULL,
                key_type TEXT NOT NULL,
                public_key TEXT NOT NULL,
                fingerprint TEXT NOT NULL,
                added_at INTEGER NOT NULL,
                PRIMARY KEY (host, port, key_type)
            );",
        )?;
        self.ensure_connection_columns()
    }

    /// 兼容旧库：为 connections 补充后续新增的列（已存在则跳过）。
    fn ensure_connection_columns(&self) -> rusqlite::Result<()> {
        let mut stmt = self.conn.prepare("PRAGMA table_info(connections)")?;
        let cols: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<_, _>>()?;
        let additions = [
            (
                "auth_type",
                "ALTER TABLE connections ADD COLUMN auth_type TEXT NOT NULL DEFAULT 'password'",
            ),
            (
                "private_key_path",
                "ALTER TABLE connections ADD COLUMN private_key_path TEXT NOT NULL DEFAULT ''",
            ),
            (
                "key_passphrase",
                "ALTER TABLE connections ADD COLUMN key_passphrase TEXT NOT NULL DEFAULT ''",
            ),
        ];
        for (name, ddl) in additions {
            if !cols.iter().any(|c| c == name) {
                self.conn.execute_batch(ddl)?;
            }
        }
        Ok(())
    }

    /// 列出全部分组及其连接（一次查询后按 group_id 归组）。
    pub fn list_groups(&self) -> rusqlite::Result<Vec<Group>> {
        let mut groups_stmt = self
            .conn
            .prepare("SELECT id, name FROM groups ORDER BY sort_order, name")?;
        let groups: Vec<(String, String)> = groups_stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<_, _>>()?;

        let mut conns_stmt = self.conn.prepare(
            "SELECT id, name, type, host, port, username, password, remark, group_id,
                    auth_type, private_key_path, key_passphrase
             FROM connections ORDER BY sort_order, name",
        )?;
        let mut conns: Vec<Connection> = conns_stmt
            .query_map([], |row| {
                Ok(Connection {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    type_: row.get(2)?,
                    host: row.get(3)?,
                    port: row.get(4)?,
                    username: row.get(5)?,
                    password: row.get(6)?,
                    remark: row.get(7)?,
                    group_id: row.get(8)?,
                    auth_type: row.get(9)?,
                    private_key_path: row.get(10)?,
                    key_passphrase: row.get(11)?,
                })
            })?
            .collect::<Result<_, _>>()?;

        let mut out: Vec<Group> = groups
            .into_iter()
            .map(|(id, name)| {
                let mut group_conns = Vec::new();
                let mut i = 0;
                while i < conns.len() {
                    if conns[i].group_id == id {
                        group_conns.push(conns.remove(i));
                    } else {
                        i += 1;
                    }
                }
                Group {
                    id,
                    name,
                    connections: group_conns,
                }
            })
            .collect();

        // 未归属任何分组的连接集中放在末尾，仅在前端展示
        if !conns.is_empty() {
            out.push(Group {
                id: UNGROUPED_GROUP_ID.to_string(),
                name: "未分组".to_string(),
                connections: conns,
            });
        }
        Ok(out)
    }

    /// 新建分组并返回。
    pub fn create_group(&mut self, name: &str) -> rusqlite::Result<Group> {
        let id = Uuid::new_v4().to_string();
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let sort_order: i64 = tx.query_row(
            "SELECT COALESCE(MAX(sort_order) + 1, 0) FROM groups",
            [],
            |row| row.get(0),
        )?;
        tx.execute(
            "INSERT INTO groups (id, name, sort_order) VALUES (?1, ?2, ?3)",
            params![id, name, sort_order],
        )?;
        tx.commit()?;
        Ok(Group {
            id,
            name: name.to_string(),
            connections: Vec::new(),
        })
    }

    /// 重命名分组。
    pub fn rename_group(&mut self, id: &str, name: &str) -> rusqlite::Result<()> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute(
            "UPDATE groups SET name = ?1 WHERE id = ?2",
            params![name, id],
        )?;
        tx.commit()
    }

    /// 删除分组（级联删除其下连接）。
    pub fn delete_group(&mut self, id: &str) -> rusqlite::Result<()> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute("DELETE FROM connections WHERE group_id = ?1", params![id])?;
        tx.execute("DELETE FROM groups WHERE id = ?1", params![id])?;
        tx.commit()
    }

    /// 新建连接并返回完整记录（密码与私钥口令加密后存储）。
    pub fn create_connection(&mut self, input: &ConnectionInput) -> rusqlite::Result<Connection> {
        let id = Uuid::new_v4().to_string();
        let enc_password =
            encrypt_password(&input.password).unwrap_or_else(|_| input.password.clone());
        let enc_passphrase = encrypt_optional(&input.key_passphrase);
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let sort_order: i64 = tx.query_row(
            "SELECT COALESCE(MAX(sort_order) + 1, 0) FROM connections WHERE group_id = ?1",
            params![input.group_id],
            |row| row.get(0),
        )?;
        tx.execute(
            "INSERT INTO connections
                (id, name, type, host, port, username, password, remark, group_id,
                 auth_type, private_key_path, key_passphrase, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                id,
                input.name,
                input.type_,
                input.host,
                input.port,
                input.username,
                enc_password,
                input.remark,
                input.group_id,
                input.auth_type,
                input.private_key_path,
                enc_passphrase,
                sort_order,
            ],
        )?;
        tx.commit()?;
        Ok(Connection {
            id,
            name: input.name.clone(),
            type_: input.type_.clone(),
            host: input.host.clone(),
            port: input.port,
            username: input.username.clone(),
            password: enc_password,
            remark: input.remark.clone(),
            group_id: input.group_id.clone(),
            auth_type: input.auth_type.clone(),
            private_key_path: input.private_key_path.clone(),
            key_passphrase: enc_passphrase,
        })
    }

    /// 更新连接字段。
    ///
    /// 密码/私钥口令为空时保留原值（前端编辑框掩码提交空值）；非空则加密后覆盖。
    pub fn update_connection(&mut self, id: &str, input: &ConnectionInput) -> rusqlite::Result<()> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;

        // 读取当前值，用于"空提交=保留原值"语义
        let current: Option<(String, String)> = tx
            .query_row(
                "SELECT password, key_passphrase FROM connections WHERE id = ?1",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let (cur_password, cur_passphrase) = current.unwrap_or_default();

        let password = if input.password.is_empty() {
            cur_password
        } else {
            encrypt_password(&input.password).unwrap_or_else(|_| input.password.clone())
        };
        let passphrase = if input.key_passphrase.is_empty() {
            cur_passphrase
        } else {
            encrypt_optional(&input.key_passphrase)
        };

        tx.execute(
            "UPDATE connections
             SET name = ?1, type = ?2, host = ?3, port = ?4, username = ?5, password = ?6,
                 remark = ?7, group_id = ?8, auth_type = ?9, private_key_path = ?10,
                 key_passphrase = ?11
             WHERE id = ?12",
            params![
                input.name,
                input.type_,
                input.host,
                input.port,
                input.username,
                password,
                input.remark,
                input.group_id,
                input.auth_type,
                input.private_key_path,
                passphrase,
                id,
            ],
        )?;
        tx.commit()
    }

    /// 保存拖拽后的树顺序：分组顺序 + 每条连接的目标分组与全局顺序。
    pub fn reorder_tree(
        &mut self,
        group_ids: &[String],
        connections: &[ConnectionOrder],
    ) -> rusqlite::Result<()> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        for (index, id) in group_ids.iter().enumerate() {
            tx.execute(
                "UPDATE groups SET sort_order = ?1 WHERE id = ?2",
                params![index as i64, id],
            )?;
        }
        // 连接使用全局递增顺序，list_groups 归组后每组内部相对顺序仍然正确
        for (index, connection) in connections.iter().enumerate() {
            tx.execute(
                "UPDATE connections SET group_id = ?1, sort_order = ?2 WHERE id = ?3",
                params![connection.group_id, index as i64, connection.id],
            )?;
        }
        tx.commit()
    }

    /// 读取连接已保存的密码与私钥口令（密文，供编辑模式测试连接使用）。
    pub fn connection_secret(&self, id: &str) -> rusqlite::Result<Option<(String, String)>> {
        self.conn
            .query_row(
                "SELECT password, key_passphrase FROM connections WHERE id = ?1",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
    }

    /// 删除连接。
    pub fn delete_connection(&mut self, id: &str) -> rusqlite::Result<()> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute("DELETE FROM connections WHERE id = ?1", params![id])?;
        tx.commit()
    }

    /// 读取已保存的主机公钥（public_key, fingerprint）。
    pub fn get_known_host(
        &self,
        host: &str,
        port: u16,
        key_type: &str,
    ) -> rusqlite::Result<Option<(String, String)>> {
        self.conn
            .query_row(
                "SELECT public_key, fingerprint FROM known_hosts
                 WHERE host = ?1 AND port = ?2 AND key_type = ?3",
                params![host, port as i64, key_type],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
    }

    /// 保存/更新主机公钥（首次信任或密钥变更后更新）。
    pub fn upsert_known_host(
        &self,
        host: &str,
        port: u16,
        key_type: &str,
        public_key: &str,
        fingerprint: &str,
    ) -> rusqlite::Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        self.conn.execute(
            "INSERT INTO known_hosts (host, port, key_type, public_key, fingerprint, added_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(host, port, key_type) DO UPDATE SET
                public_key = excluded.public_key,
                fingerprint = excluded.fingerprint,
                added_at = excluded.added_at",
            params![host, port as i64, key_type, public_key, fingerprint, now],
        )?;
        Ok(())
    }

    /// 记录一次连接：按连接 id UPSERT（已存在则刷新时间与快照）。
    #[allow(clippy::too_many_arguments)]
    pub fn record_recent(
        &mut self,
        id: &str,
        name: &str,
        type_: &str,
        host: &str,
        port: i64,
        username: &str,
        remark: &str,
    ) -> rusqlite::Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute(
            "INSERT INTO recent_connections
                (connection_id, name, type, host, port, username, remark, last_used)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(connection_id) DO UPDATE SET
                name = excluded.name,
                type = excluded.type,
                host = excluded.host,
                port = excluded.port,
                username = excluded.username,
                remark = excluded.remark,
                last_used = excluded.last_used",
            params![id, name, type_, host, port, username, remark, now],
        )?;
        tx.commit()
    }

    /// 列出最近连接（按使用时间倒序，最多 30 条）。
    pub fn list_recent(&self) -> rusqlite::Result<Vec<RecentConnection>> {
        let mut stmt = self.conn.prepare(
            "SELECT connection_id, name, type, host, port, username, remark, last_used
             FROM recent_connections
             ORDER BY last_used DESC
             LIMIT 30",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(RecentConnection {
                id: row.get(0)?,
                name: row.get(1)?,
                type_: row.get(2)?,
                host: row.get(3)?,
                port: row.get(4)?,
                username: row.get(5)?,
                remark: row.get(6)?,
                last_used: row.get(7)?,
            })
        })?;
        rows.collect()
    }

    /// 清空最近连接记录。
    pub fn clear_recent(&mut self) -> rusqlite::Result<()> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute("DELETE FROM recent_connections", [])?;
        tx.commit()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 旧版库（无认证相关列）打开时自动补齐列并读出默认值；
    /// 私钥口令更新：非空加密存储，空提交保留原值。
    #[test]
    fn open_migrates_legacy_schema_and_updates_secrets() {
        let dir = std::env::temp_dir().join(format!("anoterm-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();

        // 构造旧版 schema
        {
            let conn = SqliteConnection::open(dir.join("anoterm.db")).unwrap();
            conn.execute_batch(
                "CREATE TABLE groups (id TEXT PRIMARY KEY, name TEXT NOT NULL, sort_order INTEGER NOT NULL DEFAULT 0);
                 CREATE TABLE connections (
                     id TEXT PRIMARY KEY, name TEXT NOT NULL, type TEXT NOT NULL DEFAULT 'ssh_sftp',
                     host TEXT NOT NULL DEFAULT '', port INTEGER NOT NULL DEFAULT 22,
                     username TEXT NOT NULL DEFAULT '', password TEXT NOT NULL DEFAULT '',
                     remark TEXT NOT NULL DEFAULT '', group_id TEXT NOT NULL, sort_order INTEGER NOT NULL DEFAULT 0);
                 CREATE TABLE recent_connections (
                     connection_id TEXT PRIMARY KEY, name TEXT NOT NULL, type TEXT NOT NULL,
                     host TEXT NOT NULL, port INTEGER NOT NULL DEFAULT 22, username TEXT NOT NULL DEFAULT '',
                     remark TEXT NOT NULL DEFAULT '', last_used INTEGER NOT NULL);
                 INSERT INTO groups (id, name) VALUES ('g1', '默认');
                 INSERT INTO connections (id, name, group_id) VALUES ('c1', '旧连接', 'g1');",
            )
            .unwrap();
        }

        let mut storage = Storage::open(&dir).unwrap();

        // 旧连接读出默认值
        let groups = storage.list_groups().unwrap();
        assert_eq!(groups.len(), 1);
        let legacy = &groups[0].connections[0];
        assert_eq!(legacy.auth_type, "password");
        assert_eq!(legacy.private_key_path, "");
        assert_eq!(legacy.key_passphrase, "");

        // 更新为私钥认证并设置口令 → 加密存储
        let input = ConnectionInput {
            group_id: "g1".into(),
            name: "新名字".into(),
            type_: "ssh".into(),
            host: "127.0.0.1".into(),
            port: 2222,
            username: "root".into(),
            password: String::new(),
            remark: String::new(),
            auth_type: "key".into(),
            private_key_path: "/keys/id_ed25519".into(),
            key_passphrase: "secret".into(),
        };
        storage.update_connection("c1", &input).unwrap();
        let updated = &storage.list_groups().unwrap()[0].connections[0];
        assert_eq!(updated.auth_type, "key");
        assert_eq!(updated.private_key_path, "/keys/id_ed25519");
        assert!(is_encrypted(&updated.key_passphrase));

        // 口令留空提交 → 保留原值（掩码语义）
        let input2 = ConnectionInput {
            key_passphrase: String::new(),
            password: String::new(),
            ..input
        };
        storage.update_connection("c1", &input2).unwrap();
        let kept = &storage.list_groups().unwrap()[0].connections[0];
        assert!(is_encrypted(&kept.key_passphrase));

        drop(storage);
        std::fs::remove_dir_all(&dir).ok();
    }

    /// 未分组连接归入虚拟分组；reorder_tree 持久化分组/连接顺序与归属。
    #[test]
    fn reorder_tree_and_ungrouped_listing() {
        let dir = std::env::temp_dir().join(format!("anoterm-test-order-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut storage = Storage::open(&dir).unwrap();
        storage
            .conn
            .execute_batch(
                "INSERT INTO groups (id, name, sort_order) VALUES ('g1', 'A', 0), ('g2', 'B', 1);
                 INSERT INTO connections (id, name, group_id, sort_order) VALUES
                   ('c1', 'one', 'g1', 0), ('c2', 'two', 'g1', 1),
                   ('c3', 'three', 'g2', 0), ('c4', 'four', '', 0);",
            )
            .unwrap();

        let groups = storage.list_groups().unwrap();
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[2].id, UNGROUPED_GROUP_ID);
        assert_eq!(groups[2].connections[0].id, "c4");

        storage
            .reorder_tree(
                &["g2".to_string(), "g1".to_string()],
                &[
                    ConnectionOrder {
                        id: "c3".into(),
                        group_id: "g1".into(),
                    },
                    ConnectionOrder {
                        id: "c2".into(),
                        group_id: "g1".into(),
                    },
                    ConnectionOrder {
                        id: "c1".into(),
                        group_id: "g2".into(),
                    },
                    ConnectionOrder {
                        id: "c4".into(),
                        group_id: String::new(),
                    },
                ],
            )
            .unwrap();

        let groups = storage.list_groups().unwrap();
        assert_eq!(groups[0].id, "g2");
        assert_eq!(groups[1].id, "g1");
        assert_eq!(groups[0].connections[0].id, "c1");
        assert_eq!(
            groups[1]
                .connections
                .iter()
                .map(|c| c.id.as_str())
                .collect::<Vec<_>>(),
            vec!["c3", "c2"]
        );
        assert_eq!(groups[2].id, UNGROUPED_GROUP_ID);
        assert_eq!(groups[2].connections[0].id, "c4");

        drop(storage);
        std::fs::remove_dir_all(&dir).ok();
    }

    /// known_hosts：保存后可读取；同键再次保存时更新公钥与指纹。
    #[test]
    fn known_hosts_upsert_and_get() {
        let dir = std::env::temp_dir().join(format!("anoterm-test-kh-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let storage = Storage::open(&dir).unwrap();

        assert_eq!(
            storage.get_known_host("h", 22, "ssh-ed25519").unwrap(),
            None
        );
        storage
            .upsert_known_host("h", 22, "ssh-ed25519", "KEY1", "SHA256:a")
            .unwrap();
        assert_eq!(
            storage.get_known_host("h", 22, "ssh-ed25519").unwrap(),
            Some(("KEY1".to_string(), "SHA256:a".to_string()))
        );
        storage
            .upsert_known_host("h", 22, "ssh-ed25519", "KEY2", "SHA256:b")
            .unwrap();
        assert_eq!(
            storage.get_known_host("h", 22, "ssh-ed25519").unwrap(),
            Some(("KEY2".to_string(), "SHA256:b".to_string()))
        );

        drop(storage);
        std::fs::remove_dir_all(&dir).ok();
    }
}
