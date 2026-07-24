//! 统一缓存管理 —— 所有磁盘缓存统一在平台数据目录下的 `.cache/` 中。
//! 非缓存用户数据（凭证、设置）放在平台数据目录根目录。
//!
//! 平台数据目录：
//! - Windows: `%APPDATA%/PezMax/`
//! - macOS:   `~/Library/Application Support/com.pezmax/`
//! - Linux:   `~/.local/share/pezmax/`

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ── 凭证 ─────────────────────────────────────────────────────────────────────

/// 保存到磁盘的凭证
#[derive(Debug, Serialize, Deserialize)]
pub struct SavedCredentials {
    pub token: String,
    pub username: String,
    pub remember_me: bool,
}

// ── CacheManager ─────────────────────────────────────────────────────────────

/// 统一缓存管理器，集中管理所有磁盘缓存路径和 I/O。
///
/// 目录结构：
/// ```text
/// %APPDATA%/PezMax/
///   credentials.json          — 用户凭证（根目录，非缓存）
///   settings.json             — 用户设置（根目录，非缓存）
///   .cache/
///     user_stats.json         — 用户统计缓存
///     avatar/{user_id}.cache  — 排行头像缓存（原始字节）
///     bookmark_cover/bm_cover_{id}.cache — 书签封面缓存
///     pdf/{file_id}/p{idx}_s{scale}.rgba — PDF 页面渲染缓存
/// ```
#[derive(Clone)]
pub struct CacheManager {
    data_dir: PathBuf,
    cache_dir: PathBuf,
}

impl CacheManager {
    /// 初始化 CacheManager，创建目录结构，迁移旧缓存。
    ///
    /// 平台数据目录：
    /// - Windows: `%APPDATA%/PezMax/`
    /// - macOS:   `~/Library/Application Support/com.pezmax/`
    /// - Linux:   `~/.local/share/pezmax/`
    pub fn new() -> Self {
        let data_dir = if let Some(base) = dirs::data_dir() {
            base.join("PezMax")
        } else {
            PathBuf::from(".")
        };
        let cache_dir = data_dir.join(".cache");

        // 创建目录
        let _ = std::fs::create_dir_all(&data_dir);
        let _ = std::fs::create_dir_all(&cache_dir);
        let _ = std::fs::create_dir_all(cache_dir.join("avatar"));
        let _ = std::fs::create_dir_all(cache_dir.join("bookmark_cover"));
        let _ = std::fs::create_dir_all(cache_dir.join("pdf"));

        let mgr = Self { data_dir, cache_dir };
        mgr.migrate_old_cache();
        mgr
    }

    // ── 基础路径 ───────────────────────────────────────────────────────────

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    pub fn avatar_dir(&self) -> PathBuf {
        self.cache_dir.join("avatar")
    }

    pub fn bookmark_cover_dir(&self) -> PathBuf {
        self.cache_dir.join("bookmark_cover")
    }

    pub fn pdf_cache_dir(&self) -> PathBuf {
        self.cache_dir.join("pdf")
    }

    pub fn pdf_file_dir(&self, file_id: i64) -> PathBuf {
        self.cache_dir.join("pdf").join(file_id.to_string())
    }

    // ── 文件路径 ───────────────────────────────────────────────────────────

    pub fn settings_path(&self) -> PathBuf {
        self.data_dir.join("settings.json")
    }

    pub fn credentials_path(&self) -> PathBuf {
        self.data_dir.join("credentials.json")
    }

    pub fn user_stats_path(&self) -> PathBuf {
        self.cache_dir.join("user_stats.json")
    }

    pub fn avatar_cache_path(&self, user_id: i64) -> PathBuf {
        self.avatar_dir().join(format!("{}.cache", user_id))
    }

    pub fn bookmark_cover_cache_path(&self, bm_id: i64) -> PathBuf {
        self.bookmark_cover_dir().join(format!("bm_cover_{}.cache", bm_id))
    }

    /// PDF 页面缓存路径，scale 是实际渲染缩放（RENDER_SCALE * logical_scale）。
    /// 文件名中的 scale 编码：`(scale * 100.0) as u32`，`{:05}` 格式。
    pub fn pdf_page_cache_path(&self, file_id: i64, page_idx: usize, render_scale: f32) -> PathBuf {
        let scale_encoded = (render_scale * 100.0) as u32;
        self.pdf_file_dir(file_id).join(format!("p{:04}_s{:05}.rgba", page_idx, scale_encoded))
    }

    // ── 凭证 I/O ──────────────────────────────────────────────────────────

    pub fn save_credentials(&self, token: &str, username: &str, remember_me: bool) {
        let creds = SavedCredentials {
            token: token.to_string(),
            username: username.to_string(),
            remember_me,
        };
        if let Ok(json) = serde_json::to_string(&creds) {
            let _ = std::fs::write(self.credentials_path(), json);
        }
    }

    pub fn load_credentials(&self) -> Option<SavedCredentials> {
        let path = self.credentials_path();
        if !path.exists() {
            return None;
        }
        let json = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&json).ok()
    }

    pub fn clear_credentials(&self) {
        let path = self.credentials_path();
        let _ = std::fs::remove_file(path);
    }

    // ── 用户统计 I/O ──────────────────────────────────────────────────────

    pub fn save_user_stats<T: Serialize>(&self, stats: &T) {
        if let Ok(json) = serde_json::to_string(stats) {
            let _ = std::fs::write(self.user_stats_path(), json);
        }
    }

    pub fn load_user_stats<T: serde::de::DeserializeOwned>(&self) -> Option<T> {
        let path = self.user_stats_path();
        if !path.exists() {
            return None;
        }
        let json = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&json).ok()
    }

    // ── 头像缓存 I/O ──────────────────────────────────────────────────────

    pub fn read_avatar_cache(&self, user_id: i64) -> Option<Vec<u8>> {
        let path = self.avatar_cache_path(user_id);
        if !path.exists() {
            return None;
        }
        let bytes = std::fs::read(&path).ok()?;
        if bytes.is_empty() { None } else { Some(bytes) }
    }

    pub fn write_avatar_cache(&self, user_id: i64, bytes: &[u8]) {
        if !bytes.is_empty() {
            let _ = std::fs::write(self.avatar_cache_path(user_id), bytes);
        }
    }

    // ── 书签封面缓存 I/O ──────────────────────────────────────────────────

    pub fn read_bookmark_cover_cache(&self, bm_id: i64) -> Option<Vec<u8>> {
        let path = self.bookmark_cover_cache_path(bm_id);
        if !path.exists() {
            return None;
        }
        let bytes = std::fs::read(&path).ok()?;
        if bytes.is_empty() { None } else { Some(bytes) }
    }

    pub fn write_bookmark_cover_cache(&self, bm_id: i64, bytes: &[u8]) {
        if !bytes.is_empty() {
            let _ = std::fs::write(self.bookmark_cover_cache_path(bm_id), bytes);
        }
    }

    // ── PDF 渲染缓存 I/O ──────────────────────────────────────────────────

    /// 读取 RGBA 缓存文件，返回 `(rgba_bytes, width, height)`。
    ///
    /// 文件格式：
    ///   [width: u32 LE][height: u32 LE][raw RGBA: w * h * 4]
    pub fn read_rgba_cache(path: &Path) -> Option<(Vec<u8>, usize, usize)> {
        let data = std::fs::read(path).ok()?;
        if data.len() < 8 {
            return None;
        }
        let w = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let h = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
        let expected = w * h * 4;
        if data.len() - 8 != expected {
            return None;
        }
        Some((data[8..].to_vec(), w, h))
    }

    /// 写入 RGBA 缓存文件：header (w, h) + raw RGBA bytes。
    pub fn write_rgba_cache(path: &Path, rgba: &[u8], w: usize, h: usize) {
        let _ = std::fs::create_dir_all(path.parent().unwrap());
        let mut data = Vec::with_capacity(8 + rgba.len());
        data.extend_from_slice(&(w as u32).to_le_bytes());
        data.extend_from_slice(&(h as u32).to_le_bytes());
        data.extend_from_slice(rgba);
        let _ = std::fs::write(path, data);
    }

    // ── 清理 ──────────────────────────────────────────────────────────────

    /// 清除所有缓存（删除并重建 `.cache/` 目录）。
    pub fn clear_all_cache(&self) -> std::io::Result<()> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)?;
        }
        std::fs::create_dir_all(&self.cache_dir)?;
        std::fs::create_dir_all(self.cache_dir.join("avatar"))?;
        std::fs::create_dir_all(self.cache_dir.join("bookmark_cover"))?;
        std::fs::create_dir_all(self.cache_dir.join("pdf"))?;
        Ok(())
    }

    /// 清除 PDF 渲染缓存。
    pub fn clear_pdf_cache(&self) -> std::io::Result<()> {
        let pdf_dir = self.cache_dir.join("pdf");
        if pdf_dir.exists() {
            std::fs::remove_dir_all(&pdf_dir)?;
        }
        std::fs::create_dir_all(&pdf_dir)?;
        Ok(())
    }

    // ── 迁移 ──────────────────────────────────────────────────────────────

    /// 迁移旧缓存目录（启动时调用）。
    /// 删除旧版 `avatar_cache/` 和 `bookmark_cover_cache/`（位于 %APPDATA%/PezMax/ 根目录）。
    fn migrate_old_cache(&self) {
        let old_dirs = ["avatar_cache", "bookmark_cover_cache"];
        for name in &old_dirs {
            let old_path = self.data_dir.join(name);
            if old_path.exists() {
                let _ = std::fs::remove_dir_all(&old_path);
            }
        }
    }
}