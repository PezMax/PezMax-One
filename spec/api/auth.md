# 账户与认证流程

> **上游基线**：`PezMax-Go@061a888`（2026-11-04）。依据：`k_admin/internal/kadmin/datum_auth.go`、
> `datum_user_auth.go`、`security.go`、`session.go`。上游变化时先改本文，再改各端。
>
> 响应信封、错误与 401 的统一规则见 `envelope.md`；本文只写账户流程与校验规则。

## 1. 验证码

### `GET /datum/user/captchaImage`（匿名）

```json
{
  "code": 0,
  "message": "ok",
  "msg": "ok",
  "data": {
    "captchaEnabled": true,
    "uuid": "9f0c…",
    "img": "/9j/4AAQSkZJRg…"
  }
}
```

- `captchaEnabled` **恒为 `true`**。桌面端验证码不受管理端 `security.captcha_enabled` 开关影响
  （`datum_auth.go:35-51`）；客户端不要用 `false` 分支，也不要去读 `/api/system/config/login`。
- `img` 是**不带 `data:` 前缀**的 base64 JPEG；客户端自行拼 `data:image/jpeg;base64,`。
- `uuid` 是不透明标识；验证码一次性，默认有效期 120 秒（管理端可配 30–600 秒，
  `security.go:100-113`）。过期或已用后必须重新获取。
- 获取失败回 `503`「验证码获取失败，请稍后重试」。

登录、注册、找回密码第一步共用这一套验证码；每次提交都会消耗 `uuid`。

## 2. 登录

### `POST /datum/user/login`（匿名）

请求：

```json
{ "username": "alice", "password": "secret", "code": "123456", "uuid": "9f0c…" }
```

成功：

```json
{
  "code": 0,
  "message": "ok",
  "msg": "ok",
  "data": { "token": "64位十六进制不透明串", "expiresAt": 1760000000000 }
}
```

- `expiresAt` 是 **Unix 毫秒**；token 默认有效期 120 小时，可由服务端环境变量
  `KADMIN_DATUM_SESSION_TTL` 覆盖（`datum_user_auth.go:25-55`）。
- 之后所有会话请求带 `Authorization: Bearer <token>`。
- 上游没有 refresh token / 续期端点；过期或不合法就重新登录。

失败（按上游判定顺序）：

| 条件 | HTTP | 文案 |
|---|---|---|
| `username` 或 `password` 为空 | 400 | 用户名或密码错误 |
| 验证码错误 / 过期 / `uuid` 无效 | 400 | 验证码错误或已过期 |
| 验证码服务不可用 | 503 | 验证码服务暂不可用，请稍后重试 |
| 账号或 IP 已锁定 | 429 | 登录失败次数过多，账号已锁定，请稍后再试 |
| 安全存储不可用 | 503 | 登录安全存储不可用 |
| 用户不存在（也计一次失败） | 400 | 用户名或密码错误 |
| 账号被停用（`status == "0"`） | 403 | 账号已被停用 |
| 密码错误（计失败，可能触发锁定） | 400 / 429 | 用户名或密码错误 / 账号已锁定… |
| 创建会话失败 | 503 | 会话创建失败，请稍后重试 |

客户端规则：

- **不要**把「用户名或密码错误」细分出「用户不存在」；上游故意合并。
- 429 不自动重试；显示锁定文案，等用户手动再试。
- token 存平台安全存储（Windows `PasswordVault`、Linux Secret Service、Android Keystore）；
  设置文件里只存 userId 等非敏感信息。

## 3. 注册

### `POST /datum/user/register`（匿名）

请求：

```json
{
  "username": "alice",
  "password": "secret",
  "confirmPassword": "secret",
  "securityQuestionOne": "母亲的姓名？",
  "securityAnswerOne": "张三",
  "securityQuestionTwo": "第一所学校？",
  "securityAnswerTwo": "实验小学",
  "securityQuestionThree": "最喜欢的城市？",
  "securityAnswerThree": "济南",
  "avatar": "",
  "code": "123456",
  "uuid": "9f0c…"
}
```

成功：`data = { "userId": 7 }`。**不自动登录**，客户端注册完成后回登录页。

校验（`datum_user_auth.go:588-659`）：

| 规则 | 失败文案 |
|---|---|
| `username` 去首尾空白后非空 | 用户名不能为空 |
| `username` 按 **Unicode 字符**计 ≤ 64 | 用户名格式错误 |
| `password` 按 **字节**计 5–64 | 密码格式错误：长度需在 5 到 64 位之间 |
| `password == confirmPassword` | 两次输入的密码不一致 |
| 三问均非空 | 密保问题不能为空 |
| 三答均非空 | 密保答案不能为空 |
| `avatar` ≤ 255 **字节** | 头像地址格式错误 |
| 验证码有效 | 验证码错误或已过期（400）/ 验证码服务暂不可用（503） |
| 用户名未被占用 | 用户名已存在，请更换后重试（409） |

- 账号、密码、密保在一条 SQL 中原子创建；不会出现「有账号无密保」。
- 密码与密保答案在服务端用 BCrypt 存储，客户端不要自行哈希。
- 用户名规则只限长度；没有字符集限制，界面层不要擅自加「仅字母数字」。

## 4. 找回密码（两步）

验证码只在第一步使用；第一步成功后，服务端签发一张**一次性重置工单**（10 分钟，绑定 `uuid + username`），
第二步凭工单重置（`datum_user_auth.go:115-150`）。

### 第一步：`GET /datum/user/securityQuestions`（匿名）

查询参数：`userName`、`uuid`、`code`。

成功：

```json
{
  "code": 0,
  "message": "ok",
  "msg": "ok",
  "data": [
    { "question": "母亲的姓名？" },
    { "question": "第一所学校？" },
    { "question": "最喜欢的城市？" }
  ]
}
```

| 条件 | HTTP | 文案 |
|---|---|---|
| `userName` 为空 | 400 | 用户名不能为空 |
| 验证码错误 / 过期 | 400 | 验证码错误或已过期 |
| 用户不存在 | 400 | 用户不存在 |
| 账号查询 / 密保查询失败 | 500 | 账号查询失败 / 密保查询失败 |
| 工单存储不可用 | 503 | 身份存储不可用 |

- 正常账号返回 3 条；客户端按实际条数渲染，不要假定一定有 3 条。
- 第一步已消耗验证码；界面此时不得再拿旧 `uuid` 去取验证码。

### 第二步：`POST /datum/user/resetPasswordBySecurity`（匿名）

请求：

```json
{
  "username": "alice",
  "code": "123456",
  "uuid": "9f0c…",
  "securityAnswerOne": "张三",
  "securityAnswerTwo": "实验小学",
  "securityAnswerThree": "济南",
  "newPassword": "newsecret",
  "confirmPassword": "newsecret"
}
```

成功：`data = true`。

| 条件 | HTTP | 文案 |
|---|---|---|
| `username` / `uuid` / `code` 任一为空 | 400 | 验证码错误或已过期 |
| 任一密保答案为空 | 400 | 密保答案不能为空 |
| 新密码不在 5–64 字节 | 400 | 密码格式错误：长度需在 5 到 64 位之间 |
| 两次新密码不一致 | 400 | 两次输入的密码不一致 |
| 工单不存在 / 已用 / 过期 | 400 | 验证码错误或已过期 |
| 用户不存在 | 400 | 用户不存在 |
| 密保答案不匹配 | 400 | 密保答案不正确 |
| 身份存储 / 写库失败 | 503 / 500 | 身份存储不可用 / 密码重置失败 |

- `code` 在第二步只做**非空校验**；真正凭证是第一步签发的工单。客户端仍应原样回传第一步的
  `uuid` 与非空 `code`，不要伪造。
- 工单一次性：第二步失败后（例如答案错），需要重新走第一步拿新验证码与工单。

## 5. 当前用户与登出

### `GET /datum/user/getInfo`（会话）

```json
{
  "code": 0,
  "message": "ok",
  "msg": "ok",
  "data": {
    "user": {
      "userId": 7,
      "userName": "alice",
      "nickName": "alice",
      "avatar": "",
      "count": 3,
      "status": "1"
    },
    "roles": ["ptmj_user"],
    "permissions": []
  }
}
```

- `userId` 用于 `/datum/desktop/*/{userId}` 路径；必须与会话用户一致。
- `nickName` 现在恒等于 `userName`；界面不要假设有独立昵称。
- `status` 是字符串：`"1"` 正常，`"0"` 停用。
- 账号行不存在回 `403`「账号不存在」；会话无效回 `401`。

### `POST /datum/user/logout`（会话）

成功：`data = true`，服务端删除 Redis 会话。客户端无论成功与否都清本地凭据；若回 401，本来已失效。

## 6. 会话失效的额外触发条件

除过期与登出外，以下情况会让**已签发的旧 token 立即失效**：

- 用户被删除或停用（每次请求都会重新查账号状态，`datum_user_auth.go:436-460`）。
- **密码被修改或重置**：会话里绑定了密码哈希，密码一变，旧 token 校验失败。
  因此：
  - `PUT /datum/desktop/user/profile/password` 成功后，客户端应主动清会话并回登录页；
  - `PUT /datum/desktop/user/profile/password/by-security` 同样；
  - 找回密码成功后，其他设备上的会话也会失效。

客户端不要尝试在改密后继续用旧 token；下一次请求必然 401。

## 7. 夹具与后续

账户流程的错误目前以本文的文案表为准。等三端核心开始写认证模块时，再补
`fixtures/auth/`（校验矩阵与两步找回的工单语义）；先不要在没有夹具的情况下把文案写死进界面。
