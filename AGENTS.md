# AGENTS.md

<!-- BEGIN workbench-pointer · v1 · 勿手改本块（由 GuitaristRin/Projects 统一维护；改模板请改 templates/repo-AGENTS-block.md） -->
> **本文件只写本仓特有的规则。跨仓通用约定不在本仓重复**，唯一真源是工作台文档库：
> - Windows：`%USERPROFILE%\Documents\Projects\AGENTS.md`
> - Linux：`~/projects/AGENTS.md` → `~/projects/Arcturius/AGENTS.md`（按序取第一个存在的）
> - 远端：<https://github.com/GuitaristRin/Projects/blob/main/AGENTS.md>
>
> 动手之前先读它。**读不到就在报告里写明「工作台未读到」**，然后按本文件执行，不要猜。
> 新发现的通用规则先回流工作台（`docs/09-回流与同步.md`），本文件只留引用，不抄正文。
> 冲突优先级：用户当前明确指令 > 本仓特有规则 > 工作台通用规则。
<!-- END workbench-pointer -->

仓库级权威指南，面向在此工作的编码 agent（Claude Code、Codex 等）和人。`CLAUDE.md` 是本文件的镜像——
**两者不一致时以本文件为准**，改完本文件后整份同步过去。

## 这是什么

PezMax One：PezMax（拼图满绩，学习资料分享与整理）的原生客户端家族，Windows / Linux / Android 三端，
消费上游 PezMax-Go 后端的 `/datum/*` 契约，与上游的 Electron 官方客户端并存。

**状态（2026-10-19）**：技术栈转向刚立项，仓库已重组，**三端都还没有代码**。旧版 egui 1.x 客户端已从主干移除，
在分支 `legacy/egui-1.x`；不要把它的代码搬回主干，需要它的经验时去那个分支查。

## 先读什么

| 要做的事 | 读 |
|---|---|
| 任何工作 | `docs/ARCHITECTURE.md`（架构与立项决策 D0–D14，均已定） |
| 排期 / 下一步做什么 | `docs/PLAN.md`（「下一轮」一节） |
| 在某个平台目录里工作 | 该目录的 `AGENTS.md`——**它是该目录的权威指南** |
| 改后端契约、协议、业务规则 | `spec/README.md`，然后 `spec/api/` |
| 发布 | `docs/RELEASES.md` |
| 工具链 | `docs/ENVIRONMENT.md` |

## 仓库布局

```
windows/   C# + UWP + WinUI 2（MSBuild，仅 Release|x64）
linux/     Rust + Kanesumi（独立 Cargo workspace，只在 Linux / Wayland 上构建）
android/   Kotlin + Compose + Kanesumi-sec-a（Gradle，composite build 引用 ../../Kanesumi-sec-a）
spec/      三端共享的规格与 JSON 夹具
docs/      跨端文档
assets/    品牌资产
```

三个平台目录各自是独立的构建系统，互不感知；仓根没有 `Cargo.toml`、`build.gradle.kts` 或 `.sln`。

## 铁律

1. **不共享实现，共享答案。** 三端之间不共享代码；共享的是 `spec/`。**先改 spec，再改实现**；
   各端测试按相对路径读 `spec/`，不复制夹具；不为让测试变绿去改夹具。
2. **后端契约是外部输入。** PezMax-Go 由他人维护，本仓只读消费。`spec/api/` 标注上游基线 commit（当前 `f79a4e0`）；
   上游变了先改 spec。上游的问题去上游提，客户端只做写明撤除条件的兼容层。
3. **业务逻辑进各端的无 UI 核心**（`PezMax.Core` / `pezmax-core` / `core/`），页面里不写协议与状态机。
4. **Kanesumi Design。** 新文档写「Kanesumi Design」，不写「Metro Design」；`Metro*` 代码标识符保持原名。
   动画曲线只来自 Sokuou 及其扇区分支。
5. **WinUI 3 / Windows App SDK 不在考虑范围内**，不要引入，不要提议迁移。
6. **不要写死版本号**，各平台版本只有一个来源（见 `docs/RELEASES.md`）。
7. 文档与提交信息用中文。

## 提交规范

Conventional Commits，小写类型前缀，中文主题；正文（空一行后）写**为什么**，一年后不开 PR 也能读懂。

| 前缀 | 用途 |
|---|---|
| `feat:` | 用户可见的新能力 |
| `fix:` | 修复缺陷 |
| `refactor:` | 不改行为的结构调整 |
| `perf:` | 纯性能优化 |
| `docs:` | 只改文档 |
| `build:` | 构建系统、依赖、版本号 |
| `chore:` | 杂务（删文件、改目录、gitignore） |
| `style:` | 格式与空白 |

- **一个逻辑单元一个 commit。** 必须一起编译通过的改动是一个单元；能独立回滚的分开提交。
- Scope：`windows/` → `(windows)`，`linux/` → `(linux)`，`android/` → `(android)`，`spec/` → `docs(spec): …`；
  跨端文档与仓库杂务不带 scope。同时改 spec 与一端实现时，scope 写那一端，正文说明另两端的跟进状态。
- 发布用 CrateStack 班次 tag `CrateStack-YYYYMMDD<A..Z>`；旧 tag `v1.0.0` 保留。
- 历史只在 `git log` 里，**不要另建 `*_log.md`**。
- 推送到 `origin` 前先问负责人。
