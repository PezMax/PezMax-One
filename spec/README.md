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
5. **后端契约以上游源码为准**。`api/` 下每份文档开头写明上游基线（当前 `PezMax-Go@f79a4e0`）；
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
| `api/endpoints.md` | 客户端用到的端点目录：方法、路径、门禁、参数、响应形状 | 📝 初稿（取自上游 Swagger） |
| `api/envelope.md` | 信封、错误、HTTP 状态码与会话失效 | 待写 |
| `api/auth.md` | 登录 / 注册 / 找回密码 / 密保三问三答流程与校验规则 | 待写（校验规则需对照上游源码） |
| `fixtures/envelope/` | 成功、业务错误、401、非 JSON 响应的解析结果 | 待写 |
| `fixtures/pagination/` | `TableDataInfo` 与其它分页形状的归一；`pageSize ≤ 100`；按 `total` 拉全量的终止条件 | 待写 |
| `fixtures/state-cache/` | 文件树 / 排行的哈希缓存协议：首次、未变化、已变化、本地缓存损坏 | 待写 |
| `fixtures/file-tree/` | 树响应 → 导航模型（学科 / 学校 / 类型 / 年份层级、节点 id、排序） | 待写 |
| `fixtures/url/` | 文件 URL 归一：相对路径、内网 host 改写、`data:` / `blob:` 原样。**过渡规则**，上游修库后按条撤除 | 待写 |
| `fixtures/download/` | 断点续传状态机：Range 续传、ETag 变化重下、416、空对象、取消、服务端限流 | 待写 |
| `fixtures/upload/` | 扩展名白名单、大小上限、哪些格式会转 PDF、TXT / MD 编码要求 | 待写 |
| `fixtures/notification/` | 5 类通知的可见窗口（发布 / 故障 / 维护时间段）、展示方式、本地已读按用户隔离 | 待写 |
| `fixtures/update/` | CrateStack 资产解析与版本比较（规则同 Ncrust，前缀换为 `pezmax-one-`，补多段扩展名） | 待写（M0 第一批） |
| `design/tokens.json` | 颜色、字号、缓动、时长、布局常量 | 待写（随第一个端落地） |
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
