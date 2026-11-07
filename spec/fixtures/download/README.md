# `fixtures/download/` —— 断点续传状态机

> 协议事实以 **PezMax-Go@061a888** 为准：`k_admin/internal/kadmin/datum_activity.go:520-579`。
> `PezMax-Desktop/src/main/index.js:595-700` 是**旧接口时代**的参考实现，只借用它的判断顺序，
> 不当作契约来源。

下载流 `GET /datum/download/file?fileId=` 把对象体交给 `http.ServeContent`（`datum_activity.go:568-573`），
于是自动获得四件事：强 `ETag`（`"datum-<fileId>-<size>"`，`:562-566`）、`Range` → `206`、
`If-Range` 匹配校验、越界回 `416`。本夹具把这些事实抽成**与 I/O 无关的决策表**，
三端各自照着决策、各自写自己的字节搬运。

## 为什么要有这张表

续传的坑都不在「写文件」，而在「什么时候不能续」：

- 服务端文件被替换（ETag 变了）→ `ServeContent` 忽略 `If-Range`，直接回 **200 全量**；
  此时若还追加，拼出来就是坏文件；
- 半成品其实已经完整、只是没改名 → 再要 `Range` 会收到 `416`；
- 半成品没有可信 ETag（元数据丢了 / 只有弱 ETag）→ `If-Range` 送不出去，等于从头下；
- 上游每次请求都插一条下载记录（`:549-552`，注释 `:570` 明确接受 Range 也记），
  所以「记录条数」不代表「文件个数」。

## 三个决策点

| 决策点 | 输入 | 输出 |
|---|---|---|
| `before` | 本地半成品（字节数 + 记录的 ETag） | 起始偏移、是否带 `Range` / `If-Range`、是否先删半成品 |
| `response` | 本地半成品 + 响应（状态码、`ETag`、`Content-Length`、`Content-Range`） | 动作：`start` / `resume` / `restart` / `fail` / `session-expired` |
| `after` | 已写入的字节数 + 总大小 | `continue` / `complete`（改名）|

## 动作语义

| 动作 | 含义 |
|---|---|
| `start` | 从 0 开始写；不存在半成品时的正常开局 |
| `resume` | 追加写到已有半成品，偏移 = 半成品字节数 |
| `restart` | **删掉**半成品与元数据，从 0 重下 |
| `complete` | 字节数已等于总大小，原子改名为最终文件 |
| `fail` | 终止并报错；**保留**半成品，用户可再试（5xx / 网络中断都走这里）|
| `session-expired` | 401：清会话、回登录层；半成品保留，重新登录后还能续（`:466-486` 的会话闸门回的就是 401）|

## 形状

```json
{
  "id": "download.response.206-valid",
  "description": "半成品 1024 字节、ETag 一致、Content-Range 覆盖到末尾：可以续",
  "source": "PezMax-Go@061a888 k_admin/internal/kadmin/datum_activity.go:562-573",
  "input": {
    "kind": "response",
    "part": { "size": 1024, "etag": "\"datum-12-4096\"" },
    "response": {
      "status": 206,
      "etag": "\"datum-12-4096\"",
      "contentLength": 3072,
      "contentRange": "bytes 1024-4095/4096"
    }
  },
  "expect": { "action": "resume", "offset": 1024, "total": 4096, "discardPart": false }
}
```

- `input.kind`：`before` / `response` / `after`。
- `part.size = 0` 表示没有半成品；`part` 整体缺省等价于 `{size:0}`。
- `response` 里缺省的字段表示响应没有该头。
- `expect`：`action` 必有；`offset` / `total` / `discardPart` 缺省按 0 / 0 / false 断言；
  `before` 场景还可断言 `range` / `ifRange` 两个请求头。

## 续传的额外约定

- **文件名**：半成品 `<最终文件名>.part`，元数据同目录 `<最终文件名>.part.json`（只存 `etag`）。
  元数据缺 `etag` 时按「无法校验」处理，直接 `restart`，不要发半个 `If-Range`。
- **`Accept-Encoding: identity`**：下载请求显式要求不压缩。Go 侧不自动 gzip，但反代 / 边缘可能加，
  一旦压缩，`Content-Length` 与实际字节数就对不上，续传偏移会错位。
- **只有 `restart` 删半成品**；401 / 5xx / 用户取消都保留。
- **并发**：默认串行。经 edge 部署时每 IP 只允许 1 个下载连接（`deploy/nginx.conf.template:86-93`），
  并发 2 个会被拒，错误看起来像「下载失败」，所以队列必须在 Core 里就是串行的。
- **「我的下载」按 `fileId` 去重**：服务端记录是「请求次数」口径，客户端展示是「文件」口径。

## 本地记录与「我的下载」合并

本地记录写在 `downloads.json`（`ARCHITECTURE.md` 核心原则 5），形状：

```json
{
  "records": [
    {
      "kind": "file", "id": 12, "fileName": "期中试卷.pdf",
      "localPath": "C:\\Users\\me\\Downloads\\PezMax\\期中试卷.pdf",
      "size": 4096, "bytesReceived": 4096, "etag": "\"datum-12-4096\"",
      "status": "done", "updatedAt": "2026-11-06T21:30:00+08:00"
    }
  ]
}
```

- `kind`：`file` / `ebook`（两者 `id` 的命名空间独立，同一个 12 不冲突）。
- `status`：`part`（有半成品，可续）、`done`（已落盘改名）、`failed`（失败但保留半成品）。
- `etag` 与 `bytesReceived` 是续传的依据；`done` 之后保留它们无害。
- `localPath` 只对本机有意义，跨设备不同步（服务端也没有这个字段）。

**合并规则**（服务端 `/datum/desktop/download/list/{userId}` + 本地记录）：

1. 以服务端行为骨架（它有题目 / 科目 / 类型等元信息），顺序沿用服务端。
2. 按 `fileId`（电子书按 `ebookId`）把本地记录挂上去：有本地路径的行才能「打开 / 在文件夹中显示」。
3. 本地有、服务端没有的（服务端记录被清过、或那次请求没落库）**也要显示**，追加在服务端行之后，
   按 `updatedAt` 倒序——否则用户刚下完的东西会凭空消失。
4. 同一 `fileId` 有多条本地记录时取 `updatedAt` 最新的那条。
5. 服务端 `total` 与 `rows` 口径不一致（见 `spec/api/endpoints.md`），分页只看 `rows` 条数。

## 不覆盖的

- 多段 `Range`（`multipart/byteranges`）：`ServeContent` 不产生，客户端也不必支持。
- 磁盘写失败、权限拒绝：属于本端 I/O，各端自己处理。
- 服务端 404 的三种成因（未过审且非属主、对象键无法解析、对象不存在，`:532-546`）：
  归为 `fail`，保留半成品，不重试。
