# `fixtures/auth/` —— 账户表单校验与两步找回

`validation.json` 是**客户端提交前的表单校验矩阵**，与上游 handler 的校验分支逐条对应。
依据：`spec/api/auth.md`，上游基线 `PezMax-Go@061a888`。

## 为什么客户端也要校验

后端会拒，但客户端在提交前先拦一遍能省一次往返、把验证码留给真正的提交。代价是**文案与顺序必须跟上游逐字一致**：
上游文案直接透传到界面（`DatumException.Message`），客户端自己另编一套就会出现「同一错误两种说法」。

## 形状

```json
[
  {
    "id": "auth.register.password.mismatch",
    "description": "两次输入的密码不一致时按上游文案拦截",
    "source": "PezMax-Go@061a888 k_admin/internal/kadmin/datum_user_auth.go:602-604",
    "input": {
      "action": "register",
      "username": "alice",
      "password": "12345",
      "confirmPassword": "54321",
      "securityQuestionOne": "q1",
      "securityQuestionTwo": "q2",
      "securityQuestionThree": "q3",
      "securityAnswerOne": "a1",
      "securityAnswerTwo": "a2",
      "securityAnswerThree": "a3",
      "avatar": "",
      "code": "123456",
      "uuid": "uuid-fixture"
    },
    "expect": { "error": "两次输入的密码不一致" }
  }
]
```

- `input.action`：`register` / `reset` / `login`，决定用哪组字段与哪个校验函数。
  - `register`：`username`、`password`、`confirmPassword`、`securityQuestionOne..Three`、`securityAnswerOne..Three`、
    `avatar`、`code`、`uuid`（对应 `RegisterRequest`）。
  - `reset`：`username`、`code`、`uuid`、`securityAnswerOne..Three`、`newPassword`、`confirmPassword`
    （对应 `ResetPasswordRequest`）。
  - `login`：`username`、`password`。
- `expect.error`：`null` 表示校验通过；字符串表示**必须完全相等**的提示文案。
- 缺省字段按空串处理。

## 顺序即契约

上游是 `switch` 逐条判定（`datum_user_auth.go:592-614`、`717-730`），**第一条命中的分支决定文案**。
所以夹具里每个用例只制造一个违规点；同时制造多个违规的用例会锁死顺序，改动顺序就得改夹具，得不偿失。

## 计数口径

| 字段 | 上游口径 | 出处 |
|---|---|---|
| 用户名 | 先 `TrimSpace`，再按 **rune** 计 ≤ 64 | `datum_user_auth.go:591-598` |
| 密码 | 按 **字节**（`len(string)`）计 5–64 | `datum_user_auth.go:599-601`、`724-726` |
| 头像 | 按**字节** ≤ 255 | `datum_user_auth.go:611-613` |
| 密保问题 / 答案 | `TrimSpace` 后非空 | `datum_user_auth.go:605-610`、`721-723` |

用户名**没有字符集限制**：中文、空格、标点都可以，只有长度上限。界面不要擅自加「仅字母数字」。

## 不在这里覆盖的

- **验证码是否正确 / 是否过期**：需要服务端状态，属于集成测试。夹具只覆盖「字段缺失」这一层
  （`code` / `uuid` 为空 → 文案与验证码错误相同）。
- **用户名已存在（409）**、**密保答案不匹配**、**重置工单过期**：都取决于服务端存储，同样留给集成测试
  （见 `spec/tools/`，尚未建立）。
- 第二步的 `code` 只做非空校验，真正凭证是第一步签发的工单（`datum_user_auth.go:731-741`）。
