# AGENTS.md —— PezMax-One for Linux（Rust + Kanesumi）

本文件是 `linux/` 目录的权威指南。仓库级约定见仓根 `AGENTS.md`，跨端架构见 `docs/ARCHITECTURE.md`。

**状态**（2026-10-19）：规划稿，尚无代码；排在 Windows 之后（D14）。

> Kanesumi 的写法以其 `docs/DEV_GUIDE.md`（元素树）为准，
> 控件行为以 `docs/CONTROL_SPEC.md` 为准；本文只写 PezMax 的取舍。

## 立项决策

| 决策 | 结论 | 原因 |
|---|---|---|
| UI | Kanesumi 主线（`ProjectArcturius/Kanesumi`）元素树：一个 `TreeApp` + `TreeHost`，**不用旧手写几何 `App` 接口** | Kanesumi 规定新应用一律走元素树；命中、焦点、悬停、损伤、IME 上下文由树负责 |
| 会话 | **只支持 Wayland**（KDE Plasma、GNOME mutter、其它实现 xdg-shell 的合成器），不做 X11 | harness 只有 sctk + xdg-shell 外壳 |
| 业务核心 | `pezmax-core` crate：tokio + reqwest（rustls）+ serde；**不依赖 Kanesumi**；`cargo test` 跑 spec 夹具 | 与 Windows / Android 对称；核心可脱离 UI 测试 |
| 动画 | Sokuou 经 `kanesumi-anim` 消费，不自造动画原语；不复用 egui 版的 `src/sokuou/` 副本 | Sokuou 是唯一真源 |
| PDF | pdfium-render（继承 egui 版 `src/pdf/mod.rs` 的管线与磁盘缓存设计），渲染结果交给 Kanesumi 图像控件 | 已在 egui 版跑通，质量与性能已知 |
| egui 代码 | 不迁移代码，只迁移经验（见本文「从 egui 版继承」） | 渲染模型完全不同 |
| 打包 | `.deb` + `.pkg.tar.zst`，x86_64 / aarch64 | 与 1.0.0 的发行矩阵一致 |

## Kanesumi 的引用方式

Kanesumi 以独立 workspace 存在，`kanesumi-anim` 以 git 依赖声明 sokuou（`GuitaristRin/Sokuou`），在 Ether 内用 `[patch]` 收敛到本地。
本项目照同样方式：

```toml
# linux/Cargo.toml（示意）
[workspace]
members = ["crates/pezmax-core", "crates/pezmax-app"]
resolver = "3"

[workspace.dependencies]
kanesumi-harness = { git = "<Kanesumi 公开仓库地址>", rev = "<固定提交>" }
kanesumi-controls = { git = "<同上>", rev = "<同上>" }

# 本机开发时指向兄弟检出，提交前注释掉
# [patch."<Kanesumi 仓库地址>"]
# kanesumi-harness = { path = "../../Ether/shared/kanesumi/kanesumi-harness" }
```

- **固定 rev**，升级 Kanesumi 是一个独立 commit（`build(linux): 升级 Kanesumi 至 <rev>`），正文写升级原因与回归结果。
- 待确认：Kanesumi 主线的公开拉取地址（本机检出的远端是私有路径）。若不公开，CI 构建需要凭据或镜像——**这是 M0 L3 的问题**。

## Crate 结构

```
linux/
├── AGENTS.md
├── Cargo.toml
├── crates/
│   ├── pezmax-core/
│   │   ├── src/{net,api,state,files,transfer,urls,notify,update,validation,platform}/
│   │   └── tests/spec_*.rs        # 读 ../../../spec/fixtures/
│   └── pezmax-app/
│       └── src/{shell,pages,reader,transfer,login,platform,i18n}/
└── packaging/{deb,arch,desktop}/
```

`pezmax-core` 的模块与 Windows `PezMax.Core` 一一对应（见 `windows/AGENTS.md`），命名改为 snake_case。

## 线程与异步

- Kanesumi 主循环是同步的；`pezmax-core` 在后台 tokio 运行时上跑全部网络与磁盘 I/O。
- 结果经 `std::sync::mpsc`（或 `crossbeam-channel`）送回 UI，送达后调用 `kanesumi_harness::wake::request()` 唤醒主循环；
  主循环在 `update` 中 drain 通道、更新状态、标记损伤。**不在主循环里 `block_on`。**
- 每个异步请求带「代」号：页面切走或条件变化后，旧请求的结果被丢弃（egui 版 `AsyncData` 的教训：
  幂等 + 可丢弃比可取消更容易做对）。

## 外壳与导航

- Kanesumi `NavigationView`（汉堡菜单），自适应断点与 Windows 端一致（600 / 1008）；结构与 Windows 端相同，便于对照。
- **窗口装饰**：KDE Plasma 提供服务端装饰（xdg-decoration）；**GNOME 不提供**，需要应用自绘标题栏（Kanesumi `title_bar`）
  并处理 `xdg_toplevel` 的 move / resize。harness 当前未见 xdg-decoration 协商代码——M0 L1 必验。
- 全局菜单（`kanesumi-appmenu`，dbusmenu + `org_kde_kwin_appmenu`）只在 Plasma 下有意义，作为可选增强，不承载唯一入口。

## 阅读器

| 格式 | 实现 |
|---|---|
| PDF | pdfium-render 后台渲染 RGBA → Kanesumi 图像（纹理）；网格 / 连续两视图；可视区 ± 2 页，限 3 并发；缩放先做视觉缩放、停止后重渲染；磁盘缓存 `$XDG_CACHE_HOME/pezmax/pdf/{fileId}/p{idx}_s{scale}.rgba`（4B 宽 + 4B 高 + RGBA，同 egui 版） |
| 图片 | `image` crate 解码（只开需要的格式特性） |
| TXT | Kanesumi 段落（`Scene::paragraph`），大文件分段 |
| Markdown | `pulldown-cmark` → 块级元素（标题、段落、列表、代码块、引用、图片）映射到 Kanesumi 元素；表格后置 |
| EPUB | **M1：下载后 `xdg-open` 交给系统阅读器**（如 Foliate）；原生阅读器（解包 + XHTML 子集 → Kanesumi 排版）列为后续研究项，不排期 |
| MOBI / AZW3 | 下载后 `xdg-open` |

pdfium 动态库随包分发（`/usr/lib/pezmax-one/libpdfium.so`），运行时按绝对路径加载；不依赖系统 pdfium。

## 平台集成

| 需求 | 实现 |
|---|---|
| 凭据 | Secret Service（`oo7` 或 `keyring`，经 D-Bus）；无 Secret Service 时退回 `$XDG_DATA_HOME/pezmax/credentials.json`（0600） |
| 文件选择 | xdg-desktop-portal FileChooser（`ashpd`）；上传多选、文件夹上传、选择下载目录都走 portal |
| 打开文件 / 链接 | portal OpenURI，退回 `xdg-open` |
| 系统通知 | `org.freedesktop.Notifications`（下载完成、维护提醒，可关闭） |
| 开机启动 | `$XDG_CONFIG_HOME/autostart/pezmax-one.desktop` |
| IME | Kanesumi 已接 `zwp_text_input_v3`；在 Plasma + fcitx5 与 GNOME + ibus 下各验一次搜索框与登录框 |
| 下载 | reqwest 流式 + `.part` + `Range` 续传（状态机在 core）；队列串行 |

## 从 egui 版继承

| egui 版 | 继承的东西 |
|---|---|
| `src/pdf/mod.rs` | 渲染管线、并发上限、两视图交互、磁盘缓存格式与 `scale_encoded` 规则 |
| `src/cache.rs` | 缓存与非缓存数据分目录（换成 XDG 目录）；清缓存不碰凭据与记录 |
| `src/app_menu/linux/` | dbusmenu / kwin appmenu 的协议细节（现已有 `kanesumi-appmenu`，以它为准） |
| `src/updater/` + `src/api/update.rs` | Linux 包安装流程（换成 CrateStack 规则） |
| `resources/linux/`、`resources/wayland-protocols/` | desktop 文件、图标；协议 XML 改由 Kanesumi 依赖提供 |

## M0 验证项（Linux）

| # | 验证项 | 通过标准 |
|---|---|---|
| L1 | 在 GNOME（mutter）与 KDE Plasma 上运行 Kanesumi 应用 | 窗口可移动 / 缩放 / 最大化；GNOME 下有可用标题栏；IME 输入中文 |
| L2 | pdfium → Kanesumi 图像 | 100 页以上 PDF 连续视图滚动无白屏、内存稳定；纹理上传不阻塞主循环 |
| L3 | 仓外引用 Kanesumi | 在 Ether 之外用 git rev 依赖编过 `tree_minimal` 等价程序；确认拉取地址与 CI 凭据 |
| L4 | portal 文件选择 | Plasma 与 GNOME 下多选、选目录可用 |
| L5 | tokio ↔ 主循环 | 后台请求结果经 `wake::request()` 送达，空闲时 CPU 接近 0 |
