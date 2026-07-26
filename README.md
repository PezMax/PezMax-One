# PezMax One · 拼图满绩·绫

> **高性能试卷资源管理桌面客户端** — 基于 Rust + egui 的 Metro Design 原生应用

[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange?logo=rust)](https://www.rust-lang.org)
[![egui](https://img.shields.io/badge/egui-0.31-blue?logo=egui)](https://github.com/emilk/egui)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)](https://github.com/PezMax/PezMax-One)

---

## 📖 项目简介

PezMax（拼图满绩）是一个面向教育场景的试卷资源管理与分享平台。**PezMax One** 是桌面端原生客户端，使用 Rust 和 egui 框架构建，以 Metro Design 设计语言带来流畅、高效的浏览体验。

本项目基于 [PezMax-Desktop](https://github.com/PezMax/PezMax-Desktop) 的原理自发制作，但两者**并非替代或迭代关系**，而是各具特色的共存方案：原版采用现代毛玻璃风格，PezMax One 则选用 Takahashi Rinta 偏好的 Metro Design 平面设计语言，追求原生性能的同时，也是 **Sokuou Engine** 早期型的试验程序之一。

### 核心功能

| 功能 | 说明 |
|------|------|
| 📄 **PDF 预览** | 基于 pdfium 的工业级 PDF 渲染引擎，支持网格/连续滚动两种模式，缩放平滑动画过渡 |
| 🔍 **资源浏览** | 多级文件树 + 搜索，支持按学科/学校/年级筛选 |
| 🔖 **书签管理** | 自定义书签分组，支持封面图上传 |
| ⭐ **收藏系统** | 试卷收藏与分类管理 |
| 📥 **下载管理** | 下载记录追踪，静默下载模式 |
| 👤 **用户系统** | 注册/登录、密码找回、个人资料管理、头像上传 |
| 🔔 **通知推送** | 弹窗通知 + 滚动通知栏 |
| 📊 **统计看板** | 用户数据统计仪表盘，含动画计数器 |
| 🏆 **社区功能** | 用户排行榜、试卷贡献、举报系统 |

### 技术亮点

- **Sokuou Engine** — 自研动画引擎，提供 SpringAnim（弹簧物理）、Progress（缓动插值）、MetroAnim（UWP 风格）三种原语，所有 UI 过渡均使用 Sokuou Engine 驱动
- **Metro Design 主题系统** — 5 套强调色预设，亮/暗主题切换带平滑过渡动画，支持运行时动态切换
- **异步数据加载** — `AsyncData<T>` 模式封装 tokio oneshot 通道，页面骨架屏 + 加载态 + 错误态覆盖所有数据流
- **PDF 渲染管线** — 后台 tokio 任务渲染，RGBA 纹理磁盘缓存，最多 3 并发渲染，缩放自适应
- **统一缓存管理** — `CacheManager` 将所有缓存文件强制置于 `.cache/` 目录，支持清理和迁移

---

## 🚀 快速开始

### 系统要求

| 平台 | 最低要求 | 推荐 |
|------|----------|------|
| Windows | Windows 10 x64 | Windows 11 x64 |
| macOS | macOS 11 Big Sur | macOS 14 Sonoma |
| Linux | glibc 2.28+, X11/Wayland | Ubuntu 22.04+ |

### 安装依赖

**Rust 工具链**（必需）：

```bash
# 安装 Rust（如尚未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable
```

### 快速开发（调试运行）

```bash
cargo run                # 调试运行（首次约 2-3 分钟）
cargo check              # 只做类型检查（秒级）
cargo build --release    # 发布构建（首次冷编译约 5-10 分钟）
./target/release/pezmax-one     # Linux/macOS 直接运行
./target/release/pezmax-one.exe # Windows
```

PDFium 会由**首次运行 build 脚本**时自动下载到 `build/pdfium/`；调试模式下需要手动跑一次对应平台的 build 脚本先把 pdfium 拉下来（或把 `libpdfium.so` / `.dylib` / `pdfium.dll` 放到 `target/{debug,release}/` 里）。

---

## 📦 生产构建与打包

三个平台各一个入口脚本，pdfium 已内置到脚本里（存在则跳过，`FORCE_PDFIUM=1` 强制重下）。

### Linux（.deb + Arch pkg，多架构）

```bash
build/build-linux.sh            # 只打 host 架构
build/build-linux.sh x64        # x86_64
build/build-linux.sh arm64      # aarch64
build/build-linux.sh all        # x64 + arm64 全打（需 arm64 交叉工具链）
```

每个架构产出 **两种包**：
- `build/dist/pezmax-one-1.0.0-x64.deb`
- `build/dist/pezmax-one-1.0.0-1-x86_64.pkg.tar.zst`

安装：
```bash
sudo pacman -U build/dist/pezmax-one-*.pkg.tar.zst     # Arch / Manjaro
sudo dpkg -i  build/dist/pezmax-one-*.deb              # Debian / Ubuntu
```

安装后系统内路径：
- 二进制：`/usr/bin/pezmax-one`（wrapper，自动设 `LD_LIBRARY_PATH`）
- 库：`/usr/lib/pezmax-one/{pezmax-one, libpdfium.so}`
- Desktop 入口：`/usr/share/applications/io.github.pezmax.one.desktop`
- 图标：`/usr/share/icons/hicolor/{256x256,scalable}/apps/io.github.pezmax.one.*`

**KDE Plasma Global Menu 集成**：应用启动时通过 D-Bus 暴露 `com.canonical.dbusmenu`，并绑定 KWin 私有协议 `org_kde_kwin_appmenu_manager`。面板需**手动添加 "Global Menu" widget** 才能显示。

**交叉编译要求**（`all` 或 `arm64`）：
```bash
rustup target add aarch64-unknown-linux-gnu
sudo pacman -S aarch64-linux-gnu-gcc   # Arch
# Debian/Ubuntu: apt install gcc-aarch64-linux-gnu
```
另外需要 libwayland / libdbus 的 arm64 sysroot；否则请只跑 host 架构。

### macOS

```bash
build/build-macos.sh
# 产出：build/dist/pezmax-one-macos-{x64,arm64}.tar.gz
# 目前仍是裸二进制 + 启动器；完整 .app bundle 待补
```

### Windows

```cmd
build\build-windows.bat
```
产出：
- `build/dist/PezMaxOne-x64.msi`（需 WiX Toolset v3）
- `build/dist/pezmax-one-windows-x64.zip`（便携版）

---

## 🧰 常用命令

---

## 🏗️ 项目结构

```
PezMax-One/
├── Cargo.toml              # Rust 模块定义与依赖管理
├── build.rs                # Windows 资源编译（应用图标嵌入）
├── build/                  # 构建脚本（pdfium 已内置到各脚本，无需单独 fetch）
│   ├── build-windows.bat   # Windows：MSI + ZIP
│   ├── build-linux.sh      # Linux：.deb + .pkg.tar.zst（支持 x64/arm64/all）
│   └── build-macos.sh      # macOS：tar.gz（.app bundle 待补）
├── resources/              # 应用图标（ico / png / svg）
├── wix/                    # WiX 安装包配置
├── repowiki/               # 项目文档（架构设计、API 接口、部署运维）
└── src/
    ├── main.rs             # 入口：eframe 窗口初始化、PDF 引擎启动
    ├── app.rs              # PezMaxApp 状态管理、路由、AsyncData 加载器
    ├── cache.rs            # CacheManager 统一缓存管理
    ├── settings.rs         # AppSettings 本地设置持久化
    ├── api/                # 类型化 HTTP 客户端（基于 reqwest）
    │   ├── client.rs       # ApiClient 核心：GET/POST/PUT/DELETE/upload/download
    │   ├── models.rs       # 28 个 serde 模型，匹配后端 JSON 契约
    │   ├── auth.rs         # 登录、注册、验证码、密码重置
    │   ├── file.rs         # 试卷文件 CRUD、文件树、搜索
    │   ├── bookmark.rs     # 书签 CRUD
    │   ├── user.rs         # 个人资料、头像、密码、安全设置
    │   ├── download.rs     # 下载记录、收藏
    │   ├── notification.rs # 弹窗/滚动通知
    │   ├── report.rs       # 举报创建与时间线
    │   └── favorite.rs     # 文件收藏 CRUD
    ├── theme/              # Metro Design 主题系统
    │   └── mod.rs          # 颜色、字体、间距、过渡动画
    ├── components/         # 可复用 UI 组件
    │   ├── sidebar.rs      # 可折叠侧边栏（SpringAnim 48↔200px）
    │   ├── topbar.rs       # 标题栏、搜索、头像、返回按钮
    │   ├── action_bar.rs   # 预览模式底部工具栏
    │   ├── toast.rs        # 动画角标通知
    │   ├── animated_counter.rs  # 数字滚动动画
    │   ├── step_indicator.rs    # 步骤指示器
    │   ├── timeline_panel.rs    # 时间线面板
    │   ├── report_dialog.rs     # 举报对话框
    │   └── disclaimer_dialog.rs # 免责声明对话框
    ├── pages/              # 页面模块
    │   ├── login.rs        # Metro 登录卡片
    │   ├── register.rs     # 三步注册流程
    │   ├── forget_password.rs  # 密码找回
    │   ├── home.rs         # Metro 磁贴仪表盘
    │   ├── browse.rs       # 资源管理器、书签、收藏
    │   ├── community.rs    # 用户排行、试卷贡献、举报记录
    │   └── profile.rs      # 个人中心、通知、下载历史、设置
    ├── pdf/                # PDF 渲染引擎
    │   └── mod.rs          # PdfEngine + PdfViewer（网格/连续模式）
    └── sokuou/             # Sokuou Engine 动画系统
        ├── mod.rs          # 公共 API 与工具函数
        ├── spring.rs       # SpringAnim：阻尼振荡器解析解
        ├── progress.rs     # Progress：时长驱动线性插值
        ├── easing.rs       # 缓动函数库
        ├── uwp.rs          # MetroAnim: UWP 风格缓动
        ├── animator.rs     # Animation trait + Animator（预留存根）
        ├── SOKUOU_ENGINE.md # 完整设计书
        └── SOKUOU_USAGE.md # 开发者调用手册
```

---

## 🎨 设计系统

### Metro Design

PezMax One 采用 **Metro Design** 设计语言，特点：

- **扁平色彩** — 5 套强调色预设（钴蓝、云杉绿、绯红、琥珀、紫罗兰）
- **大号排版** — 清晰的内容层级，信息密度适中
- **宽松间距** — 内容优先的卡片式布局
- **平滑过渡** — 所有动画由 Sokuou Engine 驱动

### 暗色/亮色模式

支持运行时动态切换，主题色过渡通过 MetroAnim（0.3s）实现平滑插值。

### 动画系统：Sokuou Engine

Sokuou（即応エンジン）是自研动画引擎，核心哲学：**动画不是播放，而是空间状态的连续解析**。

```rust
// SpringAnim — 弹性物理动画（面板滑动、页面过渡）
pub sidebar_anim: SpringAnim,  // 0.0=折叠(48px) / 1.0=展开(200px)
self.sidebar_anim.set_target(1.0);  // 事件处理中触发

// 每帧更新
let dt = ctx.input(|i| i.stable_dt) as f64;
self.sidebar_anim.update(dt);
if !self.sidebar_anim.is_steady() { ctx.request_repaint(); }

// 渲染时只读
let width = map_range(self.sidebar_anim.value(), 54.0, 200.0);
```

详见 `src/sokuou/SOKUOU_ENGINE.md` 和 `src/sokuou/SOKUOU_USAGE.md`。

---

## 🧪 开发指南

### 代码规范

- 遵循 Rust 标准命名规范（snake_case 变量/函数，PascalCase 类型）
- 所有动画使用 Sokuou Engine，禁止手动 `timer` 或 `egui::lerp`
- 异步数据加载使用 `AsyncData<T>` 模式
- 磁盘缓存统一通过 `CacheManager` 管理

### 构建产物

```bash
# 调试
cargo build
target/debug/pezmax-one       # 或 .exe（Windows）

# 发布
cargo build --release
target/release/pezmax-one
```

### 常见问题

**Q: PDFium 未找到？**
A: 跑一次对应平台的 build 脚本（`build/build-{linux,macos}.sh` / `build\build-windows.bat`），它会自动下载 pdfium 到 `build/pdfium/`。若要在 `cargo run` 调试模式下用，把 pdfium 库（`libpdfium.so` / `.dylib` / `pdfium.dll`）拷到 `target/debug/` 或 `target/release/`。

**Q: 如何切换强调色？**
A: 在个人中心 → 设置中选择，当前支持 5 种预设，切换时有平滑过渡动画。

**Q: 如何清理缓存？**
A: 在个人中心 → 设置中点击"清除缓存"，或手动删除 `%APPDATA%/PezMax/.cache/`（Windows）。

---

## 📄 许可证

本项目基于 MIT 许可证开源 — 详见 [LICENSE](LICENSE) 文件。

---

## 👥 贡献

欢迎提交 Issue 和 Pull Request！在提交 PR 前请确保：

1. `cargo check` 通过
2. 代码风格与现有代码一致
3. 新功能包含对应的错误处理

---

## 🙏 致谢

- [egui](https://github.com/emilk/egui) — 即时模式 GUI 框架
- [pdfium-render](https://github.com/ajrcarey/pdfium-render) — PDF 渲染 Rust 绑定
- [pdfium-binaries](https://github.com/bblanchon/pdfium-binaries) — PDFium 预编译二进制
- [RuoYi](https://github.com/yangzongzhuan/RuoYi) — 后端框架参考