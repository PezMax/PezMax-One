//! 本地设置持久化 —— 所有用户偏好保存在 `settings.json` 中。

use crate::cache::CacheManager;
use crate::pdf::ViewMode;
use crate::theme::ThemeMode;
use serde::{Deserialize, Serialize};

/// 本地持久化设置，启动时从 `settings.json` 加载，修改时自动保存。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// 外观模式
    pub theme_mode: ThemeMode,
    /// 强调色索引（0-4，对应 ACCENT_PRESETS）
    pub accent_idx: usize,
    /// 开机自启
    pub setting_auto_launch: bool,
    /// 静默下载
    pub setting_silent_download: bool,
    /// PDF 默认视图模式
    pub pdf_view_mode: ViewMode,
    /// PDF 默认缩放
    pub pdf_scale: f32,
    /// 默认下载目录（None = 使用平台默认 ~/Downloads/PezMax）
    #[serde(default)]
    pub download_dir: Option<String>,
    /// 窗口大小 (w, h)
    pub window_size: Option<(f32, f32)>,
    /// 窗口位置 (x, y)
    pub window_pos: Option<(f32, f32)>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::System,
            accent_idx: 0,
            setting_auto_launch: false,
            setting_silent_download: false,
            pdf_view_mode: ViewMode::Line,
            pdf_scale: 1.0,
            download_dir: None,
            window_size: None,
            window_pos: None,
        }
    }
}

impl AppSettings {
    /// 从磁盘加载设置，文件不存在或格式错误时返回默认值。
    pub fn load(cm: &CacheManager) -> Self {
        let path = cm.settings_path();
        if !path.exists() {
            return Self::default();
        }
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default()
    }

    /// 保存设置到磁盘。
    pub fn save(&self, cm: &CacheManager) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(cm.settings_path(), json);
        }
    }
}