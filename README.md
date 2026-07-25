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

### 获取 PDFium 引擎

PezMax 使用 [pdfium-render](https://github.com/ajrcarey/pdfium-render) 作为 PDF 渲染后端，需要 PDFium 原生库支持。

> **Windows 用户**：运行构建脚本会自动下载 PDFium
> **macOS/Linux 用户**：需手动下载或使用对应平台的构建脚本

```bash
# Windows（会自动下载 pdfium.dll）
build\build-windows.bat

# 或手动下载
build\fetch-pdfium.bat        # 下载所有平台
build\fetch-pdfium.bat windows-x64  # 仅下载 Windows x64
```

更多平台支持见 `build/fetch-pdfium.sh` / `build/fetch-pdfium.bat`。

### 编译运行

```bash
# 调试模式
cargo run

# 发布模式（推荐日常使用）
cargo build --release
./target/release/pezmax-egui.exe   # Windows
./target/release/pezmax-egui       # macOS/Linux
```

### 快速检查

```bash
cargo check      # 仅类型检查（秒级）
cargo build      # 调试构建
cargo fix        # 自动修复警告
```

---

## 📦 安装包

生产构建使用 `build/build-windows.bat` 生成：

- **MSI 安装包** — `build/dist/PezMax-x64.msi`（需要 WiX Toolset v3）
- **便携版 ZIP** — `build/dist/pezmax-windows-x64.zip`

---

## 🏗️ 项目结构

```
PezMax-One/
├── Cargo.toml              # Rust 模块定义与依赖管理
├── build.rs                # Windows 资源编译（应用图标嵌入）
├── build/                  # 构建脚本与 PDFium 下载
│   ├── build-windows.bat   # Windows 完整构建（msi + zip）
│   ├── build-linux.sh      # Linux 构建脚本
│   ├── build-macos.sh      # macOS 构建脚本
│   ├── fetch-pdfium.bat    # PDFium 下载脚本（Windows）
│   └── fetch-pdfium.sh     # PDFium 下载脚本（Unix）
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
target/debug/pezmax-egui.exe

# 发布
cargo build --release
target/release/pezmax-egui.exe
```

### 常见问题

**Q: PDFium 未找到？**
A: 运行 `build/fetch-pdfium.bat`（Windows）或 `build/fetch-pdfium.sh`（Unix）下载对应平台的 PDFium 库。

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