# spec/ —— 跨平台标准答案

PezMax-One 的三个端（Windows `windows/`、Linux `linux/`、Android `android/`）**不共享实现代码，共享答案**。
本目录存放与语言无关的规格与测试夹具：后端契约怎么调、信封怎么解、分页怎么拼、哈希缓存什么时候沿用本地、
续传遇到 ETag 变化怎么办、文件 URL 怎么归一、哪条通知该显示、更新该选哪个资产、设计 token 取什么值。

> 为什么不做共享核心库：见 `docs/ARCHITECTURE.md`「核心原则 1」。

## 规则

1. **先改 spec，再改实现。** 契约或业务行为的变更，先更新这里的文档 / 夹具，再改各端代码。
   只改了一端时，commit 正文写明其余两端的跟进状态。
2. **各端测试直接读本目录**（相对路径引用），**不复制夹具**。复制即漂移。
3. **夹具是数据不是代码**：JSON，UTF-8，每条用例带 `id`（全局唯一）、`description`（这条在防什么）、
   `source`（依据：上游 commit + 文件:行、实测记录或决策出处）。
4. **spec 与实现冲突时**：以实测正确的一方为准修正另一方（可能是 spec 本身），commit 里说明判断依据。
   不要为了让测试变绿去改夹具。
5. **后端契约以上游源码为准**。`api/` 下每份文档开头写明上游基线（当前 `PezMax-Go@061a888`）；
   上游变化时先改这里，`source` 写新 commit。上游文档与上游源码矛盾时，以源码为准并在此记录。
6. `design/tokens.json` 只收录**已在某端代码中落地**的取值并注明来源；某端独有的取值写在该端自己的文档里。

## 夹具格式

```json
{
  "id": "state-cache.unchanged.keeps-local",
  "description": "服务端回 unchanged=true 时必须沿用本地 payload，不得清空列表",
  "source": "PezMax-Go@f79a4e0 k_admin/internal/kadmin/datum_file.go:179-195",
  "platforms": ["windows", "linux", "android"],
  "input": { },
  "expect": { }
}
```

- 同一主题的用例放在同一文件的数组里；`input` / `expect` 的形状在该子目录 `README.md` 里写明。
- `platforms` 可选，缺省为三端。只适用于部分端的用例必须写明原因。

## 目录与状态

| 路径 | 内容 | 状态 |
|---|---|---|
| `api/endpoints.md` | 客户端用到的端点目录：方法、路径、门禁、参数、响应形状 | 📝 M1 读接口与账户 / 个人中心已补齐；写接口随实现补 |
| `api/envelope.md` | 信封、错误、HTTP 状态码与会话失效 | ✅ 已按源码写定 |
| `api/auth.md` | 登录 / 注册 / 找回密码 / 密保三问三答流程与校验规则 | ✅ 已按源码写定 |
| `api/desktop-reference.md` | 旧 Electron 官方端的调用方式（参数怎么写、字段读哪个）+ 它的过时用法清单 | ✅ 参考手册；契约仍以 `endpoints.md` 为准 |
| `fixtures/envelope/` | 成功、业务错误、401、非 JSON 响应的解析结果 | ✅ 初版 13 例 |
| `fixtures/pagination/` | `TableDataInfo` 与其它分页形状的归一；`pageSize ≤ 100`；按 `total` 拉全量的终止条件 | ✅ 初版 11 例 |
| `fixtures/auth/` | 注册 / 两步找回 / 登录的表单校验矩阵（文案与判定顺序逐条对齐上游 handler） | ✅ 初版 25 例 |
| `fixtures/state-cache/` | 哈希缓存的本地状态机：键按 (服务器, 资源) 隔离、损坏缓存不可用、并发响应按序号落地 | ✅ 初版 17 例 |
| `fixtures/file-tree/` | 树响应 → 导航模型（层级、文件夹在前、面包屑、`fileId` 不从 `id` 解析、服务端兜底值原样显示） | ✅ 初版 15 例 |
| `fixtures/url/` | 文件 URL 归一：相对路径、内网 host 改写、`minio://`、`data:` / `blob:` 原样。**过渡规则**，上游 ETL 后按条撤除 | ✅ 初版 21 例 |
| `fixtures/download/` | 断点续传决策表：`before` / `response` / `after` 三个决策点，`Range`、`If-Range`、`206`、`416`、ETag 失效 | ✅ 初版 20 例 |
| `fixtures/upload/` | 两套上传规则（资料 / 电子书）、字节 vs rune 的边界、转档与 200MB 硬上限、批量逐个结果 | ✅ 初版 28 例 |
| `fixtures/notification/` | 通道判定、弹窗队列与一次性语义、本地已读按用户隔离、`hash` + `unchanged` 缓存 | ✅ 初版 25 例 |
| `fixtures/update/` | CrateStack 资产解析与版本比较（前缀 `pezmax-one-`，多段扩展名，`amd64` 不算 `x86_64`，镜像前缀） | ✅ 初版 23 例 |
| `fixtures/reading/` | PDF 阅读器的视图几何与渲染调度：连续视图的页顶偏移 / 可见范围 / 当前页，±2 页渲染窗口与渲染顺序，缩放步进与适应宽度，页缓存键，网格列数，缓存释放顺序 | ✅ 初版 48 例 |
| `design/tokens.json` | 颜色（深 / 浅 / 高对比三档）、圆角、字号、布局不变量；只收已落地的值 | ✅ 首版（来源：Windows `Themes/PezMax.xaml`） |
| `tools/` | 零依赖的契约探针脚本（只读请求；写操作的脚本必须在文件头注明「会改账号数据」） | 待写 |

消费方：

| 端 | 测试位置 | 接入时间 |
|---|---|---|
| Windows | `windows/tests/PezMax.Core.Tests`（net9.0 + xUnit） | M1，与 Core 同步编写 |
| Linux | `linux/crates/pezmax-core/tests/`（`cargo test`） | Linux M1 |
| Android | `android/app/src/test`（JUnit） | Android M1 |

## 与 Ncrust spec 的关系

`fixtures/update/` 的规则与 Ncrust `spec/fixtures/update/` 同源（CrateStack 是两个项目共用的发布机制）。
现阶段各自维护一份；若两边规则开始需要同步修改，再考虑把 CrateStack 规格抽成独立仓库。
