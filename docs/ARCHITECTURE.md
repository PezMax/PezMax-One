# PezMax-One 新架构总纲

PezMax-One 是 PezMax（拼图满绩，学习资料分享与整理）的**原生客户端家族**：Windows、Linux、Android 三端，
消费 PezMax-Go 后端的 `/datum/*` 契约，与上游的 Electron 官方桌面端并存。macOS 暂缓。

本文是跨平台层面的**已定架构**。各平台细节见 `windows/AGENTS.md`、`linux/AGENTS.md`、`android/AGENTS.md`，
它们是各自目录的权威指南。现状依据见 `STATE_2026-10-19.md`，排期见 `PLAN.md`。

写代码时如发现与本文冲突，**先改本文、再改代码**，并在 commit 里说明原因。

## 平台矩阵

| 平台 | 语言 / UI | 运行时 / 外壳 | 视觉底座 | 产物 |
|---|---|---|---|---|
| **Windows** | C# 10 + XAML，UWP + WinUI 2.8 | .NET Native，AppContainer | Kanesumi.Xaml（XAML 扇区） | MSIX + `.appinstaller` + `.cer` |
| **Linux** | Rust（edition 2024）+ Kanesumi 元素树 | Kanesumi harness：Wayland（sctk）+ wgpu | Kanesumi 主线（Ether 扇区） | `.deb`、`.pkg.tar.zst`（x86_64 / aarch64） |
| **Android** | Kotlin + Jetpack Compose | Android 7.0+（minSdk 24） | Kanesumi-sec-a | 通用 APK |
| macOS | —— | —— | —— | 暂缓；1.0.0 dmg 不再更新 |

WinUI 3 / Windows App SDK **不在考虑范围内**（与 Ncrust 同一决定），不要引入、不要主动提议迁移。

## 仓库布局（目标形态）

沿用 Ncrust 的「按平台命名的目录 + 跨端 spec」：

```
PezMax-One/
├── windows/                 # C# + UWP + WinUI 2  —— 权威文档 windows/AGENTS.md
│   ├── PezMax.Windows.sln   # 手写维护，仅 Release|x64（M2 加 ARM64，D9）
│   ├── src/PezMax.Core/     # netstandard2.0：契约、业务、状态机；不依赖 WinRT 与 UI
│   ├── src/PezMax.App/      # UWP 应用：页面、阅读器、下载 / 上传、平台集成
│   ├── tests/PezMax.Core.Tests/   # net9.0 + xUnit，直接读 ../../spec/
│   └── tools/               # shot（自 arc-deck / Ncrust 移植）、release、icons
├── linux/                   # Rust + Kanesumi    —— 权威文档 linux/AGENTS.md
│   ├── Cargo.toml           # 独立 workspace
│   ├── crates/pezmax-core/  # 契约、业务、状态机；tokio + reqwest；不依赖 Kanesumi
│   ├── crates/pezmax-app/   # TreeApp 实现：页面、阅读器、平台集成
│   └── packaging/           # deb / pkg.tar.zst 打包脚本与 desktop 文件
├── android/                 # Kotlin + Compose   —— 权威文档 android/AGENTS.md
│   ├── settings.gradle.kts  # includeBuild("../../Kanesumi-sec-a")
│   └── app/
├── spec/                    # 跨端标准答案：契约、夹具、设计 token —— 见 spec/README.md
├── assets/                  # 跨端品牌资产（应用图标源文件）
├── docs/                    # 跨端文档：本文、RELEASES、PLAN、ENVIRONMENT、现状分析
└── README.md
```

- 每个平台目录**自成构建系统**：MSBuild、Cargo workspace、Gradle 互不感知。仓根不再有 `Cargo.toml`。
- egui 1.x 代码已从主干移除，留在分支 `legacy/egui-1.x`（D2）。
- 仓库级约定（提交规范、一个逻辑单元一个 commit）放在仓根 `AGENTS.md`，平台目录的 `AGENTS.md` 只写本平台。

## 核心原则

### 1. 共享答案，不共享实现

三端**不共享任何实现代码**，共享 `spec/` 里语言无关的规格与夹具。理由与 Ncrust 相同且更充分：

- PezMax 的客户端业务是 REST CRUD + 少量协议，三端各写一遍的成本低；
- FFI 共享核心的固定成本高：UWP 的 .NET Native 对原生库与反射都敏感，Android 要 JNI 绑定，Rust 端反而是最不需要绑定的一端；
- 真正要防的是「同一个契约三端各理解一次」和「一端修了另一端忘了」——夹具正好防这个。

规则（与 Ncrust `spec/README.md` 一致）：

1. **先改 spec，再改实现。** 契约或业务行为变更，先更新 `spec/` 的文档 / 夹具，再改各端。
2. **各端测试按相对路径直接读 `spec/`**，不复制夹具。
3. 夹具是 JSON 数据，每条带 `id` / `description` / `source`。
4. spec 与实现冲突时，以**实测正确的一方**为准修另一方，不为测试变绿去改夹具。

### 2. 后端契约是外部输入

PezMax-Go 由他人维护，本仓**只读消费**它的契约：

- `spec/api/` 记录的每个端点都标注**上游基线 commit**（当前 `PezMax-Go@f79a4e0`）。
- 上游契约变化时：先更新 `spec/api/` 与夹具（`source` 写上游 commit 与文件:行），再改各端。
- 发现上游缺陷（如存量文件 URL 指向内网 host），**在上游提 issue / PR**，客户端只做有期限的兼容层，
  并在 spec 里写明兼容层的撤除条件。
- 本仓不复制上游源码，也不依赖 Swagger 代码生成（.NET Native 下生成的反射式模型不可用）。

### 3. 每端一个无 UI 的业务核心

三端结构对称，便于对照与测试：

| 层 | Windows | Linux | Android |
|---|---|---|---|
| 业务核心（无 UI、可单测、跑 spec 夹具） | `PezMax.Core`（netstandard2.0） | `pezmax-core` crate | `core/` 包（纯 Kotlin，JVM 单测） |
| 应用（页面、平台集成） | `PezMax.App`（UWP） | `pezmax-app`（Kanesumi `TreeApp`） | `app/`（Compose） |

业务核心包含：HTTP 客户端与信封解析、会话、分页聚合、哈希缓存、文件树模型、下载 / 上传状态机、
URL 归一、通知可见性与已读、更新检查、表单校验。**页面里不写这些逻辑。**

核心的异步规则：Windows `ConfigureAwait(false)` 不假设 UI 线程；Linux 核心跑在 tokio 运行时，
结果经 channel 回到 Kanesumi 主循环并 `wake::request()` 唤醒；Android 用协程，`Dispatchers.IO` 在核心内部切换。

### 4. Kanesumi Design，平台原生优先

- 设计语言统一为 **Kanesumi Design**（正典：Ether 仓根 `KANESUMI_DESIGN.md`）：直角、无边框、信息优先、纯色无渐变、
  UWP 缓动（无弹簧回弹）。新文档写「Kanesumi Design」，不写「Metro Design」；`Metro*` 代码标识符保持原名。
- 动画曲线与时长的唯一来源是 Sokuou 及其各扇区分支；`spec/design/tokens.json` 只收录已在某端落地的值。
- **各端不逐像素对齐**，在同一语言内各自做最合适的设计。Windows 端照搬 Ncrust 的已验证取舍：
  平台控件优先（NavigationView、Pivot、ContentDialog…），只做资源键级覆盖；`ControlsResourcesVersion="Version1"`；
  按钮用微软原生样式；Segoe MDL2 图标。
- 动画只动视觉属性（位移 / 缩放 / 透明），不动布局：Windows 用 Composition，Linux 用 Kanesumi 合成时钟，
  Android 用单一 progress + `graphicsLayer`。

### 5. 本地数据不进服务端

服务端没有的状态由客户端本地持有，三端形状一致（键名与 JSON 结构写进 spec）：

| 数据 | 内容 | Windows | Linux | Android |
|---|---|---|---|---|
| 凭据 | Bearer token、用户 id | `PasswordVault` | Secret Service（不可用时 0600 文件） | Keystore 包裹的加密存储 |
| 设置 | 主题、下载目录、阅读偏好、服务器地址 | `LocalSettings` | `$XDG_CONFIG_HOME/pezmax/settings.json` | DataStore / SharedPreferences |
| 下载记录 | 本地路径、续传状态、完成时间 | `LocalFolder/downloads.json` | `$XDG_DATA_HOME/pezmax/downloads.json` | 应用私有目录 JSON |
| 通知已读 | `(userId, notifyId)` 集合 | `LocalFolder/notice_reads.json` | 同上目录 | 同上 |
| 缓存 | 文件树 / 排行 `{hash, payload}`、封面、阅读器页缓存 | `LocalCache` | `$XDG_CACHE_HOME/pezmax/` | `cacheDir` |

缓存一律可清除，清除不影响登录态与下载记录（延续 egui 版 `.cache/` 隔离约定）。

## 业务域与功能面

对齐 Electron 官方桌面端的功能面（PezMax-Go README §功能概览），按域划分；每个域在各端核心里是一个模块：

| 域 | 功能 | 主要端点（详见 `spec/api/endpoints.md`） |
|---|---|---|
| 账户 | 登录（验证码）、注册（含密保）、找回密码（两步）、登出、会话失效 | `/datum/user/{login,register,logout,captchaImage,securityQuestions,resetPasswordBySecurity,getInfo}` |
| 个人中心 | 资料、头像、改名、改密、密保三问三答、统计 | `/datum/desktop/user/profile/*` |
| 资料（试卷） | 文件树、列表、搜索、科目 / 学校、详情、上传、属主编辑删除 | `/datum/file/*`（文件树为 `/datum/file/tree`） |
| 电子书 | 列表、学科、在线阅读（PDF / EPUB）、下载、上传、删除 | `/datum/ebook/*`、`/datum/download/ebook` |
| 书签 | 列表、详情、CRUD、封面上传、外链打开 | `/datum/bookmark/*` |
| 收藏 | 资料 / 电子书 / 书签三类收藏与取消 | `/datum/favorite`、`/datum/ebook/favorite`、`/datum/bookmark/favorite`、`/datum/desktop/*/favorite/*` |
| 下载 | 断点续传、进度、取消、本地记录、服务端记录 | `/datum/download/*`、`/datum/desktop/download/*` |
| 举报 | 资料 / 电子书 / 书签举报、我的举报、处理时间线、撤回 | `/datum/report/*`、`/datum/ebook/report*`、`/datum/bookmarkReport/*` |
| 社区 | 上传排行（哈希缓存）、用户公开资料 | `/datum/user/rank`、`/datum/user/{userId}` |
| 通知 | 弹窗通知、滚动提醒、本地已读 | `/system/notification/user/{popup,scroll}` |
| 应用 | 更新检查（CrateStack）、设置、缓存管理、快捷键、开机启动 | GitHub Releases API |

管理端接口（用户管理、通知 CRUD、举报审核、缓存清理）**不在客户端范围内**。

### 阅读器：三端差异最大的部分

| 格式 | Windows | Linux | Android |
|---|---|---|---|
| PDF | `Windows.Data.Pdf.PdfDocument` 渲染到位图 | pdfium-render（继承 egui 版管线）→ Kanesumi 图像 | `android.graphics.pdf.PdfRenderer` |
| 图片 | `BitmapImage` | `image` crate 解码 → Kanesumi 图像 | Compose 图片加载 |
| TXT / Markdown | 原生文本 / Markdown 渲染（方案见平台文档） | `pulldown-cmark` → Kanesumi 段落 | 复用 Ncrust 的 Markdown 渲染思路 |
| EPUB | WebView2 + 本地打包的阅读器页面（M0 验证） | M1 交给系统阅读器（`xdg-open`），原生阅读器后置 | Readium 或 WebView（M0 验证） |
| MOBI / AZW3 | 下载后交给系统默认程序 | 同左 | 同左 |

PDF 视图沿用 egui 版的两种模式：**网格**（缩略图）与**连续**（纵向滚动 + 左侧概览），页面按需渲染、限并发、磁盘缓存。

## 跨端发布：CrateStack

发布沿用 Ncrust 的 **CrateStack 滚动多平台班次**，完整规范见 `RELEASES.md`。要点：

- tag `CrateStack-YYYYMMDD<A..Z>`，一班装载各平台当时的最新资产，附 `manifest.json`；
- 资产名 `pezmax-one-[平台]-[架构]-[版本].[扩展名]`（D4）；
- 各平台版本线独立、版本号各有唯一来源；应用内更新按资产名跨班次找严格更高版本，不看 Latest 徽章；
- 规则由 `spec/fixtures/update/` 锁死，三端跑同一份夹具。

## 立项决策

### 已定

2026-10-19 负责人确认全部建议（D2–D14），以下即为现行决策。

| # | 决策 | 结论 | 原因 |
|---|---|---|---|
| D0 | 客户端定位 | 原生客户端家族，与 Electron 官方端并存，消费同一 `/datum/*` | 后端由上游维护，本仓不分叉契约 |
| D1 | 技术栈 | Windows：C# + UWP + WinUI 2；Linux：Rust + Kanesumi；Android：Kotlin + Kanesumi-sec-a；macOS 暂缓 | 负责人决定；Kanesumi 主线已足够成熟，WinUI 2 经 arc-deck / Ncrust 验证 |
| D2 | egui 旧代码 | 主干删除；完整代码留在分支 `legacy/egui-1.x`（指向 `b83247a`）与 tag `v1.0.0` | 主干保持干净；可复用经验已记入各平台文档。✅ 已执行（`5d3e7d4`） |
| D3 | Kanesumi.Xaml 归属 | 从 Ncrust 抽成独立仓（暂名 `../Kanesumi-xaml`），Ncrust 与本仓都按相对路径引用 | 与 Kanesumi-sec-a 的组织方式一致；复制必然漂移，直接引用 Ncrust 目录会把两仓绑死。⏳ 跨仓操作，单独排一轮（`PLAN.md` M0） |
| D4 | 资产名前缀 | `pezmax-one-` | 与 1.0.0 资产连续，也和上游 Electron 端产物区分开 |
| D5 | 代码共享 | 不共享实现，共享 `spec/` | 见「核心原则 1」 |
| D6 | 发布 | CrateStack 班次 + `manifest.json` + 资产名扫描更新 | 延续 Ncrust 机制，三端同一套更新规则 |
| D7 | Android minSdk | 24 | 与 Ncrust 和 Kanesumi-sec-a 的验证范围一致；有具体需求再提 |
| D8 | Windows 运行时 | .NET Native，`LangVersion` 10，`x:Bind` only，无 DI / MVVM 框架，自研零反射 JSON | Ncrust 已在本机验证；.NET Native 对反射敏感 |
| D9 | Windows ARM64 | M1 只出 x86_64，M2 补 `aarch64` | 1.0.0 有 aarch64 用户；UWP 编译成本低，主要是验证成本 |
| D10 | Linux 会话 | 只支持 Wayland 会话（KDE Plasma、GNOME），不做 X11 | Kanesumi harness 只有 Wayland 外壳 |
| D11 | 强调色 | Windows 跟随系统强调色，读不到时回落钴蓝；Linux / Android 保留 5 色预设（钴蓝 `#3B82F6`、云杉 `#1DB954`、绯红 `#EF4444`、琥珀 `#F59E0B`、堇紫 `#8B5CF6`），默认钴蓝 | Windows 与 Ncrust 一致；预设取自 egui 版，保持品牌连续 |
| D12 | 更新下载源 | GitHub 为唯一真源；客户端提供可选镜像前缀（默认关闭），镜像拼接规则写进 `spec/fixtures/update` | PezMax 用户主要在国内，与 Ncrust 情况不同；默认关闭保证行为可预测 |
| D13 | 后端地址 | 默认地址随构建写入；「设置 → 高级」可改，改动后清会话 | 上游部署形态多样（内网测试服 / 公网 edge） |
| D14 | 平台顺序 | Windows 先行，spec 与 Core 随它定型；Linux、Android 随后 | 见 `PLAN.md` |

### 待提供（不是决策，是缺的信息）

| 项 | 用途 | 何时必须 |
|---|---|---|
| 生产后端地址 | D13 的默认值 | Windows M1 发布前 |
| Windows 包身份 `Publisher` | MSIX 签名；发布后永不改变 | Windows 首次发布前 |
| 只读测试账号 | `spec/tools` 探针取证 | M0 写夹具时 |

## 提交规范

沿用 Ncrust：Conventional Commits，小写类型前缀，中文主题；正文写**为什么**；一个逻辑单元一个 commit，主动提交。
monorepo scope：

| 范围 | 写法 |
|---|---|
| `windows/` | `feat(windows): …` |
| `linux/` | `feat(linux): …` |
| `android/` | `feat(android): …` |
| `spec/` | `docs(spec): …`（夹具也算规格） |
| 跨端文档、仓库杂务 | 不带 scope |

同时改了 spec 和一端实现时，scope 写那一端，正文说明其余两端的跟进状态。
