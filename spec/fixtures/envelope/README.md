# fixtures/envelope

信封解析的跨端判据。规则正文见 `spec/api/envelope.md`；本目录只放数据。

## 文件

| 文件 | 内容 |
|---|---|
| `cases.json` | 成功对象 / 列表 / 错误 / 非 JSON / 401 / 哈希缓存 / 文件流 |

## 用例形状

```jsonc
{
  "id": "envelope.success.object",          // 全局唯一
  "description": "这条在防什么",             // 一句话，写给一年后的人
  "source": "PezMax-Go@f79a4e0 …:行号",      // 依据
  "platforms": ["windows", "linux", "android"], // 可省略，缺省三端
  "input": {
    "status": 200,                          // HTTP 状态码
    "contentType": "application/json; charset=utf-8",
    "body": "原始响应正文字符串",             // 文本响应
    "bodyBase64": "…",                      // 二进制响应用这个，二者取一
    "requestKind": "json"                   // 可选；缺省 json，文件流写 stream
  },
  "expect": { }
}
```

### `expect.kind`

| kind | 含义 | 相关字段 |
|---|---|---|
| `success` | 成功对象 | `code`、`data`、`session` |
| `list` | 成功列表 | `code`、`rows`、`total`、`dataAbsent`、`session` |
| `error` | 业务错误（HTTP 非 2xx 或 `code != 0`） | `code`、`message`、`session`、`retryable` |
| `transport` | 传输错误（非 JSON、解析失败、网络） | `status`、`messageClass`、`session`、`retryable` |
| `session-expired` | 401 | `code`、`message`、`session=clear`、`rememberRoute` |
| `state` | 哈希缓存信封 | `unchanged`、`hash`、`data`、`keepLocalPayload`、`session` |
| `binary` | 文件流 | `contentType`、`notJson`、`session` |

### `session`

| 值 | 含义 |
|---|---|
| `keep` | 保留本地凭据，不跳登录 |
| `clear` | 删除本地凭据并按会话失效流程回登录，`rememberRoute=true` 时记住当前页 |

### `message` 的比对口径

- `error` / `session-expired`：`message` 是服务端 `msg`（为空时取 `message`）；空串表示客户端使用本地兜底文案。
- `transport`：不比对具体文案，只比对 `messageClass`（`service-unavailable` / `bad-response`）。
- 实现可以只断言 `kind` / `code` / `session` 等关键字段，但不得跳过 `session`。

## 边界

- `body` 是**原始字符串**，不要预解析后写对象；这样能锁住空白、转义与 `data:null` 等细节。
- 夹具不覆盖平台 HTTP 库的 gzip 解压与重定向细节；这些属于平台适配层，不在跨端契约内。
- 上游若新增信封字段，先改 `spec/api/envelope.md` 与本目录，再改三端；不要为让测试变绿去改夹具
  （`spec/README.md` 规则 4）。
