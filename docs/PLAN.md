# 排期与里程碑

写于 2026-10-19，技术栈转向立项时。按「先验证风险、再定型契约、一端先行、其余跟进」排序。
立项决策已于 2026-10-19 全部拍板（`ARCHITECTURE.md` D0–D14）；开发环境现状见 `ENVIRONMENT.md`。

## 总览

| 阶段 | 内容 | 产出 | 依赖 |
|---|---|---|---|
| **M0 · 地基与验证** | 仓库重组、spec 骨架、三端风险验证 | 新目录结构；`spec/api` 首批文档；`spec/fixtures/update` 等首批夹具；三端 spike 报告 | ✅ 决策已定；仓库重组已完成 |
| **M1 · Windows MVP** | 能日常浏览、阅读、下载 | 第一班 CrateStack（Windows 1.1.0） | M0 的 W1–W5 通过 |
| **M2 · Windows 对齐官方端** | 补齐 Electron 端功能面 | Windows 1.2.x | M1 |
| **M3 · Linux MVP → 对齐** | 用定型后的 spec 实现 Linux | Linux 1.1.0 起 | M0 的 L1–L5 通过；Windows M1 定型的 spec |
| **M4 · Android MVP → 对齐** | 同上 | Android 1.1.0 起 | M0 的 A1–A3 通过 |

**为什么 Windows 先行**：用户多在 Windows；UWP 平台控件最多、实现最快，能最早把 spec 与 Core 的形状定下来；
Linux 与 Android 随后照 spec 实现，少走弯路。（D14 已定为 Windows 先行。）

## M0 · 地基与验证

### 仓库重组 ✅（2026-10-19）

| 步骤 | 状态 |
|---|---|
| egui 1.x 留档到分支 `legacy/egui-1.x`（`b83247a`），主干移除 `src/`、`Cargo.toml`、`build.rs`、`wix/`、`build/`、macOS workflow | ✅ `5d3e7d4` |
| 移除 Java / Electron 时代的 `repowiki/` | ✅ `7788b86` |
| 设计文档与 `spec/` 入库 | ✅ |
| 建 `windows/`、`linux/`、`android/` 骨架，平台文档迁为各目录 `AGENTS.md`；品牌图标移入 `assets/`，desktop 文件移入 `linux/packaging/` | ✅ |
| 重写仓根 `README.md`、`AGENTS.md`、`CLAUDE.md` | ✅ |
| 推送到 `origin` | ⏳ 本地已提交，待负责人确认后推送 |

### 下一轮（新 session 从这里开始）

1. **下载链路**：`DownloadService` 接 `BackgroundDownloader`（Bearer + `Range` 续传），本地记录 `downloads.json`，
   与服务端「我的下载」按 `fileId` 合并；同时跑 W1。依据：`spec/api/endpoints.md` 的 `GET /datum/download/file`。
2. **通知本地已读**：`spec/fixtures/notification/` 尚未写，先补夹具（5 类通知的可见窗口、`popup` / `scroll` 的
   `hash` + `unchanged` 协议、已读集合按 `userId` 隔离），再做外壳铃铛与滚动条。
3. **PDF 网格 / 连续两种视图 + W2**；随后 EPUB（W3）。
4. **Kanesumi.Xaml 抽仓（D3）**：从 `../Ncrust/windows/src/Kanesumi.Xaml` 抽出到 `../Kanesumi-xaml`（保留历史可用
   `git filter-repo --subdirectory-filter`），Ncrust 改为相对路径引用并验证 MSBuild Release 构建通过；Ncrust 侧单独提交。
5. **发布工具 `windows/tools/release/`**：签名证书、`.appinstaller`、`.cer`、`manifest.json`。
   没有它连「装包冒烟」都做不了——.NET Native 的 IL 布局注册上去会在激活期崩溃（见 `windows/AGENTS.md`）。
6. Linux / Android spike（L1–L5、A1–A3）。

### spec 首批

| 项 | 内容 | 状态 |
|---|---|---|
| `spec/api/endpoints.md` | 补齐 M1 范围端点的参数与响应形状（账户、资料、下载、收藏、通知、个人资料） | 🟡 读接口已补，写接口随实现 |
| `spec/api/envelope.md` | 信封、错误、401 | ✅ |
| `spec/fixtures/update/` | 自 Ncrust 规则移植，前缀换为 `pezmax-one-`，补 `.pkg.tar.zst` 多段扩展名、`amd64` 不算 `x86_64` 等用例 | 待写 |
| `spec/fixtures/envelope/`、`pagination/`、`auth/` | M1 必需的协议与表单夹具 | ✅ 13 + 11 + 25 例 |
| `spec/fixtures/state-cache/`、`url/`、`download/`、`notification/` | 哈希缓存、URL 归一、续传状态机、通知可见性 | 待写（随对应实现） |
| `spec/tools/` | 只读契约探针（登录用测试账号，不写数据），用来给夹具取 `source` | 待写；需要测试账号 |

### 风险验证（spike）

| # | 平台 | 验证项 | 详见 |
|---|---|---|---|
| W1–W5 | Windows | 后台下载续传、`Windows.Data.Pdf` 大文件、WebView2 EPUB、内网访问、Kanesumi.Xaml 引用 | `windows/AGENTS.md` |
| L1–L5 | Linux | GNOME / Plasma 窗口与 IME、pdfium 纹理、仓外引用 Kanesumi、portal、异步唤醒 | `linux/AGENTS.md` |
| A1–A3 | Android | composite build、`PdfRenderer`、EPUB 方案 | `android/AGENTS.md` |

任何一项不通过，回到对应平台文档重新评估方案，记录结论后再进入下一阶段。
spike 代码不进主干，结论写进平台文档（「规格是资产，跑规格的工程不是」——Kanesumi `PORT_ROADMAP.md`）。

## 功能面分档（上游 061a888 核对）

上游 `docs/pezmax-next-tasks.md` / `docs/pezmax-remaining-tasks.md` 写于后端重置之前，
**只当「意图清单」用**；凡涉及「现在到底有没有」的判断，以当时的 Go 代码为准。
逐条核对（文档说法 → 代码事实 → 结论，带 `文件:行号`）记在 `docs/research/upstream-backlog-061a888.md`——
那里是排期输入，不是契约；契约看 `spec/api/`。

M1 之外按三档排：

| 档 | 条目 | 说明 |
|---|---|---|
| **必须有** | 电子书只读版（列表 / 详情 / 预览 / 收藏 / 下载）、举报提交 + 我的举报时间线、排行（含 503 降级）、下载取消 + 串行队列 | 后端全就绪，缺的是客户端；后两项也是上游已知投诉的根源 |
| **应当有** | 启动可中断（并行 + 超时降级 + 骨架屏）、统一用户详情、书签完整能力（封面二步上传）、URL 兜底改写 | 前两项是架构件，越晚做返工越大 |
| **可以没有** | AI 助手 / 知识库、mobi / azw3 内嵌预览、多连接加速、电子书独立 QoS | 见 `ARCHITECTURE.md`「也不在范围内」 |

**顺带能捡的**（文档说没做、代码其实已经能用）：电子书全链路、年份区间 `yearFrom`/`yearTo`、
`/datum/knowledge/*`、封禁即时失效（401）、我的下载字段 `filePath` —— 不必再等后端。

**不要对外承诺的**（上游仍缺）：Redis 故障时排行 / 树 503 无降级；下载无服务端取消语义；
「TA 的公开文件列表」端点不存在；匿名用户投影里 `remark` / `status` 未收敛；举报审核**不封号**。

## M1 · Windows MVP

进度（2026-11-06）：

| 项 | 状态 |
|---|---|
| **Core 判定表**（账户 / URL / 续传 / 通知 / 树 / 缓存 / 更新 / 上传） | ✅ 八份夹具各有实现，Core 测试 30 个用例全绿 |
| 账户：登录（验证码）、注册（三步）、找回密码（两步）、登出、会话失效处理 | ✅ 流程与界面已接通；待测试服务器手工回归 |
| 资料：文件树（哈希缓存）、列表、搜索、详情、PDF / 图片 / TXT 预览 | 🟡 树 / 列表 / 搜索 / 详情 / 三种预览已接；`TreeNavigator` 与 `FeedCache` 就位待接线；PDF 网格与连续两种视图未做 |
| 收藏、个人中心、设置 | 🟡 页面已接；缓存清理未做 |
| 通知：弹窗、滚动提醒、本地已读 | ✅ 外壳已接（铃铛未读 / 滚动条 / 弹窗队列 / `notice_reads.json`），判定在 Core；待真机回归 |
| 下载：续传、串行队列、进度与取消、本地记录 +「我的下载」 | 🟡 `.part` 续传 + 串行队列 + `downloads.json` +「我的下载」合并已接；下载浮层（进度 / 取消）与目录授权未做，W1 未验证 |
| 更新检查：启动后异步查 + 设置里手动查 | 🟡 设置页「检查更新」与镜像前缀已接（`ReleaseFeed` + `UpdateChecker`）；启动时自动查（24 小时节流）未做 |
| 发布：`windows/tools/release/` 与第一班 CrateStack | ⏳ 未开始；没有签名证书，装包与冒烟都还跑不了 |

范围：

- 账户：登录（验证码）、注册（三步）、找回密码（两步）、登出、会话失效处理。
- 资料：文件树（哈希缓存）、列表、搜索、详情、PDF / 图片 / TXT 预览。
- 下载：续传、串行队列、进度与取消、本地记录 + 「我的下载」。
- 收藏：资料收藏与「我的收藏」。
- 通知：弹窗、滚动提醒、本地已读。
- 个人中心：资料、头像、统计。
- 设置：外观、下载目录、缓存、服务器地址、关于 + 检查更新。
- 发布：`windows/tools/release/`（编译 + 签名 + `.appinstaller` + `.cer` + `manifest.json`），第一班 CrateStack。

验收：注册 → 登录 → 浏览 → 预览 → 下载（断网续传）→ 收藏 → 我的下载 / 收藏可见 → 登出 → 401 恢复，全链路在测试服务器上手工回归一遍；
Core 测试全绿，且全部读 `spec/`。

## M2 · Windows 对齐官方端

电子书（列表、学科、PDF / EPUB 在线阅读、下载、收藏、举报）、书签（CRUD、封面、收藏、举报、外链）、
上传（多文件 / 文件夹、元信息表单、待审提示）、举报与处理时间线、我的上传 / 举报、社区排行、密保三问三答与改密、
Markdown 预览、快捷键补全、开机启动、ARM64 包（D9）。

## M3 / M4

Linux 与 Android 各自按「MVP（同 Windows M1 范围）→ 对齐（同 Windows M2 范围）」两段推进；
开工时以当时的 `spec/` 为准，发现 spec 缺口先补 spec 再写实现。Linux 的 EPUB 原生阅读器不在 M3 范围内。

## 等负责人提供

- 生产后端地址与测试服务器访问方式（测试服务器在内网 `10.*`，UWP 需 `privateNetworkClientServer`）。
- Windows 包身份 `Publisher`（首次发布前必须定，之后永不改变）。
- 只读测试账号（用于 spec 探针取证，不在上游产生垃圾数据）。
- 是否在上游 PezMax-Go 登记本项目为第三方客户端，以及契约变更的通知渠道（建议关注上游 `k_admin/internal/kadmin/docs/swagger.json` 的变更）。
- Linux 构建机：本机未装 WSL，见 `ENVIRONMENT.md`。

## 风险验证（spike）

| # | 平台 | 验证项 | 详见 |
|---|---|---|---|
| W1–W5 | Windows | 后台下载续传、`Windows.Data.Pdf` 大文件、WebView2 EPUB、内网访问、Kanesumi.Xaml 引用 | `windows/AGENTS.md` |
| L1–L5 | Linux | GNOME / Plasma 窗口与 IME、pdfium 纹理、仓外引用 Kanesumi、portal、异步唤醒 | `linux/AGENTS.md` |
| A1–A3 | Android | composite build、`PdfRenderer`、EPUB 方案 | `android/AGENTS.md` |

任何一项不通过，回到对应平台文档重新评估方案，记录结论后再进入下一阶段。
spike 代码不进主干，结论写进平台文档（「规格是资产，跑规格的工程不是」——Kanesumi `PORT_ROADMAP.md`）。

## 风险

| 风险 | 影响 | 应对 |
|---|---|---|
| 上游契约变动频繁（PezMax-Go 仍在第二阶段开发，见其 `docs/pezmax-next-tasks.md`） | 三端反复返工 | spec 锁定上游基线 commit；每次升级基线是独立的 `docs(spec)` 提交并跑全部夹具 |
| 存量文件 URL 指向内网 host（上游任务四未完成） | 预览 / 下载失败 | 客户端过渡改写规则进 `spec/fixtures/url`，写明撤除条件 |
| EPUB 渲染三端方案不一 | 体验差异、维护三套 | Windows / Android 尽量共用同一套 WebView 阅读器页面；Linux 先交给系统阅读器 |
| GNOME 无服务端装饰 | Linux 窗口无标题栏 | L1 验证；自绘标题栏 |
| Kanesumi.Xaml 抽仓涉及 Ncrust | 跨仓改动、节奏耦合 | D3 定案后单独排一轮，Ncrust 侧改引用作为独立提交 |
| edge 每 IP 1 个下载连接 | 并发下载被拒、用户以为失败 | 客户端默认串行队列；错误码明确映射为「排队中」而非失败 |
