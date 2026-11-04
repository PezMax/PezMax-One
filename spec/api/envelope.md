# 信封、错误与会话失效

> **上游基线**：`PezMax-Go@061a888`（2026-11-04）。依据：`k_admin/internal/kadmin/transport/httpx/response.go:8-24`、
> `k_admin/internal/kadmin/datum_user_auth.go:462-489`、`k_admin/internal/kadmin/datum_file.go:179-195`、
> `k_admin/internal/kadmin/datum_notification.go:287-308`。上游变化时先改本文与 `fixtures/envelope/`，再改各端。

三端必须用同一套规则解析响应、判定错误与处理 401。各家 HTTP 库不同，**判定顺序与结果不能不同**。

## 1. 三种响应形状

### 1.1 成功对象

```json
{ "code": 0, "message": "ok", "msg": "ok", "data": { } }
```

- HTTP 状态恒为 `200`；`code` 恒为 `0`。
- `data` 一定存在，可以为 `null`、`true`、对象或数组。
- `message` 与 `msg` 恒为 `"ok"`；**客户端不读成功文案**。

### 1.2 成功列表（RuoYi `TableDataInfo`）

```json
{ "code": 0, "message": "ok", "msg": "ok", "rows": [], "total": 0 }
```

- 列表**没有 `data`**；分页结果在顶层 `rows` / `total`。
- `total` 是过滤后的总条数，不是本页条数。
- 旧版 Electron 端还兼容过 `data: {items, total}` 的 kadmin 分页（`PezMax-Desktop/src/renderer/utils/request.js`
  `normalizeKAdminPage`）；`/datum/*` 现行源码已全部改为顶层 `rows` / `total`，新客户端**不再兼容旧形状**。

### 1.3 错误

```json
{ "code": 400, "message": "用户名或密码错误", "msg": "用户名或密码错误", "data": null }
```

- HTTP 状态**就是** `code`（`fail` 把 `status` 同时写进两处）。
- `message` 与 `msg` 内容相同；客户端优先读 `msg`，为空再读 `message`。
- `data` 恒为 `null`。

### 1.4 非信封响应

- `GET /datum/file/content`：`text/plain; charset=utf-8` 正文，状态 200；不是 JSON。
- `GET /datum/download/file`、`GET /datum/download/ebook`、`GET /datum/ebook/content`：文件流，
  `Content-Type` 随格式变化，支持 `Range` / `ETag` / `416`；不是 JSON。
- `OPTIONS *`：`204 No Content`，无正文（仅 CORS 预检，客户端通常不主动发）。
- 网关 / 反代故障时可能出现 HTML、纯文本或空响应。

## 2. 解析算法（必须按此顺序）

```
1. 取 HTTP 状态 status 与响应头 Content-Type。
2. 若该请求预期是文件流（下载 / 预览）：走流式处理，不解析 JSON，不套用下面的信封规则。
3. status == 401：
   - 无论正文能否解析，都按「会话失效」处理（见 §4）。
4. 若 Content-Type 不是 application/json（允许带 charset）：
   - 归为传输错误；保留会话；展示「服务暂不可用（HTTP {status}）」，5xx 可重试。
5. 解析 JSON 失败：
   - 归为传输错误；保留会话；按 status 生成文案。
6. status 不是 2xx：
   - 归为业务错误；取 msg → message → 按 status 的兜底文案；
   - 保留会话（403 / 409 / 429 都不是会话失效）。
7. JSON 中有 `rows`（数组）：
   - 归为列表成功；**不要**因为缺少 `data` 报错；
   - `total` 缺失时按本页 `rows.length` 兜底（防御性规则，见 `fixtures/pagination/total-missing`）。
8. JSON 中 `code == 0`：
   - 归为对象成功；`data` 缺失按 `null`。
9. JSON 中 `code != 0`（HTTP 2xx）：
   - 上游不会这样返回，但按业务错误处理，**不得**当作成功。
10. 其余形状（既无 rows/total 也无 code）：
   - 归为传输错误；保留会话。
```

补充：

- 不因 `message` / `msg` 的具体文字做流程分支；只有 **HTTP 状态**（尤其 401）决定会话行为。
- 响应按 UTF-8 解码；`Content-Type` 的 charset 可忽略。
- 请求头默认带 `Accept-Encoding: gzip`（上游启用 gzip），由 HTTP 库透明解压。
- JSON 请求体用 `Content-Type: application/json`；multipart 由平台库自行生成 boundary，**不要**手写该头。
- 客户端只发 `Authorization: Bearer <token>`。上游还接受 `Access-Token` / `X-Access-Token` / `?token=`，
  但那是兼容入口，新客户端不用（`k_admin/internal/kadmin/session.go:108-127`）。

## 3. 状态码与客户端行为

| HTTP | 上游含义 | 客户端行为 | 清会话 |
|---|---|---|---|
| 200 | 成功对象 / 列表 / 文件流 | 正常处理 | 否 |
| 400 | 参数、验证码、密码、密保答案不合法 | 显示服务端文案；表单留在原地 | 否 |
| 401 | token 缺失、过期、用户被删或被停用 | 关闭覆盖层，回登录层；保留当前页以便登录后返回 | **是** |
| 403 | 没有权限、userId 与会话不符、账号不存在 / 停用（登录时） | 显示服务端文案；不跳登录 | 否 |
| 404 | 资源不存在或已删除、接口不存在 | 显示文案；列表项可刷新 | 否 |
| 409 | 用户名已存在、重复收藏、重复举报、幂等冲突 | 显示文案；按业务提示处理 | 否 |
| 413 | 上传 / 请求体 / 文本预览过大 | 显示文案；不要自动重试 | 否 |
| 429 | 登录失败次数过多，临时锁定 | 显示文案；按建议等待 | 否 |
| 5xx | 服务端 / 存储 / Redis 故障 | 显示「服务暂不可用」；可重试；有旧缓存时继续显示旧缓存 | 否 |

**403 不是会话失效。** 只有 401 清会话；把 403 当 401 会把「没有权限」误判成掉登录。

### 3.1 错误文案兜底

服务端没给 `msg` / `message` 时，各端用自己的本地化文案，语义对应：

| HTTP | 兜底语义 |
|---|---|
| 400 | 请求有误 |
| 403 | 没有权限 |
| 404 | 内容不存在 |
| 409 | 操作冲突 |
| 413 | 内容过大 |
| 429 | 操作过于频繁，请稍后再试 |
| 5xx | 服务暂不可用 |

## 4. 会话失效（401）

```text
收到 401
  → 删除本地凭据（token + userId）
  → 关闭登录层以外的覆盖层（阅读器、详情、弹窗通知等）
  → 打开登录层，记住当前页面 / 路由
  → 登录成功后回到记住的位置
  → 同一时间只处理一次；多个并发 401 不得弹出多个登录层
```

- 账号被停用时上游回的是 **401**，文案为「账号已被停用，请重新登录」
  （`datum_user_auth.go:470-484`）；客户端照常清会话，并把该文案显示出来。
- Redis / 身份存储故障回 **503**「身份存储不可用」；**不清会话**，提示稍后重试。
- 登录页自身的请求（登录、验证码、找回密码）若回 401，不弹「会话失效」，按普通错误显示。

## 5. 哈希缓存信封

文件树、上传排行、通知喂给共用同一套「带 hash 的成功对象」：

```json
{ "code": 0, "message": "ok", "msg": "ok", "data": null, "hash": "…", "unchanged": true }
```

| 字段 | 说明 |
|---|---|
| `hash` | 不透明字符串。树 / 排行是每次重算生成的 16 位十六进制；通知是内容 SHA-256 的 64 位十六进制。客户端**只比较，不解析** |
| `unchanged` | `true` 表示客户端带来的 `?hash=` 与服务端当前一致，`data` 为 `null` |
| `data` | `unchanged=false` 时为完整 payload；`unchanged=true` 时为 `null` |

客户端规则：

1. 首次请求不带 `hash`，保存 `hash` 与 payload。
2. 之后带 `?hash=<上次的 hash>`。
3. `unchanged=true` → **沿用本地 payload**；不得把 `null` 写进缓存、不得清空列表。
4. `unchanged=false` → 用新 payload 与新 hash 整体替换。
5. `unchanged=true` 但本地没有可用 payload（缓存被清 / 首次启动）→ **立刻不带 hash 重取一次**。
6. 请求失败（5xx / 网络）→ 继续显示旧 payload；不要用失败结果覆盖缓存。
7. hash 是空串或缺失 → 当作未命中，按完整响应处理。

来源：`datum_file.go:29-195`（树 / 排行）、`datum_notification.go:287-308`（通知）。

## 6. 分页

分页参数与归一规则见 `fixtures/pagination/README.md`。要点：

- `pageNum` / `page`（前者优先）、`pageSize` / `size`（前者优先）。
- 默认 `pageNum=1`、`pageSize=20`；**单页上限 100**，超出按 100 处理（不报错）。
- 需要全量时按 `total` 逐页拉，见 `fixtures/pagination/` 的 `fetch-all` 用例。
- 列表端点没有 `data`，不要拿 `data.rows` 去解析。

## 7. 夹具

`fixtures/envelope/cases.json` 覆盖：成功对象、成功列表、错误、非 JSON、401（JSON 与非 JSON）、
HTTP 200 但 `code != 0`、哈希缓存命中 / 未命中、文件流。夹具是唯一判据；本文与夹具冲突时先改本文并说明依据。
