# 端点目录

> **上游基线**：`PezMax-Go@f79a4e0`（2026-10-16）。来源：`k_admin/internal/kadmin/docs/swagger.json`（摘要）
> 与 `k_admin/internal/kadmin/datum_*.go`（路由与门禁）。上游变化时先改本文，再改各端。
>
> 初稿只列**路径、门禁与用途**。参数与响应形状在各端实现对应模块时逐条补齐，并同步补夹具；
> 补齐一条，在「形状」列标 ✅。

## 通用约定

- **Base URL**：站点根地址（后端或 edge），客户端自行拼 `/datum/*`，**不要追加 `/api`**。默认值随构建写入，设置里可改（`ARCHITECTURE.md` D13）。
- **鉴权**：`Authorization: Bearer <token>`，token 来自登录响应；HTTP 401 → 清会话、回登录页（保留当前页以便登录后返回）。
- **信封**：`{code, data, message, msg}`，`code = 0` 为成功；错误优先读 `msg`，其次 `message`。详见 `envelope.md`（待写）。
- **列表**：`{code, rows, total, msg}`（RuoYi `TableDataInfo`）；分页参数 `pageNum` + `pageSize`，默认 20，**上限 100**。
- **哈希缓存**：标注 `#` 的端点接受 `?hash=`，未变化时回 `unchanged: true` 且不带 payload（`datum_file.go:29-195`）。
- **流式**：标注 `⇣` 的端点返回文件流，支持 `Range` / `ETag`。
- 门禁：**匿名**；**会话**（`requireDatumAuth`）；**可选会话**（不带 `userId` 时匿名且只出已审核内容，带 `userId` 时要求会话且必须是本人，
  `datum_file.go:296-312`）；**管理**（管理端 JWT，客户端不调用，列出只为避免误用）。

## 账户 `/datum/user`

| 方法 | 路径 | 门禁 | 用途 | 形状 |
|---|---|---|---|---|
| GET | `/datum/user/captchaImage` | 匿名 | 登录验证码：`{captchaEnabled, uuid, img}`，`img` 为不带前缀的 base64 JPEG | |
| POST | `/datum/user/login` | 匿名 | 登录：`{username, password, code, uuid}` → token | |
| POST | `/datum/user/register` | 匿名 | 注册（含密保三问三答，原子创建） | |
| POST | `/datum/user/logout` | 会话 | 登出，服务端作废 token | |
| GET | `/datum/user/getInfo` | 会话 | 当前用户信息（取 `userId`，后续 `/datum/desktop/*/{userId}` 用） | |
| GET | `/datum/user/securityQuestions` | 匿名 | 找回密码第一步：校验账号与验证码，返回密保问题 | |
| POST | `/datum/user/resetPasswordBySecurity` | 匿名 | 找回密码第二步：验重置工单与密保答案后重置 | |
| GET | `/system/config/login` | 匿名 | 公开登录配置（是否启用验证码等，待核） | |

## 个人中心 `/datum/desktop/user/profile`（会话）

| 方法 | 路径 | 用途 |
|---|---|---|
| GET | `/profile` | 个人资料 |
| GET | `/profile/stats` | 上传 / 下载 / 收藏计数 |
| PUT | `/profile/username` | 改用户名 |
| PUT | `/profile/avatar` | 改头像地址 |
| POST | `/profile/avatar/upload` | 上传头像（multipart） |
| POST | `/profile/password/verify` | 敏感操作前校验登录密码 |
| PUT | `/profile/password` | 改密码 |
| PUT | `/profile/password/by-security` | 用密保答案重置密码 |
| GET | `/profile/security` | 密保问题（不含答案） |
| POST | `/profile/security/answer/verify` | 校验密保答案 |
| PUT | `/profile/security` | 设置 / 更换三问三答（需当前答案或密码） |

## 资料（试卷）`/datum/file`

| 方法 | 路径 | 门禁 | 用途 |
|---|---|---|---|
| GET | `/datum/file/tree` # | 匿名 | 文件树 |
| GET | `/datum/file/list` | 可选会话 | 列表 |
| GET | `/datum/file/search` | 匿名 | 搜索 |
| GET | `/datum/file/subjects` | 匿名 | 科目列表 |
| GET | `/datum/file/schools` | 匿名 | 学校列表 |
| GET | `/datum/file/schools/check` | 匿名 | 校验学校是否在册（上传表单用） |
| GET | `/datum/file/{fileId}` | 可选会话 | 详情；待审件仅属主凭会话可见 |
| POST | `/datum/file` | 会话 | 上传（multipart） |
| PUT | `/datum/file` | 会话 | 属主编辑元信息（局部合并） |
| DELETE | `/datum/file/{fileId}` | 会话 | 属主删除 |
| DELETE | `/datum/file-tree/cache` | 管理 | —— |

## 电子书 `/datum/ebook`

| 方法 | 路径 | 门禁 | 用途 |
|---|---|---|---|
| GET | `/datum/ebook/list` | 匿名 | 列表（仅已上架） |
| GET | `/datum/ebook/subjects` | 匿名 | 学科列表 |
| GET | `/datum/ebook/content` ⇣ | 待核 | 在线预览流（inline，不落下载记录；PDF / EPUB） |
| GET | `/datum/ebook/favorite` | 匿名 / 会话 | 收藏状态（匿名回未收藏） |
| POST | `/datum/ebook/favorite` | 会话 | 收藏 |
| DELETE | `/datum/ebook/favorite/{ebookId}` | 会话 | 取消收藏 |
| POST | `/datum/ebook/report` | 会话 | 举报（一用户一书一条待审） |
| GET | `/datum/ebook/report/list` | 待核 | 我的电子书举报（与 `/datum/report/list` 同契约） |
| POST | `/datum/ebook/upload` | 会话 | 上传（PDF / EPUB / MOBI / AZW3，进入待审） |
| DELETE | `/datum/ebook/{ebookId}` | 会话 | 属主删除（软删） |

## 书签 `/datum/bookmark`

| 方法 | 路径 | 门禁 | 用途 |
|---|---|---|---|
| GET | `/datum/bookmark/list` | 可选会话 | 列表（匿名仅已审核） |
| GET | `/datum/bookmark/{bookmarkId}` | 可选会话 | 详情（待审仅属主可见） |
| POST | `/datum/bookmark` | 会话 | 新增 |
| PUT | `/datum/bookmark` | 会话 | 编辑（局部合并；id 兼容字符串 / 数字） |
| DELETE | `/datum/bookmark/{bookmarkId}` | 会话 | 删除 |
| POST | `/datum/bookmark/uploadCover` | 会话 | 上传封面（multipart ≤ 5 MB，JPG / PNG / GIF / WEBP） |

## 收藏

| 方法 | 路径 | 门禁 | 用途 |
|---|---|---|---|
| POST | `/datum/favorite` | 会话 | 收藏资料 |
| GET | `/datum/favorite/{fileId}` | 会话 | 资料收藏状态 |
| POST | `/datum/bookmark/favorite` | 会话 | 收藏书签 |
| GET | `/datum/bookmark/favorite/list` | 会话 | 书签收藏关系列表 |
| GET | `/datum/desktop/favorite/list/{userId}` | 会话（本人） | 我的资料收藏 |
| DELETE | `/datum/desktop/favorite/{userId}/{fileId}` | 会话（本人） | 取消资料收藏 |
| GET | `/datum/desktop/ebook/favorite/list/{userId}` | 会话（本人） | 我的电子书收藏（联查书目） |
| DELETE | `/datum/desktop/ebook/favorite/{userId}/{ebookId}` | 会话（本人） | 取消电子书收藏 |
| GET | `/datum/desktop/bookmark/favorite/list/{userId}` | 会话（本人） | 我的书签收藏 |
| DELETE | `/datum/desktop/bookmark/favorite/{userId}/{bookmarkId}` | 会话（本人） | 取消书签收藏 |

## 下载

| 方法 | 路径 | 门禁 | 用途 |
|---|---|---|---|
| GET | `/datum/download/file?fileId=` ⇣ | 会话 | 下载资料流；服务端落下载记录 |
| GET | `/datum/download/ebook?ebookId=` ⇣ | 会话 | 下载电子书流；落 `ptmj_ebook_download` |
| GET | `/datum/desktop/download/list/{userId}` | 会话（本人） | 我的下载记录 |
| DELETE | `/datum/desktop/download/{userId}/{fileId}` | 会话（本人） | 删除下载记录 |
| POST / PUT / GET / DELETE | `/datum/download`、`/datum/download/{id}`、`/datum/download/{ids}` | 会话 | 下载记录 CRUD（属主）。**客户端不手动写记录**——流式端点已自动记录；仅列出备查 |

经 edge 时每 IP 只允许 1 个下载连接（`deploy/nginx.conf.template`）：**客户端下载队列默认串行**，并发 > 1 会被拒。

## 举报

| 方法 | 路径 | 门禁 | 用途 |
|---|---|---|---|
| POST | `/datum/report` | 会话 | 举报资料 |
| GET | `/datum/report/list` | 会话 | 我的资料举报 |
| GET | `/datum/report/{reportId}` | 会话 | 举报详情 |
| PUT | `/datum/report` | 会话 | 编辑举报（仅待处理状态） |
| DELETE | `/datum/report/{ids}` | 会话 | 撤回举报（批量） |
| GET | `/datum/report/timeline/{fileId}` | 会话 | 举报处理时间线 |
| POST / GET / PUT / DELETE | `/datum/bookmarkReport`、`/list`、`/{reportId}`、`/{ids}` | 会话 | 书签举报，同上 |
| POST | `/datum/report/audit/{reportId}`、`/datum/bookmarkReport/audit/{reportId}` | 管理 | —— |

## 社区

| 方法 | 路径 | 门禁 | 用途 |
|---|---|---|---|
| GET | `/datum/user/rank` # | 匿名 | 上传排行（SQL top 50 + Redis 缓存；Redis 故障时 503，客户端显示旧缓存） |
| GET | `/datum/user/{userId}` | 匿名 | 用户公开资料 |
| GET | `/datum/user/list` | 匿名 | 平台用户列表（客户端暂无用途） |

## 通知

| 方法 | 路径 | 门禁 | 用途 |
|---|---|---|---|
| GET | `/system/notification/user/popup` | 会话 | 弹窗通知（铃铛 + 首页弹窗），载荷含正文 |
| GET | `/system/notification/user/scroll` | 会话 | 滚动提醒 |
| GET | `/datum/notification/list`、`/datum/notification/{notifyId}` | 会话 | 通知列表 / 详情（与上两条的取舍待核） |
| POST / PUT / DELETE | `/datum/notification…` | 管理 | —— |

通知类型 `notifyType`：1 版本更新、2 系统故障、3 系统维护、4 资料下架、5 公告。已读状态**只在客户端本地**按 `(userId, notifyId)` 记录。

## 不在客户端范围内（管理）

`POST/PUT/DELETE /datum/user`、`POST /datum/user/resetSecurityAnswers`、`DELETE /datum/user/rank/cache`、
`DELETE /datum/file-tree/cache`、通知 CRUD、两类举报审核，以及全部 `/api/*`。
