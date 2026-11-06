# `fixtures/state-cache/` —— 哈希缓存的本地状态机

树、排行、通知喂共用「带 hash 的成功对象」这套协议，服务端侧见 `datum_file.go:181-195`、
`datum_notification.go:287-309`，信封层面的规则见 `spec/api/envelope.md` §5。
这份夹具不重复那些规则，只写**客户端本地这一侧**：键怎么算、读出来怎么算「可用」、
两个响应抢着落地时听谁的。

## 与通知的区别（不要混）

| | 树 / 排行 | 通知 feed |
|---|---|---|
| hash 怎么来 | 服务端生成一个随机 token，**写操作时失效重建**（`datum_file.go:141-180`、`:260-262`） | 每次现算内容的 SHA-256（`datum_notification.go:283-286`） |
| 什么时候变 | 有人上传 / 审核 / 删除 | 内容变了，或者发布窗口跨过了边界 |
| 客户端要做的 | 一样：带上次 hash，`unchanged` 就沿用本地 | 一样 |

差别只在「为什么不带 hash 重取」的触发频率，客户端逻辑是同一套。

## 三个决策点

| kind | 输入 | 输出 |
|---|---|---|
| `key` | 服务器地址 + 资源名 + 本地已有条目 | `hit` / `miss` |
| `load` | 缓存条目的原始内容（可能损坏） | `usable`：能不能当 payload 用 |
| `apply` | 本地状态 + 响应 | `store` / `keep` / `refetch` |
| `stale` | 已应用的最大请求序号 + 本次响应的序号 | `apply`：要不要落地 |

## 规则与理由

**key**：键是 `(serverUrl, resource)`。**换服务器必须未命中**——设置里改后端地址后，
旧服务器的树 / 排行不能拿来渲染，否则用户看到的是另一个站点的数据（D13 要求改地址后清会话，
缓存同理）。同一个服务器的 `tree` 与 `rank` 互不影响。

**load**：本地缓存是「文件里的一段 JSON」，随时可能损坏（写一半断电、手工改坏）。
判定可用的条件缺一不可：文件存在、能解析、`hash` 非空、payload 非 null。
不满足就当没有缓存——**不要**把坏数据当空列表渲染（那会让「树空了」和「没加载」无法区分）。

**apply**：
- 首次（无 hash）：保存 `hash` + payload。
- `unchanged=true`：沿用本地 payload，**不要**把 `data:null` 写进缓存。
- `unchanged=true` 但本地不可用：立刻不带 hash 重取一次（不是显示空）。
- `unchanged=false`：整体替换。
- 请求失败（5xx / 网络）：保留旧 payload，**不要**用错误结果覆盖缓存。

**stale**：同一个资源可能同时有两个请求在飞（切页 + 定时刷新）。给每个请求一个递增序号，
只接受**序号不小于已应用序号**的结果。先发后到的旧响应必须丢弃——否则会拿旧 hash 覆盖新的，
下一次请求又带着旧 hash 去问，服务端回全量，白白多传一遍。

## 形状

```json
{
  "id": "state-cache.key.server-changed",
  "description": "换后端地址后，旧服务器的树缓存必须未命中",
  "source": "spec/ARCHITECTURE.md 核心原则 5 + D13（改服务器地址清会话）",
  "input": {
    "kind": "key",
    "serverUrl": "https://new.example.com",
    "resource": "tree",
    "entries": [
      { "serverUrl": "https://old.example.com", "resource": "tree", "hash": "h1", "payload": [] }
    ]
  },
  "expect": { "hit": false }
}
```

- `apply` 的输入：`previous`（`hash` + `usable`）+ `response`（`kind` 为 `success` / `failure`、
  `unchanged`、`hash`、`data`），输出 `action` 与可选的 `hash`。
- `stale` 的输入：`appliedSeq`（已落地的最大序号）+ `incomingSeq`，输出 `apply`。

## 撤除条件

协议本身没有「过渡」性质，不存在撤除；只有 hash 算法演进时（比如换成 ETag）整份重做。

## 不覆盖的

- 缓存容量 / 过期淘汰策略：各端按自己的缓存目录实现，不进夹具。
- 服务端 Redis 故障导致 503：属于错误路径，见 `spec/api/endpoints.md` 的树 / 排行说明，
  客户端「有旧缓存就继续显示」这条已在 `apply` 的 `failure` 用例里。
