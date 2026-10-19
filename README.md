# PezMax One · 拼图满绩·绫

> PezMax 学习资料平台的**原生客户端家族**：Windows · Linux · Android，统一 Kanesumi Design。

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

PezMax（拼图满绩）是面向学习资料分享与整理的平台：试卷、课件、资源书签与电子书。
PezMax One 是它的原生客户端，与上游 [PezMax-Go](https://github.com/PezMax/PezMax-Go) 自带的 Electron 官方客户端**并存而非替代**，
消费同一套后端接口，以 Kanesumi Design（直角、无边框、信息优先）带来平台原生的体验。

## 状态

**2026-10-19 起技术栈全面转向，正在重建。** 旧版 egui 客户端 1.0.0 已停止维护（代码在分支
[`legacy/egui-1.x`](https://github.com/PezMax/PezMax-One/tree/legacy/egui-1.x)），它对接的旧后端契约已被 PezMax-Go 取代。

| 平台 | 技术栈 | 状态 |
|---|---|---|
| Windows | C# + UWP + WinUI 2（.NET Native） | 规划中，首个开发平台 |
| Linux | Rust + Kanesumi（Wayland） | 规划中 |
| Android | Kotlin + Jetpack Compose + Kanesumi-sec-a | 规划中 |
| macOS | —— | 暂缓 |

各平台最新版本与下载见 [`docs/RELEASES.md`](docs/RELEASES.md)。发布采用 CrateStack 滚动班次，
**请不要以 GitHub 的 Latest 标签判断某个平台的最新版**。

## 仓库结构

```
windows/   Windows 客户端（UWP + WinUI 2）
linux/     Linux 客户端（Rust + Kanesumi）
android/   Android 客户端（Kotlin + Compose）
spec/      三端共享的规格与测试夹具：后端契约、协议、更新规则、设计 token
docs/      架构、发布、排期、开发环境
assets/    品牌资产
```

三端**不共享实现代码，共享 `spec/` 里的答案**——同一条规则三端跑同一份夹具。

## 文档

| 文档 | 内容 |
|---|---|
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | 架构总纲与立项决策 |
| [`docs/PLAN.md`](docs/PLAN.md) | 里程碑与排期 |
| [`docs/RELEASES.md`](docs/RELEASES.md) | 发布索引与 CrateStack 规范 |
| [`docs/ENVIRONMENT.md`](docs/ENVIRONMENT.md) | 各平台开发环境 |
| [`spec/README.md`](spec/README.md) | 规格与夹具约定 |
| `windows/` · `linux/` · `android/` 下的 `AGENTS.md` | 各平台设计与工程指南 |

## 许可证

MIT，见 [LICENSE](LICENSE)。
