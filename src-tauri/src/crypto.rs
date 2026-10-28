//! 连接密码加密模块。
//!
//! 使用 AES-256-GCM 对密码加密后存入数据库，避免明文直读。
//! 存储格式：`enc:v1:<base64(iv(12B) + ciphertext + tag(16B))>`
//!
//! 说明：主密钥为内置固定常量（当前阶段选择），防护目标是"无法直接查看
//! 数据库文件"；二进制逆向可提取密钥，后续可升级为系统凭据库（keyring）保护。

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::Engine as _;
use rand::RngCore;

/// 加密值前缀标记；解密时以此识别已加密数据，旧明文（无前缀）原样返回
const PREFIX: &str = "enc:v1:";

/// 内置固定主密钥（32 字节 = AES-256）
const MASTER_KEY: [u8; 32] = [
    0x5f, 0x1a, 0x8c, 0x3e, 0x9d, 0x2b, 0x47, 0xf0, 0x6c, 0x81, 0x04, 0xab, 0x7d, 0x33, 0xe9, 0x52,
    0x0d, 0x96, 0xb4, 0x2e, 0x71, 0xc8, 0x5a, 0x13, 0x88, 0x42, 0xdf, 0x60, 0x35, 0xa7, 0x19, 0x6b,
];

/// 加密明文密码，返回 `enc:v1:` 前缀的存储值。
pub fn encrypt_password(plain: &str) -> Result<String, String> {
    let cipher = Aes256Gcm::new_from_slice(&MASTER_KEY).map_err(|e| e.to_string())?;

    // 随机 12 字节 IV（GCM 要求 nonce 不重复使用）
    let mut iv = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut iv);

    let nonce = Nonce::from_slice(&iv);
    let ciphertext = cipher
        .encrypt(nonce, plain.as_bytes())
        .map_err(|e| format!("密码加密失败: {e}"))?;

    // iv + ciphertext(含 tag) 拼接后 base64
    let mut payload = Vec::with_capacity(iv.len() + ciphertext.len());
    payload.extend_from_slice(&iv);
    payload.extend_from_slice(&ciphertext);
    Ok(format!(
        "{PREFIX}{}",
        base64::engine::general_purpose::STANDARD.encode(payload)
    ))
}

/// 解密存储的密码。
///
/// - 带 `enc:v1:` 前缀：解密返回明文
/// - 无前缀（旧版明文数据）：原样返回，兼容迁移前数据
pub fn decrypt_password(stored: &str) -> Result<String, String> {
    if let Some(b64) = stored.strip_prefix(PREFIX) {
        let payload = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .map_err(|e| format!("密码数据解码失败: {e}"))?;
        if payload.len() < 12 + 16 {
            return Err("密码数据长度非法".to_string());
        }
        let (iv, ct) = payload.split_at(12);
        let cipher = Aes256Gcm::new_from_slice(&MASTER_KEY).map_err(|e| e.to_string())?;
        let plain = cipher
            .decrypt(Nonce::from_slice(iv), ct)
            .map_err(|_| "密码解密失败（密钥不匹配或数据损坏）".to_string())?;
        String::from_utf8(plain).map_err(|e| format!("密码解码失败: {e}"))
    } else {
        Ok(stored.to_string())
    }
}

/// 判断是否为已加密的存储值。
pub fn is_encrypted(stored: &str) -> bool {
    stored.starts_with(PREFIX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let enc = encrypt_password("P@ssw0rd!").unwrap();
        assert!(enc.starts_with(PREFIX));
        assert_eq!(decrypt_password(&enc).unwrap(), "P@ssw0rd!");
    }

    #[test]
    fn decrypt_plain_passthrough() {
        assert_eq!(decrypt_password("legacy-plain").unwrap(), "legacy-plain");
        assert!(!is_encrypted("legacy-plain"));
        assert!(is_encrypted(&encrypt_password("x").unwrap()));
    }

    #[test]
    fn ciphertext_differs_per_call() {
        // 每次加密使用随机 IV，同一明文两次加密结果不同
        let a = encrypt_password("same").unwrap();
        let b = encrypt_password("same").unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn decrypt_tampered_fails() {
        let mut enc = encrypt_password("secret").unwrap();
        enc.push('A');
        assert!(decrypt_password(&enc).is_err());
    }
}
