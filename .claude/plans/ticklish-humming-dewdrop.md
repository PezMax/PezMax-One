# 统一缓存系统 + 本地设置持久化

## Context

目前的缓存是一盘散沙：`credentials.json` 在 `%APPDATA%/PezMax/` 根目录，`user_stats.json` 在 `.cache/`，头像/书签 Cover 用原始字节散落在 `.cache/avatar/` 和 `.cache/bookmark_cover/`，PDF 渲染完全没有磁盘缓存（每次打开文档都要重新渲染全部页面）。更严重的是，主题模式、强调色、所有设置开关**全部没有持久化**，每次重启丢失。`confy` 和 `sled` 声明在 Cargo.toml 里但从未使用。

目标：把所有缓存统一到 `.cache/` 下集中管理，添加本地设置持久化，PDF 渲染磁盘缓存，并更新 CLAUDE.md 强制执行 `.cache` 规范。

---

## 方案

### 1. 新建 `src/cache.rs` — CacheManager

统一管理所有磁盘缓存路径和 I/O，取代 `app.rs` 里散落的 `get_data_dir()` / `get_cache_dir()` / `avatar_cache_dir()` 等函数。

```rust
pub struct CacheManager {
    data_dir: PathBuf,   // %APPDATA%/PezMax/
    cache_dir: PathBuf,  // %APPDATA%/PezMax/.cache/
}
```

**路径方法：** `data_dir()` / `cache_dir()` / `avatar_dir()` / `bookmark_cover_dir()` / `pdf_cache_dir()` / `pdf_file_dir(file_id)` / `settings_path()` / `credentials_path()` / `user_stats_path()`

**缓存读写方法：**
- `save_credentials()` / `load_credentials()` / `clear_credentials()`
- `save_user_stats()` / `load_user_stats()`
- `read_avatar_cache(user_id)` / `write_avatar_cache(user_id, bytes)`
- `read_bookmark_cover_cache(bm_id)` / `write_bookmark_cover_cache(bm_id, bytes)`
- `read_pdf_rgba_cache(path)` / `write_pdf_rgba_cache(path, rgba, w, h)` — 4B width + 4B height + raw RGBA
- `clear_all_cache()` / `clear_pdf_cache()`
- `migrate_old_cache()` — 删除旧 `avatar_cache/`、`bookmark_cover_cache/`

将 `SavedCredentials` 结构体从 `app.rs` 移入 `cache.rs`。

### 2. 新建 `src/settings.rs` — AppSettings

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme_mode: ThemeMode,        // Light / Dark / System
    pub accent_idx: usize,             // 0-4
    pub setting_auto_launch: bool,
    pub setting_silent_download: bool,
    pub pdf_view_mode: ViewMode,       // Grid / Line
    pub pdf_scale: f32,
    pub window_size: Option<(f32, f32)>,
    pub window_pos: Option<(f32, f32)>,
}
```

- `Default` trait + `AppSettings::load(cache_manager)` / `AppSettings::save(&self, cache_manager)`
- 序列化到 `%APPDATA%/PezMax/settings.json`（`serde_json` 直接读写，不用 `confy`）
- 给 `ThemeMode`（`theme/mod.rs`）和 `ViewMode`（`pdf/mod.rs`）添加 `Serialize, Deserialize` derive

### 3. PDF 磁盘缓存

渲染完成后写入 `.cache/pdf/{file_id}/p{idx}_s{scale}.rgba`，下次打开同文件同缩放直接读取。

- **缓存键格式：** `p{page_idx}_s{scale_encoded}.rgba`，`scale_encoded = (RENDER_SCALE * scale * 100.0) as u32`，`{:05}` 格式
- **文件格式：** `[width: u32 LE][height: u32 LE][RGBA bytes: w * h * 4]`
- **读取流程：** `request_render()` 先检查磁盘缓存 → 命中则直接 `ctx.load_texture()` 返回，不启线程；未命中则走现有渲染，渲染线程写入缓存
- **线程安全：** `CacheManager` 只存 `PathBuf`（`Clone`），传给 `std::thread::spawn` 的闭包是安全的

### 4. 修改 `src/app.rs`

- 删除 `get_data_dir()` / `get_cache_dir()` / `avatar_cache_dir()` / `bookmark_cover_cache_dir()` / `user_stats_cache_path()` / `migrate_old_cache()` / `credentials_path()` / `save_credentials()` / `load_credentials()` / `clear_credentials()` / `SavedCredentials`（移到 `cache.rs`）
- 保留 `decode_base64_image()`（无关缓存）
- `PezMaxApp` 新增字段：`cache_manager: CacheManager`、`settings: AppSettings`、`pdf_file_id: Option<i64>`
- `new()` 中初始化 `CacheManager`，加载 `AppSettings`，应用到主题全局变量
- `trigger_load_pdf_bytes()` 中记录 `self.pdf_file_id`
- 调用 `pdf_viewer.load_document()` 和 `pdf_viewer.poll_render()` 时传入 `&cache_manager`
- 新增 `clear_cache()` 方法：清磁盘缓存 + 清内存纹理
- `update()` 中主题/强调色变化时立即保存 settings
- 新增 `on_exit()` 在 `eframe::App` impl 中：保存窗口大小 + 位置 + settings

### 5. 修改 `src/pdf/mod.rs`

- `PdfViewer` 新增 `pub file_id: Option<i64>`
- `load_document()` 签名增加 `file_id: Option<i64>` 和 `cache_manager: &CacheManager` 参数
- `request_render()` 检查磁盘缓存 → 命中直接加载纹理；未命中则渲染后写入缓存
- 渲染线程中写入 `.rgba` 文件（`CacheManager::write_pdf_rgba_cache`）
- 新增 `clear_textures()` 方法
- 给 `ViewMode` 添加 `Serialize, Deserialize`

### 6. 修改 `src/pages/profile.rs`

- `action_row(ui, "清理缓存", ...)` 绑定 `app.clear_cache()` 点击事件
- 可选：添加 "清理 PDF 缓存" 独立按钮

### 7. 修改 `src/pages/browse.rs`

- 将 `crate::app::bookmark_cover_cache_dir().join(...)` 改为 `app.cache_manager.bookmark_cover_cache_path(id)`

### 8. 修改 `src/main.rs`

- 添加 `mod cache; mod settings;`

### 9. 更新 `CLAUDE.md`

- 添加 `cache.rs` 和 `settings.rs` 到 monorepo layout
- 添加 Cache System 章节，强制所有缓存文件放 `.cache/` 下

---

## 关键文件

| 文件 | 改动 |
|------|------|
| `src/cache.rs` | **新建** — CacheManager，统一缓存路径和 I/O |
| `src/settings.rs` | **新建** — AppSettings 结构体 + 序列化 |
| `src/main.rs` | 添加 `mod cache; mod settings;` |
| `src/app.rs` | 删除旧缓存函数，集成 CacheManager/AppSettings，新增 `clear_cache()`、`on_exit()` |
| `src/pdf/mod.rs` | 添加 file_id 跟踪、磁盘缓存检查/写入、`clear_textures()`、`ViewMode` serde |
| `src/theme/mod.rs` | `ThemeMode` 添加 Serialize/Deserialize |
| `src/pages/profile.rs` | "清理缓存" 按钮绑定 `app.clear_cache()` |
| `src/pages/browse.rs` | 更新 `bookmark_cover_cache_dir()` 调用为 `cache_manager.bookmark_cover_cache_path()` |
| `CLAUDE.md` | 添加 Cache System 章节和 `.cache` 规范 |

---

## 验证

1. `cargo check` — 编译通过
2. 启动应用，检查 `%APPDATA%/PezMax/settings.json` 是否自动创建
3. 修改主题/强调色，重启应用，验证设置恢复
4. 打开 PDF，关闭，再打开，验证第二次快于第一次（磁盘缓存命中）
5. "清理缓存" 按钮 → `.cache/` 被清空，纹理被清除，应用仍正常工作
6. 缩放 PDF → 新的缩放级别产生新的缓存文件
7. 旧 `avatar_cache/` 和 `bookmark_cover_cache/` 目录若存在，启动时自动删除