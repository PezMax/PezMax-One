# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```bash
cargo build              # debug build
cargo build --release    # release build
cargo check              # fast type-check only
cargo run                # run the desktop app
cargo fix                # auto-fix warnings
```

The app is a native Windows desktop binary (egui/eframe). No external build tooling needed.

The app connects to the remote backend API at `http://154.8.139.48:8080`.
Java backend source is maintained separately at PezMax/PezMax-Java.

## Architecture

### Monorepo Layout

```
PezMax-One/                  ← product root, Rust crate root
├── src/
│   ├── main.rs              ← eframe entry, window config, PDF engine init
│   ├── cache.rs             ← CacheManager（统一缓存管理，所有缓存文件强制在 `.cache/` 下）
│   ├── settings.rs          ← AppSettings（本地设置持久化，theme/accent/preferences 保存到 settings.json）
│   ├── app.rs               ← PezMaxApp state, routing, eframe::App impl
│   ├── api/                 ← typed HTTP client (reqwest)
│   │   ├── client.rs        ← ApiClient core: GET/POST/PUT/DELETE/upload/download
│   │   ├── models.rs        ← 28 serde models matching backend JSON
│   │   ├── auth.rs          ← login, register, captcha, password reset
│   │   ├── file.rs          ← paper file CRUD, tree, search
│   │   ├── bookmark.rs      ← bookmark CRUD
│   │   ├── user.rs          ← profile, avatar, password, security
│   │   ├── download.rs      ← download records, favorites
│   │   ├── notification.rs  ← popup/scroll notifications
│   │   ├── report.rs        ← report creation & timeline
│   │   └── favorite.rs      ← file favorite CRUD (desktop-only)
│   ├── theme/
│   │   └── mod.rs           ← Metro Design colors, fonts, spacing, transitions
│   ├── components/
│   │   ├── action_bar.rs    ← preview mode bottom toolbar (Back/Download/Favorite/Report)
│   │   ├── sidebar.rs       ← collapsible hamburger sidebar (SpringAnim 48↔200px)
│   │   ├── topbar.rs        ← title, search, avatar, back button
│   │   └── toast.rs         ← animated corner notifications (Progress-driven)
│   ├── pages/
│   │   ├── login.rs         ← Metro login card
│   │   ├── register.rs      ← 3-step registration flow
│   │   ├── forget_password.rs
│   │   ├── home.rs          ← Metro tile dashboard
│   │   ├── browse.rs        ← resource manager, bookmarks, favorites (3 subsections)
│   │   ├── community.rs     ← user ranking, contribute file, report record
│   │   ├── profile.rs       ← personal center, notifications, download history, settings
│   │   └── mod.rs           ← 7 page modules
│   ├── pdf/
│   │   └── mod.rs           ← PdfEngine (pdfium-render) + PdfViewer (Grid/Line modes)
│   └── sokuou/              ← Sokuou Engine（动画系统）
│       ├── mod.rs           ← 公共 API re-exports + map_range 工具函数
│       ├── progress.rs      ← Progress：时长驱动线性插值（可中断）
│       ├── spring.rs        ← SpringAnim：阻尼振荡器解析解
│       ├── easing.rs        ← 缓动函数（Linear/EaseOutCubic 等）
│       ├── uwp.rs           ← MetroAnim: UWP-style easing + EasingMode
│       └── animator.rs      ← Animation trait + Animator（预留存根）
├── SOKUOU_ENGINE.md         ← Sokuou Engine 完整设计书
├── SOKUOU_USAGE.md          ← Sokuou Engine 调用手册（开发者必读）
└── 后端接口列表.md           ← full API contract for all 34 backend controllers
```

### Key Design Decisions

- **egui immediate mode**: UI is rebuilt every frame. No hidden/shown state — conditional rendering via `match` on `current_page`.
- **Single state struct**: `PezMaxApp` holds all app state. Pages are pure functions `fn render(&mut PezMaxApp, &mut Ui)`. No per-page state.
- **API via trait extension**: `ApiClient` methods are defined across multiple files via `impl ApiClient { ... }` blocks, one per domain.
- **Metro Design**: Flat colors, large typography, generous whitespace, content-first cards. Theme defined in `theme/mod.rs` with runtime interpolation for dark/light mode transitions.
- **Async HTTP via tokio::spawn + oneshot**: The eframe frame loop is synchronous. Every async API call spawns a tokio task, sends the result back through a `oneshot::Receiver`, and the frame loop polls it each frame (`try_recv`).

### Async Data Loading Pattern

The `AsyncData<T>` struct in `app.rs` wraps the oneshot pattern into a reusable loader:

```rust
// 1. Declare in PezMaxApp
pub file_list_data: AsyncData<Vec<PaperFile>>,

// 2. Trigger load (idempotent — won't re-trigger if already loading or loaded)
app.trigger_load_file_list();  // ← internally calls file_list_data.load(|| async move { ... })

// 3. Poll each frame in update()
self.file_list_data.poll();

// 4. Read result
if let Some(ref files) = self.file_list_data.data { ... }
```

Each `AsyncData` has `.loading`, `.error`, `.loaded` states — use these for skeleton UI and error display.

### Navigation System

Pages are split into two groups in `app.rs`:
- **Auth pages** (Login, Register, ForgetPassword) — rendered full-screen when `is_logged_in == false`
- **App pages** — rendered inside sidebar + topbar + subtab bar + central panel when logged in

Navigation is 2-tier: `Section` (Home / Browse / Community / Profile) → `Subsection` (per-section tabs). The sidebar indicator and subtab underline both use `SpringAnim` for smooth transitions. Page enter/exit uses `SpringAnim` with opacity + vertical offset.

### Theme System

The theme system supports **smooth transitions** between light/dark mode and accent colors:

- `theme::is_dark()`, `set_dark()` — thread-local global state
- `theme::ACCENT_PRESETS[5]` — cobalt blue, spruce, crimson, amber, violet
- `theme::start_accent_transition(idx)` / `start_dark_transition(dark)` — begin a 0.3s MetroAnim-driven interpolation
- `theme::is_transitioning()` / `is_dark_transitioning()` — check if transition is in progress
- `colors::primary()`, `text_primary()`, `bg_white()`, etc. — all read from interpolated state, so colors shift smoothly each frame
- `apply_metro_theme(ctx)` — set egui Visuals to match current theme state; call every frame during transitions
- `setup_fonts(ctx)` — loads CJK fonts from Windows/macOS/Linux paths

### PDF Engine — PdfEngine + PdfViewer

- **PdfEngine**: global singleton holding `Arc<Pdfium>`. Renders pages in background tokio tasks (oneshot channel).
- **PdfViewer**: document state per opened PDF. Two view modes:
  - `Grid`: thumbnail grid, click to switch to Line mode at that page
  - `Line`: continuous vertical scroll + left overview panel
- Rendering pipeline: load bytes → sync metadata → spawn background renders (max 3 concurrent) → poll results each frame → cache textures
- Zoom: `display_scale_anim` (SpringAnim) for smooth zoom transitions

### Animation System — Sokuou Engine

**All animations and visual transitions MUST use Sokuou Engine** (`src/sokuou/`). Do not implement ad-hoc animations with raw timers or egui's built-in animation helpers.

Three animation primitives:

| Type | Class | When to use | Properties |
|------|-------|-------------|------------|
| Spring | `SpringAnim` | Position, size, panel slides, page transitions | `response` (0.3-0.5s), `damping_ratio` (0.8-0.85), interrupt-safe |
| Linear | `Progress` | Opacity, color fades, timed sequences | `duration` (0.2-0.3s), `Easing::EaseOutCubic` default |
| UWP | `MetroAnim` | Theme transitions (accent/dark mode) | `UwpEasing::Quadratic` + `EasingMode::EaseOut`, 0.3s |

Pattern:
```rust
// Animation instances live as fields on PezMaxApp
pub sidebar_anim: SpringAnim,  // 0.0=folded(48px) / 1.0=expanded(200px)

// Per-frame update (in eframe::App::update)
let dt = ctx.input(|i| i.stable_dt) as f64;
self.sidebar_anim.update(dt);
if !self.sidebar_anim.is_steady() { ctx.request_repaint(); }

// Render: read value(), never modify animation state in render
let width = map_range_clamped(anim_val, 54.0, 200.0) as f32;
```

Rules:
- **Never call `.set_target()` inside a render function** — only in event handlers
- Always call `ctx.request_repaint()` while any animation `!is_steady()`
- `Animator` in `animator.rs` is a **reserved stub** — do not use until validated
- After adding/modifying Sokuou animations, update `src/sokuou/NOTE.md`

### Cache System — `CacheManager` + `AppSettings`

**所有磁盘缓存文件必须放在 `.cache/` 目录下。** 非缓存用户数据（凭证、设置）放在根目录。

目录结构：

```
{data_dir}/                 ← 平台数据目录，由 `dirs::data_dir()` 决定
│                             Windows: %APPDATA%/PezMax/
│                             macOS:   ~/Library/Application Support/com.pezmax/
│                             Linux:   ~/.local/share/pezmax/
  credentials.json          ← 用户凭证（根目录，非缓存）
  settings.json             ← AppSettings（根目录，非缓存，serde_json 持久化）
  .cache/
    user_stats.json         ← 用户统计缓存
    avatar/{user_id}.cache  ← 排行头像缓存（原始字节）
    bookmark_cover/bm_cover_{id}.cache ← 书签封面缓存
    pdf/{file_id}/p{idx}_s{scale}.rgba ← PDF 页面渲染磁盘缓存
```

**`CacheManager`** (`src/cache.rs`) — 统一缓存管理器：
- `CacheManager::new()` — 创建目录结构，迁移旧缓存
- `save_credentials()` / `load_credentials()` / `clear_credentials()`
- `save_user_stats()` / `load_user_stats()`
- `read_avatar_cache()` / `write_avatar_cache()`
- `read_bookmark_cover_cache()` / `write_bookmark_cover_cache()`
- `read_rgba_cache()` / `write_rgba_cache()` — PDF 页面缓存（4B width + 4B height + raw RGBA）
- `clear_all_cache()` — 删除并重建 `.cache/` 目录
- `clear_pdf_cache()` — 删除 PDF 渲染缓存

**`AppSettings`** (`src/settings.rs`) — 本地设置持久化：
- 保存到 `{data_dir}/settings.json`（路径由 `dirs::data_dir()` 跨平台解析）
- 字段：`theme_mode`, `accent_idx`, `setting_auto_launch`, `setting_silent_download`, `pdf_view_mode`, `pdf_scale`, `window_size`, `window_pos`
- 启动时在 `PezMaxApp::new()` 加载，`on_exit()` 保存
- 主题/强调色变化时自动保存

**PDF 磁盘缓存：** 渲染完成后写入 `.cache/pdf/{file_id}/p{idx}_s{scale}.rgba`。下次打开同文件同缩放级别时，直接从磁盘读取缓存纹理，无需重新渲染。缩放变化会产生新的缓存文件（`scale_encoded = (RENDER_SCALE * scale * 100.0) as u32`）。

### API Layer (mapping to 后端接口列表.md)

| Module | Backend base path | Key endpoints |
|--------|------------------|---------------|
| `auth` | `/datum/user` | login, register, captcha, securityQuestions, resetPasswordBySecurity |
| `file` | `/datum/file` | list, tree, subjects, schools, search, CRUD, /datum/download/file |
| `bookmark` | `/datum/bookmark` | CRUD, uploadCover |
| `user` | `/datum/desktop/user/profile` | stats, username, avatar, password, security |
| `download` | `/datum/desktop/download` | list, hide; favorites via /datum/desktop/favorite |
| `notification` | `/system/notification/user` | popup, scroll |
| `report` | `/datum/report` | create, timeline |

### Current State

The project is a skeleton with all page/routing/API scaffolding in place, but most pages use mock data. High-priority next steps:
1. Replace mock login with real API calls in `login.rs`
2. Wire up `file_explorer.rs` to `get_file_list()` / `get_file_tree()`
3. Implement actual API calls in download, favorite, bookmark, notification pages
4. Add file download via `rfd` (save dialog) + `reqwest` streaming