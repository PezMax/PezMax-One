//! GitHub Release 自更新客户端
//!
//! 三步：
//! 1. `check_latest_release()` → 拉 GitHub API 最新 release，与当前版本比较
//! 2. `pick_asset()`           → 按 OS + ARCH + Linux 发行版选一个资产
//! 3. `download_asset()`       → 流式下载到临时目录
//!
//! 用户 Agent、超时、URL 全部本模块自持，不复用 ApiClient
//! （ApiClient 面向后端 base_url，这里是 github.com）。

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

const RELEASE_URL: &str =
    "https://api.github.com/repos/PezMax/PezMax-One/releases/latest";
const USER_AGENT: &str = "PezMax-One-Updater";

#[derive(Debug, Clone, Deserialize)]
pub struct GhAsset {
    pub name: String,
    pub browser_download_url: String,
    #[serde(default)]
    pub size: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // name/body/html_url 留给后续 release notes 展示
pub struct GhRelease {
    pub tag_name: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub html_url: String,
    #[serde(default)]
    pub assets: Vec<GhAsset>,
}

impl GhRelease {
    /// 去掉可能的 `v` 前缀，得到纯 semver 字符串。
    pub fn version(&self) -> &str {
        self.tag_name.strip_prefix('v').unwrap_or(&self.tag_name)
    }
}

/// 比对当前版本与 release 的 `tag_name`。true = 有新版本可更新。
///
/// 兼容 tag 带 / 不带 `v` 前缀；解析失败退化为字符串不等判断。
pub fn is_newer(current: &str, latest_tag: &str) -> bool {
    let latest = latest_tag.strip_prefix('v').unwrap_or(latest_tag);
    match (
        semver::Version::parse(current),
        semver::Version::parse(latest),
    ) {
        (Ok(c), Ok(l)) => l > c,
        _ => current != latest,
    }
}

/// 查询 GitHub 最新 stable release。
///
/// - `Ok(Some(release))` : 有正式发行版
/// - `Ok(None)`          : 仓库还没有正式发行版（`/releases/latest` 返回 404），
///                         调用方视同"已是最新"处理，别当错误
/// - `Err(_)`            : 真的网络/解析错误，消息保持精简
pub async fn check_latest_release() -> Result<Option<GhRelease>> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let resp = client
        .get(RELEASE_URL)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .context("网络请求失败")?;

    let status = resp.status();
    // 404 = 仓库无正式 release；不作为错误抛，让 UI 显示"已是最新"
    if status == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !status.is_success() {
        // 只保留状态码，不回显 body（GitHub 错误体是 JSON，很丑）
        anyhow::bail!("GitHub API 返回 HTTP {}", status.as_u16());
    }
    let release: GhRelease = resp
        .json()
        .await
        .context("响应解析失败")?;
    Ok(Some(release))
}

// ── 平台探测 ─────────────────────────────────────────────────────────────

/// 归一化架构标签：x86_64 / aarch64
pub fn current_arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" | "arm64" => "aarch64",
        other => other,
    }
}

/// Linux 发行版归类：arch / debian / unknown。
/// arch 系（arch/manjaro/endeavouros）→ "arch"
/// debian 系（debian/ubuntu/mint/pop）→ "debian"
#[cfg(target_os = "linux")]
pub fn detect_linux_distro() -> &'static str {
    use std::path::Path;
    if Path::new("/etc/arch-release").exists() {
        return "arch";
    }
    if Path::new("/etc/debian_version").exists() {
        return "debian";
    }
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            let (k, v) = match line.split_once('=') {
                Some(kv) => kv,
                None => continue,
            };
            let v = v.trim_matches('"').to_ascii_lowercase();
            if k == "ID" || k == "ID_LIKE" {
                for id in v.split_whitespace() {
                    match id {
                        "arch" | "manjaro" | "endeavouros" | "artix" => return "arch",
                        "debian" | "ubuntu" | "linuxmint" | "pop" | "raspbian" | "kali" => {
                            return "debian"
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    "unknown"
}

#[cfg(not(target_os = "linux"))]
pub fn detect_linux_distro() -> &'static str {
    "n/a"
}

// ── 资产匹配 ─────────────────────────────────────────────────────────────

/// 命名约定（与 build/*.sh 保持一致）：
///   Windows : pezmax-one-{ver}-windows-{x86_64|aarch64}.msi
///   macOS   : pezmax-one-{ver}-macos-universal.dmg（首选，退到 x86_64/aarch64）
///   Linux   : pezmax-one-{ver}-linux-{amd64|arm64}.deb          (Debian)
///           / pezmax-one-{ver}-linux-{x86_64|aarch64}.pkg.tar.zst (Arch)
///
/// 匹配优先级：先要求名字里带上 OS 段（windows/macos/linux），
/// 找不到再退化成"只按扩展名 + arch"（兼容误命名或旧包）。
///
/// 返回选中的 GhAsset。找不到时返回错误，UI 提示"当前平台无可用更新包"。
pub fn pick_asset(release: &GhRelease) -> Result<&GhAsset> {
    let arch = current_arch();

    #[cfg(target_os = "windows")]
    {
        let want_ext = ".msi";
        let os_key = "windows";
        // 首选：包含 OS + arch + msi
        if let Some(a) = release.assets.iter().find(|a| {
            a.name.ends_with(want_ext)
                && name_contains_os(&a.name, os_key)
                && name_matches_arch(&a.name, arch)
        }) {
            return Ok(a);
        }
        // 退化：只按扩展名 + arch
        if let Some(a) = release
            .assets
            .iter()
            .find(|a| a.name.ends_with(want_ext) && name_matches_arch(&a.name, arch))
        {
            return Ok(a);
        }
        // 兜底：任意 msi
        if let Some(a) = release.assets.iter().find(|a| a.name.ends_with(want_ext)) {
            return Ok(a);
        }
        anyhow::bail!("release 中没有 Windows(.msi) 安装包");
    }

    #[cfg(target_os = "macos")]
    {
        let os_key = "macos";
        // 首选：带 macos 段的 universal.dmg
        if let Some(a) = release.assets.iter().find(|a| {
            a.name.ends_with(".dmg")
                && name_contains_os(&a.name, os_key)
                && a.name.contains("universal")
        }) {
            return Ok(a);
        }
        // 次选：带 macos 段 + arch 匹配的 dmg
        if let Some(a) = release.assets.iter().find(|a| {
            a.name.ends_with(".dmg")
                && name_contains_os(&a.name, os_key)
                && name_matches_arch(&a.name, arch)
        }) {
            return Ok(a);
        }
        // 退化：不管 OS 段，universal.dmg
        if let Some(a) = release
            .assets
            .iter()
            .find(|a| a.name.ends_with(".dmg") && a.name.contains("universal"))
        {
            return Ok(a);
        }
        // 退化：不管 OS 段，arch 匹配的 dmg
        if let Some(a) = release
            .assets
            .iter()
            .find(|a| a.name.ends_with(".dmg") && name_matches_arch(&a.name, arch))
        {
            return Ok(a);
        }
        // 兜底：任意 dmg
        if let Some(a) = release.assets.iter().find(|a| a.name.ends_with(".dmg")) {
            return Ok(a);
        }
        anyhow::bail!("release 中没有 macOS(.dmg) 安装包");
    }

    #[cfg(target_os = "linux")]
    {
        let distro = detect_linux_distro();
        let os_key = "linux";
        // Debian 系 → .deb
        if distro == "debian" || distro == "unknown" {
            let deb_arch = match arch {
                "x86_64" => "amd64",
                "aarch64" => "arm64",
                _ => arch,
            };
            // 首选：带 linux 段的 .deb
            if let Some(a) = release.assets.iter().find(|a| {
                a.name.ends_with(".deb")
                    && name_contains_os(&a.name, os_key)
                    && a.name.contains(deb_arch)
            }) {
                return Ok(a);
            }
            // 退化：任意 .deb + deb_arch
            if let Some(a) = release
                .assets
                .iter()
                .find(|a| a.name.ends_with(".deb") && a.name.contains(deb_arch))
            {
                return Ok(a);
            }
        }
        // Arch 系 → .pkg.tar.zst
        if distro == "arch" || distro == "unknown" {
            // 首选：带 linux 段的 .pkg.tar.zst
            if let Some(a) = release.assets.iter().find(|a| {
                a.name.ends_with(".pkg.tar.zst")
                    && name_contains_os(&a.name, os_key)
                    && name_matches_arch(&a.name, arch)
            }) {
                return Ok(a);
            }
            // 退化：任意 .pkg.tar.zst + arch
            if let Some(a) = release
                .assets
                .iter()
                .find(|a| a.name.ends_with(".pkg.tar.zst") && name_matches_arch(&a.name, arch))
            {
                return Ok(a);
            }
        }
        anyhow::bail!(
            "release 中没有匹配的 Linux 安装包（发行版={} 架构={}）",
            distro,
            arch
        );
    }
}

fn name_matches_arch(name: &str, arch: &str) -> bool {
    // 简单包含判断即可：约定命名里 arch 独立成段
    let n = name.to_ascii_lowercase();
    match arch {
        "x86_64" => n.contains("x86_64") || n.contains("x64") || n.contains("amd64"),
        "aarch64" => n.contains("aarch64") || n.contains("arm64"),
        _ => n.contains(arch),
    }
}

/// 判断资产名是否携带指定 OS 段（大小写不敏感）。
/// `os_key` ∈ {"windows", "macos", "linux"}；macOS 兼容 "darwin" 别名。
fn name_contains_os(name: &str, os_key: &str) -> bool {
    let n = name.to_ascii_lowercase();
    if n.contains(os_key) {
        return true;
    }
    matches!(os_key, "macos") && n.contains("darwin")
}

// ── 下载 ────────────────────────────────────────────────────────────────

/// 下载资产到系统临时目录，返回完整文件路径。
///
/// 通过 `progress_tx` 回报字节进度（收到 total 表示下载完成）。
/// total 未知时（服务器不返回 Content-Length）会一直发 0。
pub async fn download_asset(
    asset: &GhAsset,
    progress_tx: tokio::sync::mpsc::UnboundedSender<(u64, u64)>,
) -> Result<PathBuf> {
    use tokio::io::AsyncWriteExt;

    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(600)) // 大安装包
        .build()?;

    let resp = client
        .get(&asset.browser_download_url)
        .send()
        .await
        .context("下载资产请求失败")?;

    if !resp.status().is_success() {
        anyhow::bail!("下载失败 HTTP {}", resp.status());
    }

    let total = resp.content_length().unwrap_or(asset.size);

    let tmp_dir = std::env::temp_dir();
    let dest = tmp_dir.join(&asset.name);
    // 清理旧文件（同名残留）
    let _ = std::fs::remove_file(&dest);

    let mut file = tokio::fs::File::create(&dest)
        .await
        .with_context(|| format!("创建临时文件失败: {}", dest.display()))?;

    let mut downloaded: u64 = 0;
    let mut resp = resp;
    // 用 chunk() 避免额外 futures_util 依赖 / stream feature
    while let Some(chunk) = resp.chunk().await.context("下载中断")? {
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        // 发进度；接收端可能已丢弃（UI 已切走），忽略 send 错误
        let _ = progress_tx.send((downloaded, total));
    }
    file.flush().await?;
    drop(file);

    Ok(dest)
}
