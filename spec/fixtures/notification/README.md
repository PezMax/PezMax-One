# `fixtures/notification/` —— 通知的客户端侧判定

> 协议事实以 **PezMax-Go@061a888** 为准：`k_admin/internal/kadmin/datum_notification.go:287-342`、
> `k_admin/internal/kadmin/modules/datum/notification_repository.go:21-61`。

服务端已经做了**两件**事：按可见窗口筛（`status='0'` + 发布窗口）、按 `display_mode` 分通道。
所以客户端**不要**再判断时间段。客户端真正要判定的是这份夹具里的四件事：
哪条通知在哪个通道展示、弹窗队列怎么排、已读怎么记、`hash` 怎么用。

## 服务端已经定好的（客户端照做，不要重算）

| 事实 | 依据 |
|---|---|
| 可见：`status='0'` 且（`publish_start` 为空或 ≤ now）且（`publish_end` 为空或 ≥ now）；`now` 是服务端本地时间字符串 `yyyy-MM-dd HH:mm:ss` | `notification_repository.go:21-25` |
| 弹窗通道：`display_mode='0'`，且 `notify_type='4'` 时**只给 `upload_user_id` 等于会话用户的那一条** | `:41-45` |
| 滚动通道：`display_mode='1'`，**没有 type 4 过滤** | `:52-56` |
| 排序：`sort DESC, notify_id DESC` | `:45`、`:56` |
| 空串即「不设窗口」：时间字段为 `NULL` 时响应里是 `""` | `datum_notification.go:58-81` |
| `hash` 是**内容 SHA-256**：时间窗口推进会让它变，不需要谁去失效缓存 | `:283-309` |

`fault_start_time` / `fault_end_time` / `maintenance_*` / `remind_before_minutes`
**只存不判**——服务端没有任何代码读它们，客户端只能拿来做展示文案（比如维护倒计时）。
响应里也**没有** `forceUpdate` / `updateDownloadUrl`：所谓「强制更新」在契约里不存在，
要做只能靠客户端自己比版本，不能靠通知字段。

## 四个决策点

| kind | 输入 | 输出 |
|---|---|---|
| `channel` | 一条通知 + 当前 userId + 它来自哪个通道 | `show`：要不要展示 |
| `popup-plan` | 弹窗通道的行 + 本地「已弹过」集合 + 当前 userId | 依次要弹出的 `notifyId` 列表 |
| `read-set` | 本地已读记录 + 当前 userId + 目标 notifyId | `read`：算不算已读 |
| `feed-cache` | 上次的 hash 与「本地有没有有效 payload」+ 本次响应 | `keep` / `replace` / `refetch` / `keep-and-warn` |
| `dismissible` | 一条通知 + 当前时间 | `dismissible`：用户能不能关掉 |

## 规则与理由

**channel**：弹窗通道的东西服务端已经按 type 4 定向过，来了就展示。滚动通道**没有**那层定向
（`notification_repository.go:52-56`），所以滚动条里的 `notify_type='4'` 只在 `uploadUserId` 等于自己时展示——
否则等于把「某人的资料被下架」推给所有人。这是**客户端兜底**，不是契约；上游哪天给 scroll 也加上
定向，这条兜底就删掉（撤除条件写在下面）。

**popup-plan**：`notify_type` 为 `'1'`（版本更新）与 `'4'`（资料下架）的，用户确认过就记进
「已弹过」集合，下次启动不再弹（旧端同款语义：`PezMax-Desktop/src/renderer/views/home/index.vue:279-290`）。
`'2'` 故障、`'3'` 维护、`'5'` 公告**每次都弹**：故障还没好就该再提醒一次。
顺序沿用服务端返回顺序，同一 `notifyId` 只弹一次。

**read-set**：已读**只在本地**，服务端没有 per-user 已读模型（`ptmj_notification` 无 `is_read` 列）。
键是 `(userId, notifyId)`：换账号不能串，同一个 `notifyId` 对 A 已读不影响 B。

**feed-cache**：与 `envelope.md` §5 的哈希缓存同款，但通知的 hash 是**内容哈希**：
`unchanged=true` 时 `data` 为 `null`，必须沿用本地；本地没有可用 payload 时不要显示空列表，
**不带 hash 重取一次**。

**dismissible**：`'3'`（维护）在 `maintenanceStartTime` 之后不可关闭；其余可关闭。
`'1'` 的强制更新**不由通知字段决定**——契约里没有 `forceUpdate`，见上一节。

## 形状

```json
{
  "id": "notification.channel.scroll-type4-other-user",
  "description": "滚动通道不排除 type 4：别人的下架通知不能在滚动条里出现",
  "source": "PezMax-Go@061a888 k_admin/internal/kadmin/modules/datum/notification_repository.go:52-56",
  "input": {
    "kind": "channel",
    "userId": 7,
    "channel": "scroll",
    "notification": { "notifyId": 9, "notifyType": "4", "uploadUserId": 42, "displayMode": "1" }
  },
  "expect": { "show": false }
}
```

- 通知对象只用与判定相关的字段，其余字段（`title` / `content` / 时间）不参与判定。
- `expect` 里的字段按 kind 取：`channel`→`show`；`popup-plan`→`notifyIds`；`read-set`→`read`；
  `feed-cache`→`action`（可带 `hash`）；`dismissible`→`dismissible`；
  `storage`→`reads` / `popups`（解析出来的条数）。

## 本地存储

「已读」与「已弹过」都存在本地一个文件里（Windows 为 `LocalFolder/notice_reads.json`），形状：

```json
{
  "reads":  [ { "userId": 7, "notifyId": 10 } ],
  "popups": [ { "userId": 7, "notifyId": 11 } ]
}
```

- `reads`：用户点开过的（铃铛未读计数靠它）。
- `popups`：弹窗确认过的（type 1 / 4 的一次性语义靠它，见 `popup-plan`）。
- 两个集合的键都是 `(userId, notifyId)`；换账号不串。
- 文件读不出来（丢了 / 手工改坏 / 旧版本写的）→ **当空集合**，不要因此挡住界面：
  通知状态丢了顶多再弹一次，比「一打开就报错」轻。
- 写入整体重写，不做增量；文件很小，冲突不值得设计。

## 撤除条件

- `channel` 里对滚动通道 type 4 的兜底：**上游给 `ActiveScroll` 补上 type 4 定向后删掉**
  （同时删 `notification.channel.scroll-type4-*` 两条用例）。
- `popup-plan` 的「已弹过」集合：上游若提供 per-user 已读/已推送模型，改读服务端，本地集合退化为缓存。
- `read-set`：同上；目前服务端**永远**没有已读字段，别去找。

## 不覆盖的

- 通知内容的图文排版、图标选择（`notifyType` → 图标）属于各端视觉，不进夹具。
- 管理端 CRUD（`POST/PUT/DELETE /datum/notification*`）需要管理 JWT，客户端无权限。
