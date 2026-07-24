//! 本地密码混淆 — XOR + machine-id 派生 key。
//!
//! 用于在 credentials.json 里落盘"记住密码"字段。
//! 明确目的：防止 grep credentials.json 直接看到明文；不抗内存转储、也不抗物理访问。
//! 若后端提供 RSA 公钥，应换成 jsencrypt 兼容方案。

use base64::Engine;

/// 生成 machine-scoped key（跨启动稳定）。
/// 优先用 machine-uid crate 拿系统 uuid；失败时用 hostname+user 兜底。
fn machine_key() -> Vec<u8> {
    let seed = machine_uid::get()
        .ok()
        .or_else(|| {
            let host = hostname::get().ok().and_then(|h| h.into_string().ok()).unwrap_or_default();
            let user = std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_default();
            if host.is_empty() && user.is_empty() { None } else { Some(format!("{host}:{user}")) }
        })
        .unwrap_or_else(|| "pezmax-fallback-key".to_string());

    // 简易 hash：拉伸到 32B，避免密钥太短周期性明显
    let mut key = Vec::with_capacity(32);
    for (i, b) in seed.bytes().cycle().take(32).enumerate() {
        key.push(b.wrapping_add(i as u8 ^ 0x5A));
    }
    key
}

/// 加密：XOR + base64 URL-safe，输出可直接放 JSON。
pub fn obfuscate(plain: &str) -> String {
    let key = machine_key();
    let bytes: Vec<u8> = plain
        .bytes()
        .enumerate()
        .map(|(i, b)| b ^ key[i % key.len()])
        .collect();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// 解密：反向 XOR。解码失败时返回 None（视为无有效密码）。
pub fn deobfuscate(cipher: &str) -> Option<String> {
    let key = machine_key();
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(cipher).ok()?;
    let plain: Vec<u8> = bytes
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ key[i % key.len()])
        .collect();
    String::from_utf8(plain).ok()
}
