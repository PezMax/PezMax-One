# 官方端调用参考（含「旧端过时用法」清单）

> **这份文件怎么用**：`PezMax-Go/PezMax-Desktop` 是**旧接口时代的 Electron 官方端**，
> 这里记录它怎么调 `/datum`、以及它哪些做法**已经过时**。它只作**用法与交互参考**；
> **契约一律以 `endpoints.md` 与 `k_admin` 的 Go 源码为准**，本文与它们冲突时以后者为准。
>
> 三端实现时看两件事：§2 的调用总表（参数怎么写、字段读哪个）、§8 的差异清单（别抄哪些）。
> 下游基线变化后本文要重做，不要当契约查。

---

**基线**：`PezMax-Go@061a888`。所有 `文件:行号` 相对 `PezMax-Go` 根（`PezMax-Desktop/...`、`k_admin/...`）。

---

## 1. HTTP 封装约定

### 1.1 实例与 baseURL
- axios 实例：`baseURL = import.meta.env.VITE_APP_BASE_API`，`timeout = 10000`，默认头带 `Cache-Control/Pragma: no-cache`；全局 `Content-Type: application/json;charset=utf-8` —— `PezMax-Desktop/src/renderer/utils/request.js:75-86`。
- 生产构建（无 `VITE_APP_ENV`）的 `VITE_APP_BASE_API` 是**绝对地址** `http://127.0.0.1:9033` —— `PezMax-Desktop/.env:1`。
- 开发用 `/dev-api` 前缀，由 vite dev server 反代到 `VITE_APP_TARGET_URL` 并**剥掉前缀** —— `PezMax-Desktop/.env.development:8,11`、`PezMax-Desktop/electron.vite.config.mjs:80-85`。
- 因此旧端存在两个「基址」：axios 走 `VITE_APP_BASE_API`；主进程上传与树节点相对 URL 走 `VITE_APP_TARGET_URL` —— `PezMax-Desktop/src/renderer/store/modules/upload.js:88`、`PezMax-Desktop/src/renderer/views/home/index.vue:1051`。
- 用户域前缀可被覆盖：`VITE_DATUM_USER_API_BASE || '/datum/user'` —— `PezMax-Desktop/src/renderer/api/datum/userApi.js:1-6`。

### 1.2 认证（无 Cookie，只有 Bearer）
- 全仓未出现 `withCredentials`/`xsrf`，**不携带 Cookie** —— `PezMax-Desktop/src/renderer/utils/request.js:77-86`（相关关键字在 `PezMax-Desktop/src` 下未找到）。
- token 存 localStorage 键 `Admin-Token` —— `PezMax-Desktop/src/renderer/utils/auth.js:1,11-20`。
- 每个请求注入 `Authorization: Bearer <token>`；`headers.isToken === false` 时不注入 —— `PezMax-Desktop/src/renderer/utils/request.js:117,122,126-128`。
- token 解析顺序：localStorage → Pinia `user.token` —— `PezMax-Desktop/src/renderer/utils/request.js:88-97`。
- 匿名端点（登录/注册/验证码/密保问题/密保重置）显式关掉 token —— `PezMax-Desktop/src/renderer/api/login.js:14-17,28,55-58,79-83,91-95`。
- GET 参数由客户端**手工拼进 URL**再清空 `params` —— `PezMax-Desktop/src/renderer/utils/request.js:130-135`；序列化会丢掉 `null`/`''`/`undefined`，对象展开成 `k[sub]=v` —— `PezMax-Desktop/src/renderer/utils/ruoyi.js:200-224`。

### 1.3 信封怎么解（关键）
- 响应拦截器只做两件事：`responseType` 为 `blob/arraybuffer` 时直接返回；其余 `Promise.resolve(normalizeKAdminPage(res.data))` —— `PezMax-Desktop/src/renderer/utils/request.js:171-178`。
- **不解包 `data`、不校验 `code`**：调用方自己读 `res.code` / `res.data` / `res.rows` / `res.total` —— `PezMax-Desktop/src/renderer/utils/request.js:176-177`。
- 现行信封只有两种（`code` 恒等于 HTTP 状态码）：
  - 成功体 A：HTTP 200 + `{code:0, message:"ok", msg:"ok", data:<任意>}` —— `k_admin/internal/kadmin/transport/httpx/response.go:9-16`；
  - 失败体：HTTP `<status>` + `{code:<status>, message, msg, data:null}` —— `k_admin/internal/kadmin/transport/httpx/response.go:18-25`。
- 成功体 B（RuoYi `TableDataInfo`）：HTTP 200 + 顶层 `{code:0, msg, message, rows[], total}` —— `k_admin/internal/kadmin/datum_file.go:347-354`、`k_admin/internal/kadmin/datum_activity.go:484-493`、`k_admin/internal/kadmin/datum_user_admin.go:59-66`。
- `normalizeKAdminPage` 只把 `data.items[]` + `data.total(number)` 改写成 `data.{rows,total}` —— `PezMax-Desktop/src/renderer/utils/request.js:67-73`；现调用的 `/datum` 端点**没有** `data.items` 形状（`k_admin/internal/kadmin/datum_*.go` 内列表均为顶层 `rows`），该分支是死代码。
- 旧端到处写 `res.code === 200 || res.code === 0` 双兼容 —— `PezMax-Desktop/src/renderer/views/home/index.vue:738`、`PezMax-Desktop/src/renderer/components/NotificationCenter/index.vue:86`、`PezMax-Desktop/src/renderer/views/home/components/UploadPanel.vue:278`、`PezMax-Desktop/src/renderer/views/rank/index.vue:256`。

### 1.4 非 2xx、业务错误、401、超时、重试
- 业务失败靠**真实 HTTP 状态码**承载，客户端在 error 分支取 `responseData.msg || responseData.message` —— `PezMax-Desktop/src/renderer/utils/request.js:181-188`。
- 只有命中文案白名单 `specificPatterns` 才展示服务端原文，否则显示「后端接口连接异常 / 系统接口请求超时 / 系统接口4xx异常」—— `PezMax-Desktop/src/renderer/utils/request.js:16,189-202`。
- `skipErrorMsg: true` 的请求静默失败（reject 的 Error 带 `.silent`）—— `PezMax-Desktop/src/renderer/utils/request.js:189,207-210`；用于贡献者信息等辅助查询 —— `PezMax-Desktop/src/renderer/api/datum/user.js:13-20`。
- `headers.noMessage` 是**死参数**：拦截器从不读它 —— `PezMax-Desktop/src/renderer/api/datum/user.js:54` 对 `PezMax-Desktop/src/renderer/utils/request.js:115-168`。
- 401 → `handleSessionExpired(serverMsg)` —— `PezMax-Desktop/src/renderer/utils/request.js:186-188`；处理策略：
  - 文案匹配 `/账号已被停用|账号已被封禁/` → 直接清本地会话并跳登录，不弹窗 —— `PezMax-Desktop/src/renderer/utils/request.js:33-43`；
  - 其余 → 弹「登录状态已过期」（`留在此页` / `重新登录`），确认才清会话并带 `redirect` 跳登录，取消则留在原页并 reject —— `PezMax-Desktop/src/renderer/utils/request.js:44-63`、`PezMax-Desktop/src/renderer/components/SessionExpiredModal/index.js:7-38`、`PezMax-Desktop/src/renderer/components/SessionExpiredModal/index.vue:13-24`。
- 主进程的上传/下载不走 axios，401 时发 IPC `session-expired`，渲染端用同一个 `handleSessionExpired` 收口 —— `PezMax-Desktop/src/main/index.js:625-644,1043-1045`、`PezMax-Desktop/src/preload/index.js:51-55`、`PezMax-Desktop/src/renderer/App.vue:31-33`。
- 超时：默认 10s（`PezMax-Desktop/src/renderer/utils/request.js:81`）；注册 5s（`PezMax-Desktop/src/renderer/api/login.js:32`）；验证码 20s（`PezMax-Desktop/src/renderer/api/login.js:60,71`）；文本内容 30s（`PezMax-Desktop/src/renderer/views/home/components/MainEditor.vue:1462`）。
- 重试/退避：**未找到**（`PezMax-Desktop/src` 全仓无 HTTP 重试实现）。
- 防重复提交：仅 POST/PUT，1 秒窗口内同 URL 同 body 直接 reject —— `PezMax-Desktop/src/renderer/utils/request.js:136-163`；`repeatSubmit:false` 关闭 —— `PezMax-Desktop/src/renderer/api/login.js:16`、`PezMax-Desktop/src/renderer/api/datum/desktopUser.js:25`。
- FormData 请求会删掉 `Content-Type`，交给运行时生成 boundary —— `PezMax-Desktop/src/renderer/utils/request.js:99-112,123-125`。

---

## 2. 端点调用总表

> 「关键参数」只列客户端**实际发出**的字段名。⚠ = 文档里最容易漏的（排序/hash/unchanged/校验/搜索字段名）。

### 2.1 认证与用户（`/datum/user`）
| 功能 | 方法 + 路径 | 关键参数 | 客户端读取 | 出处 |
|---|---|---|---|---|
| 登录 | POST `/datum/user/login` | body: `username`,`password`,`code`,`uuid` | `data.token`（失败读 `msg`） | `PezMax-Desktop/src/renderer/api/login.js:5-21`、`PezMax-Desktop/src/renderer/store/modules/user.js:30-37` |
| 验证码 | GET `/datum/user/captchaImage` | 无 | `data.img`(裸 base64 JPEG)、`data.uuid`、`data.captchaEnabled` | `PezMax-Desktop/src/renderer/api/login.js:53-62`、`PezMax-Desktop/src/renderer/views/login.vue:179-183` |
| 当前用户 | GET `/datum/user/getInfo` | 无 | `data.user{userId,userName,nickName,avatar,count}`、`data.roles`、`data.permissions` | `PezMax-Desktop/src/renderer/store/modules/user.js:61-83` |
| 退出 | POST `/datum/user/logout` | 无 | 忽略；失败也清本地 | `PezMax-Desktop/src/renderer/store/modules/user.js:103-116` |
| 找回密码① | GET `/datum/user/securityQuestions` | ⚠ `userName`（大写 N）、`code`、`uuid` | `data[]`，每项 `.question` | `PezMax-Desktop/src/renderer/views/forgetPassword.vue:334-353` |
| 找回密码② | POST `/datum/user/resetPasswordBySecurity` | `username`（小写）、`code`、`uuid`、`securityAnswerOne..Three`、`newPassword`、`confirmPassword` | 仅成败 | `PezMax-Desktop/src/renderer/views/forgetPassword.vue:416-425` |
| 用户详情 | GET `/datum/user/{userId}` | 路径 | `data.user` 或 `data` | `PezMax-Desktop/src/renderer/api/datum/user.js:14-20`、`PezMax-Desktop/src/renderer/views/home/components/FileInfoDrawer.vue:185-190` |
| 上传榜 | GET `/datum/user/rank` | ⚠ `hash`（可选） | `code`、`unchanged`、`data[]`、`hash` | `PezMax-Desktop/src/renderer/api/datum/user.js:48-56`、`PezMax-Desktop/src/renderer/views/rank/index.vue:254-265` |
| 用户列表 | GET `/datum/user/list` | `pageNum`,`pageSize`,`userName`,`status` | `rows`/`total` | `PezMax-Desktop/src/renderer/api/datum/user.js:5-11`（桌面端未调用：`PezMax-Desktop/src` 内无调用点） |

### 2.2 试卷文件（`/datum/file`、`/datum/download/file`）
| 功能 | 方法 + 路径 | 关键参数 | 客户端读取 | 出处 |
|---|---|---|---|---|
| 列表 | GET `/datum/file/list` | `pageNum`,`pageSize`,`fileName`,`fileUrl`,`fileSize`,`fileFormat`,`fileYear`,`fileType`,`fileSubject`,`reviewer`,`fileStatus`；⚠ `userId`（我的上传/计数） | `rows[]`、`total`；行内 `fileId`,`fileName`,`fileUrl`,`fileSize`,`fileFormat`,`fileSchool`,`fileSubject`,`fileYear`,`fileType`,`fileStatus`,`userId` | `PezMax-Desktop/src/renderer/views/datum/file/index.vue:218-230,249-251`、`PezMax-Desktop/src/renderer/views/datum/upload/index.vue:280-282` |
| 详情 | GET `/datum/file/{fileId}` | 路径 | `data`（同列表行字段）+ `userId`/`createByUserId` + 可选 `user`/`ptmjUser` | `PezMax-Desktop/src/renderer/api/datum/file.js:13-18`、`PezMax-Desktop/src/renderer/views/home/components/FileInfoDrawer.vue:172-196` |
| 上传 | POST `/datum/file`（multipart） | form: `file` + `fileName`,`fileSchool`,`fileSubject`,`fileYear`,`fileType`,`fileSize` | 旧端只读 `code`/`msg`（Go 另回 `fileId`,`fileUrl`,`fileName`） | `PezMax-Desktop/src/renderer/store/modules/upload.js:118-146`、`PezMax-Desktop/src/main/index.js:994-1010` |
| 修改 | PUT `/datum/file` | body: 表单字段 + `userId` | 仅成败 | `PezMax-Desktop/src/renderer/views/datum/upload/index.vue:339-350` |
| 删除 | DELETE `/datum/file/{fileId}` | 路径 | 仅成败 | `PezMax-Desktop/src/renderer/api/datum/file.js:39-44` |
| 文件树 | GET `/datum/file/tree` | ⚠ `hash` | `code`、`unchanged`、`data[]`、`hash`；节点 `id`/`label`/`type`/`children`/`fileId`/`fileName`/`fileUrl`/`previewUrl`/`fileFormat`/`fileStatus` | `PezMax-Desktop/src/renderer/api/datum/file.js:47-53`、`PezMax-Desktop/src/renderer/views/home/index.vue:737-747,1012-1068` |
| 科目联想 | GET `/datum/file/subjects` | ⚠ `limit=10`、`keyword` | `data[]`，元素为字符串 **或** `{Subject,Total}` | `PezMax-Desktop/src/renderer/views/home/components/UploadPanel.vue:262-268,372,384-389` |
| 学校联想 | GET `/datum/file/schools` | ⚠ `limit=10`、`keyword` | `data[]`（字符串） | `PezMax-Desktop/src/renderer/views/home/components/UploadPanel.vue:277,289-295` |
| 学校校验 | GET `/datum/file/schools/check` | ⚠ `schoolName` | `data === true` 或 `data.exists === true` | `PezMax-Desktop/src/renderer/views/home/components/UploadPanel.vue:325-326` |
| 关键字搜索 | GET `/datum/file/search` | `keyword` | —— 未调用 | `PezMax-Desktop/src/renderer/api/datum/file.js:83-91`（无调用点） |
| 文本内容 | GET `/datum/file/content` | `fileId` | 二进制（`responseType:'blob'` 后 `.text()`） | `PezMax-Desktop/src/renderer/views/home/components/MainEditor.vue:1454-1464` |
| 下载流 | GET `/datum/download/file` | `fileId` | 字节流 + `ETag`/`Content-Range`/`Content-Length` | `PezMax-Desktop/src/renderer/views/home/index.vue:1283-1287`、`PezMax-Desktop/src/main/index.js:602-675` |

### 2.3 电子书（`/datum/ebook`、`/datum/desktop/ebook`、`/datum/download/ebook`）
| 功能 | 方法 + 路径 | 关键参数 | 客户端读取 | 出处 |
|---|---|---|---|---|
| 列表 | GET `/datum/ebook/list` | `pageNum`,`pageSize`（⚠ 客户端不传 `keyword`，后端支持） | `rows[]`/`total`；`ebookId`,`ebookName`,`author`,`publisher`,`coverUrl`/`cover`,`ebookFormat`,`ebookSize`,`ebookUrl`,`ebookSubject`,`ebookType`,`ebookStatus`,`userId`,`createTime`,`remark` | `PezMax-Desktop/src/renderer/views/home/components/SidePanel.vue:960-961,975-990,1006-1020` |
| 预览流 | GET `/datum/ebook/content` | `ebookId`；⚠ 直连时 `token` 走 query | 二进制（iframe/epub.js） | `PezMax-Desktop/src/renderer/api/datum/ebook.js:31-35` |
| 下载流 | GET `/datum/download/ebook` | `ebookId` | 字节流 | `PezMax-Desktop/src/renderer/api/datum/ebook.js:37-41`、`PezMax-Desktop/src/renderer/views/home/index.vue:1244-1248` |
| 学科聚合 | GET `/datum/ebook/subjects` | 无 | 未调用 | `PezMax-Desktop/src/renderer/api/datum/ebook.js:13-19`（无调用点） |
| 收藏 id 集合 | GET `/datum/ebook/favorite/list` | 无 | `data[]`（ebookId 数组） | `PezMax-Desktop/src/renderer/api/datum/ebook.js:43-49`、`PezMax-Desktop/src/renderer/views/home/index.vue:496,503-504` |
| 单本收藏状态 | GET `/datum/ebook/favorite` | `ebookId` | 未调用 | `PezMax-Desktop/src/renderer/api/datum/ebook.js:51-58`（无调用点） |
| 收藏/取消 | POST `/datum/ebook/favorite`；DELETE `/datum/ebook/favorite/{ebookId}` | body `{ebookId,userId}` | 仅成败 | `PezMax-Desktop/src/renderer/views/home/index.vue:577,583` |
| 举报 | POST `/datum/ebook/report` | `ebookId`,`userId`,`reason`,`remark`（空则省略） | `msg` | `PezMax-Desktop/src/renderer/views/home/components/ReportFileDialog.vue:145-151` |
| 我的电子书举报 | GET `/datum/ebook/report/list` | `pageNum`,`pageSize`,`userId` | `rows[]` 或 `data[]` | `PezMax-Desktop/src/renderer/views/home/components/ReportTimelinePanel.vue:145,151-152` |
| 我的电子书收藏 | GET `/datum/desktop/ebook/favorite/list/{userId}` | 路径 `userId` + `pageNum`,`pageSize` | `rows[]`（同 book 行字段） | `PezMax-Desktop/src/renderer/api/datum/ebook.js:95-105`、`PezMax-Desktop/src/renderer/views/datum/favorite/index.vue:200-225,252-254` |
| 取消收藏 | DELETE `/datum/desktop/ebook/favorite/{userId}/{ebookId}` | 路径 | 仅成败 | `PezMax-Desktop/src/renderer/views/datum/favorite/index.vue:297` |
| 上传 | POST `/datum/ebook/upload`（multipart） | `file` + `ebookName`,`author`,`publisher`,`ebookSubject`,`ebookType`,`fileSize` | `code` | `PezMax-Desktop/src/renderer/store/modules/upload.js:92-113` |
| 删除自己的书 | DELETE `/datum/ebook/{ebookId}` | 路径 | 未调用 | `PezMax-Desktop/src/renderer/api/datum/ebook.js:115-121`（无调用点） |

### 2.4 收藏 / 下载记录 / 书签
| 功能 | 方法 + 路径 | 关键参数 | 客户端读取 | 出处 |
|---|---|---|---|---|
| 试卷收藏列表 | GET `/datum/desktop/favorite/list/{userId}` | 路径 `userId` + `pageNum`,`pageSize` | `rows[]`/`total` | `PezMax-Desktop/src/renderer/api/datum/favorite.js:4-14` |
| 收藏试卷 | POST `/datum/favorite` | `{fileId,userId}` | 仅成败（重复收藏为 409） | `PezMax-Desktop/src/renderer/views/home/index.vue:671` |
| 取消收藏 | DELETE `/datum/desktop/favorite/{userId}/{fileId}` | 路径 | 仅成败 | `PezMax-Desktop/src/renderer/views/home/index.vue:667` |
| 收藏状态 | GET `/datum/favorite/{fileId}` | 路径 | 旧端当「收藏详情」读 —— 见 §8 | `PezMax-Desktop/src/renderer/api/datum/favorite.js:16-22` |
| 服务端下载列表 | GET `/datum/desktop/download/list/{userId}` | ⚠ 只发 `pageNum=1,pageSize=1` 取 `total` | `total` | `PezMax-Desktop/src/renderer/views/datum/ptmjUser/index.vue:499-514` |
| 下载记录写/改/删 | POST `/datum/download`；PUT `/datum/download`；DELETE `/datum/download/{downloadId}` | `{fileId,userId}` / `{downloadId,remark}` | 未调用 | `PezMax-Desktop/src/renderer/api/datum/download.js:25-48`（无调用点） |
| 书签列表 | GET `/datum/bookmark/list` | ⚠ `userId`,`url`,`title`,`resourceType`,`collection` + `pageNum`,`pageSize` | `rows[]`/`total`；`id`,`title`,`url`,`cover_image`/`coverImage`,`description`,`collection`,`subject`,`resourceType`,`createTime`,`status`/`statusInfo` | `PezMax-Desktop/src/renderer/api/datum/bookmark.js:4-10`、`PezMax-Desktop/src/renderer/views/home/components/SidePanel.vue:1047-1054,1190-1211` |
| 新建书签 | POST `/datum/bookmark` | `url`,`title`,`description`,`coverImage`,`resourceType`,`collection`,`subject`,`remark` | ⚠ `bookmarkId`/`id`（旧端还兜底 `data.bookmarkId`、`data.id`、`data` 为数字） | `PezMax-Desktop/src/renderer/views/home/components/SidePanel.vue:1075-1098` |
| 改书签 | PUT `/datum/bookmark` | `{id, coverImage}` | 仅成败 | `PezMax-Desktop/src/renderer/views/home/components/SidePanel.vue:1154-1157` |
| 删书签 | DELETE `/datum/bookmark/{id}` | 路径 | 仅成败 | `PezMax-Desktop/src/renderer/api/datum/bookmark.js:31-36` |
| 上传封面 | POST `/datum/bookmark/uploadCover`（multipart） | `file` + `resourceType`,`resource_type`,`collection`,`bookmarkName`,`bookmarkId`,`id` | `data` 为 string 或 `{url|fileUrl|fileName}` | `PezMax-Desktop/src/renderer/views/home/components/SidePanel.vue:1117-1149` |
| 我的书签收藏关系 | GET `/datum/bookmark/favorite/list` | `pageNum`,`pageSize`（⚠ 旧端不传 `userId`） | `rows[]`（`bookmarkId`,`userId`） | `PezMax-Desktop/src/renderer/views/home/index.vue:495,512-514` |
| 桌面端书签收藏 | GET `/datum/desktop/bookmark/favorite/list/{userId}` | 路径 + `pageNum`,`pageSize` | `rows[]`/`total` | `PezMax-Desktop/src/renderer/api/datum/bookmarkFavorite.js:13-23` |
| 收藏书签 | POST `/datum/bookmark/favorite` | `{bookmarkId,userId}` | 仅成败 | `PezMax-Desktop/src/renderer/views/home/index.vue:629` |
| 取消收藏 | DELETE `/datum/desktop/bookmark/favorite/{userId}/{bookmarkId}` | 路径 | 仅成败 | `PezMax-Desktop/src/renderer/views/datum/favorite/index.vue:295` |

### 2.5 举报
| 功能 | 方法 + 路径 | 关键参数 | 客户端读取 | 出处 |
|---|---|---|---|---|
| 我的试卷举报 | GET `/datum/report/list` | `pageNum`,`pageSize`,`userId` | `rows[]`/`data[]`；`reportId`/`id`,`reason`,`remark`,`createTime`,`updateTime`,`fileName`/`filename`/`name` | `PezMax-Desktop/src/renderer/views/home/components/ReportTimelinePanel.vue:144,148-158` |
| 提交举报 | POST `/datum/report` | `fileId`,`userId`,`reason`,`remark` | `msg` | `PezMax-Desktop/src/renderer/views/home/components/ReportFileDialog.vue:153-159` |
| 时间线 | GET `/datum/report/timeline/{reportId}` | 路径 | `code`、`data` | `PezMax-Desktop/src/renderer/views/home/components/ReportTimelinePanel.vue:185-192` |
| 后台筛选 | GET `/datum/report/list` | `fileId`,`userId`,`result` + 分页 | `rows`/`total` | `PezMax-Desktop/src/renderer/views/datum/report/index.vue:149-155,175` |
| 书签举报 | POST `/datum/bookmarkReport` | `bookmarkId`,`userId`,`reason`,`remark` | `msg` | `PezMax-Desktop/src/renderer/views/home/components/ReportBookmarkDialog.vue:133-140` |
| 书签举报其余 | GET list / GET `{id}` / PUT / DELETE / timeline | —— | 未调用 | `PezMax-Desktop/src/renderer/api/datum/bookmarkReport.js:3-46`（除 add 外无调用点） |

### 2.6 通知
| 功能 | 方法 + 路径 | 关键参数 | 客户端读取 | 出处 |
|---|---|---|---|---|
| 弹窗喂给 | GET `/system/notification/user/popup` | ⚠ `userId`、`hash` | `code`、`data[]`、`hash`、`unchanged`；`notifyId`,`notifyType`,`title`,`content`,`createTime`,`forceUpdate`,`updateDownloadUrl`,`maintenanceStartTime` | `PezMax-Desktop/src/renderer/api/datum/notification.js:51-57`、`PezMax-Desktop/src/renderer/views/home/index.vue:276-288` |
| 滚动喂给 | GET `/system/notification/user/scroll` | ⚠ 仅 `hash`（**无 userId**，全局广播） | 同上 | `PezMax-Desktop/src/renderer/api/datum/notification.js:63-69`、`PezMax-Desktop/src/renderer/components/TitleHeader/index.vue:188-194,208` |
| 后台列表 | GET `/datum/notification/list` | `pageNum`,`pageSize`,`notifyType`,`title`,`content`,`status`,⚠`sort`,⚠`displayMode`,⚠`appVersionMin`,⚠`latestVersion`,⚠`forceUpdate`,⚠`updateDownloadUrl`,⚠`faultStartTime/EndTime`,⚠`maintenanceStartTime/EndTime`,⚠`remindBeforeMinutes`,⚠`uploadUserId`,⚠`materialId`,⚠`materialTitleSnapshot`,⚠`publishStart/publishEnd`,⚠`scrollTimeInterval` | `rows`/`total` | `PezMax-Desktop/src/renderer/views/datum/notification/index.vue:399-423` |
| 增改删详情 | POST/PUT `/datum/notification`、DELETE `/datum/notification/{notifyId}`、GET `/datum/notification/{notifyId}` | 同上字段 | `data` | `PezMax-Desktop/src/renderer/api/datum/notification.js:12-44` |

### 2.7 桌面个人中心（`/datum/desktop/user`）
| 功能 | 方法 + 路径 | 关键参数 | 客户端读取 | 出处 |
|---|---|---|---|---|
| 改用户名 | PUT `/datum/desktop/user/profile/username` | `{userName}` | 仅成败 | `PezMax-Desktop/src/renderer/api/datum/desktopUser.js:3-9` |
| 改头像地址 | PUT `/datum/desktop/user/profile/avatar` | `{avatar}` | 仅成败 | `PezMax-Desktop/src/renderer/api/datum/desktopUser.js:11-17` |
| 上传头像 | POST `/datum/desktop/user/profile/avatar/upload`（multipart `file`） | — | 仅成败 | `PezMax-Desktop/src/renderer/api/datum/desktopUser.js:19-27`、`PezMax-Desktop/src/renderer/views/datum/ptmjUser/index.vue:555-566` |
| 校验密码 | POST `/datum/desktop/user/profile/password/verify` | `{password}` | 仅成败 | `PezMax-Desktop/src/renderer/api/datum/desktopUser.js:29-35` |
| 校验密保答案 | POST `/datum/desktop/user/profile/security/answer/verify` | `{answer}` | 仅成败 | `PezMax-Desktop/src/renderer/api/datum/desktopUser.js:37-43` |
| 改密码 | PUT `/datum/desktop/user/profile/password` | `{oldPassword,newPassword}` | 仅成败 | `PezMax-Desktop/src/renderer/api/datum/desktopUser.js:45-51` |
| 密保重置密码 | PUT `/datum/desktop/user/profile/password/by-security` | `data` | 仅成败 | `PezMax-Desktop/src/renderer/api/datum/desktopUser.js:53-60` |
| 读密保 | GET `/datum/desktop/user/profile/security` | 无 | `data.questions[]` 或 `data.question` 按 `'|'` 切 | `PezMax-Desktop/src/renderer/api/datum/desktopUser.js:62-67`、`PezMax-Desktop/src/renderer/views/datum/ptmjUser/index.vue:525-538` |
| 写密保 | PUT `/datum/desktop/user/profile/security` | `password`,`securityQuestionOne..Three`,`securityAnswerOne..Three` | 仅成败 | `PezMax-Desktop/src/renderer/views/datum/ptmjUser/index.vue:596-609` |

### 2.8 知识库 / 内容流
| 功能 | 方法 + 路径 | 关键参数 | 客户端读取 | 出处 |
|---|---|---|---|---|
| 知识内容 | GET `/datum/knowledge/content` | ⚠ `kind=file`、`id`、`revision` | 文本（blob） | `PezMax-Desktop/src/renderer/views/home/components/MainEditor.vue:1454-1459` |
| 知识内容（主进程/共享） | GET `${baseUrl}/datum/knowledge/content` | `kind`,`id`,`revision` | 流 | `PezMax-Desktop/src/shared/knowledge-assistant.mjs:80` |
| 找资料助手 | —— 走 IPC `window.electronAPI.assistant.*`，非 HTTP | — | `result.ok/data/error` | `PezMax-Desktop/src/renderer/api/assistant.js:7-14` |

---

## 3. 分页与全量

- 后端分页参数：`pageNum`（别名 `page`）、`pageSize`（别名 `size`），默认 `1/20`，**单页硬上限 100** —— `k_admin/internal/kadmin/datum_file.go:268-293`。
- `normalizeKAdminPage`：只有当 `data.items` 是数组且 `data.total` 是 `number` 时把 `data` 换成 `{rows,total}`，其余原样 —— `PezMax-Desktop/src/renderer/utils/request.js:67-73`。现行 `/datum` 列表都是顶层 `rows/total`，故它只对历史响应有意义。
- `fetchAllPages(fetcher, baseQuery, {pageSize=100, maxPages=100})` 算法 —— `PezMax-Desktop/src/renderer/utils/pagination.js:8-19`：
  1. 先发 `{...baseQuery, pageNum:1, pageSize:100}`；
  2. `total = Number(first.total) || rows.length`；
  3. `for pageNum=2..maxPages && rows.length < total` 串行 `await` 逐页拉；某页 `rows` 为空即 `break`；
  4. 返回合并后的 `{rows,total}`。
  **串行、无并发**；终止条件是「累计行数 ≥ total」或「空页」或「打到 100 页上限」。它只读**顶层** `rows/total`（`pagination.js:10-14`）。
- 用全量的页面：书签、电子书（`PezMax-Desktop/src/renderer/views/home/components/SidePanel.vue:924,960`）、首页收藏/书签收藏（`PezMax-Desktop/src/renderer/views/home/index.vue:493-497,512`）、我的收藏（`PezMax-Desktop/src/renderer/views/datum/favorite/index.vue:254`）、我的上传（`PezMax-Desktop/src/renderer/views/datum/upload/index.vue:280,293`）。
- 用服务端分页的页面：后台文件列表、举报、通知（`PezMax-Desktop/src/renderer/views/datum/file/index.vue:249-251`、`PezMax-Desktop/src/renderer/views/datum/report/index.vue:175`、`PezMax-Desktop/src/renderer/views/datum/notification/index.vue:451`）。
- 只要计数时用 `pageSize=1` 取 `total` —— `PezMax-Desktop/src/renderer/views/datum/ptmjUser/index.vue:501-506`。

## 4. URL 归一

`normalizeFileUrl`（`PezMax-Desktop/src/renderer/utils/url.js:15-48`）的全部规则：

1. 空/非字符串 → `''` —— `url.js:16`。
2. `targetUrl = VITE_APP_TARGET_URL || 'http://127.0.0.1:9033'` —— `url.js:18`。
3. **相对路径**（不以 `http`/`data:`/`blob:` 开头）：先剥掉开头的 `/dev-api`、`/prod-api`、`/stage-api`，再拼上 `targetUrl` —— `url.js:22-26`。
4. **绝对地址且含内部 host 白名单** `localhost`、`127.0.0.1`、`minio`、`server`、`172.`：`new URL()` 后**只替换 hostname**（端口、path、query 原样保留），且仅当 hostname 与 target 不同才替换 —— `url.js:29-42`。
5. 其余原样返回 —— `url.js:48`。

它**不做**的：不改协议（无 http→https 逻辑，全文 `url.js:15-48`）；`minio://x/y` 不以 `http` 开头，会命中规则 3 被拼成 `${targetUrl}/minio://x/y`；`172.` 只做字符串包含匹配（`url.js:29-30`）。

调用点：头像归一 `PezMax-Desktop/src/renderer/utils/avatar.js:3-11`；图片上传/预览 `PezMax-Desktop/src/renderer/components/ImageUpload/index.vue:119`、`PezMax-Desktop/src/renderer/components/ImagePreview/index.vue:40,48`；编辑器 `PezMax-Desktop/src/renderer/views/home/components/MainEditor.vue:110,169,176`；侧栏 `PezMax-Desktop/src/renderer/views/home/components/SidePanel.vue:170,226,356`；打开文件 `PezMax-Desktop/src/renderer/views/home/index.vue:1184`。
另有一处**不等价**的重复实现：文件树节点相对 URL 直接用 `VITE_APP_TARGET_URL` 拼，不剥前缀、不换 host —— `PezMax-Desktop/src/renderer/views/home/index.vue:1050-1053`。

## 5. 下载

渲染端发起：`downloadFileDirectly({url, fileName, token, folderPath?, silent?})` —— `PezMax-Desktop/src/preload/index.js:45`，实现 `PezMax-Desktop/src/main/index.js:551-783`。

- 入参校验：`downloadURL` 只接受 `http`/`https` 且拒绝带 userinfo；`safeDownloadName` 取两级 basename、替换 `<>:"/\|?*` 与控制字符、去尾随 `.`/空格、拒绝 DOS 设备名 —— `PezMax-Desktop/src/main/main-utils/download-path.mjs:4-12,35-41`。
- 目标路径：`folderPath || settings.downloadPath || app.getPath('downloads')`；非静默弹保存框；静默时重名自动加 ` (n)` —— `PezMax-Desktop/src/main/index.js:558-579`、`PezMax-Desktop/src/main/main-utils/download-path.mjs:20-28`。
- 同一目标正在下载 → reject「该文件正在下载中」（全局 `activeDownloads` Set，只按路径去重）—— `PezMax-Desktop/src/main/index.js:17,581-586,780-782`。
- `.part` 路径：`<final>.<sha256(url) 前16位>.part`，元数据 `<part>.json` 存 `{etag}` —— `PezMax-Desktop/src/main/main-utils/download-path.mjs:14-18`、`PezMax-Desktop/src/main/index.js:598,677-680`。
- 请求头：`Accept-Encoding: identity`；有 token 时 `Authorization: Bearer`；续传时 `Range: bytes=<n>-` + `If-Range: <上次的强 ETag>` —— `PezMax-Desktop/src/main/index.js:604-611`。
- 只接受 `200`/`206`；其它状态 reject「下载失败，服务器返回: <status>」—— `PezMax-Desktop/src/main/index.js:624,646`。
- `206` 校验：`Content-Range: bytes s-e/total`，要求 `e === total-1`、`e >= resumeFrom`、响应强 ETag 与本地记录**完全相等**，否则丢弃半成品从 0 重下一次 —— `PezMax-Desktop/src/main/index.js:654-672`。
- `416` → 删 `.part`/`.json` 后从头重下 —— `PezMax-Desktop/src/main/index.js:616-623`。
- ETag 只认强标签（`"..."`，`W/` 弱标签被丢弃）—— `PezMax-Desktop/src/main/main-utils/download-path.mjs:30-33`；响应无强 ETag 就删 meta，下次无法续传 —— `PezMax-Desktop/src/main/index.js:677-684`；启动时「有 `.part` 无 meta」→ 删 `.part` 从 0 开始 —— `PezMax-Desktop/src/main/index.js:757-775`。
- 完整性：`base + received !== totalSize` → reject「下载内容不完整」—— `PezMax-Desktop/src/main/index.js:705-710`。
- 进度事件：`download-progress`，载荷 `{fileName, progress}`（`progress` 是 2 位小数的百分比字符串），仅在 `totalSize > 0` 时发 —— `PezMax-Desktop/src/main/index.js:699-702`；preload 暴露 `onDownloadProgress` —— `PezMax-Desktop/src/preload/index.js:46-50`；渲染端按 `fileName` 过滤，只刷当前下载项 —— `PezMax-Desktop/src/renderer/views/home/index.vue:1331-1337`。
- 完成：写流 `end` 后原子改名 `.part` → 最终文件；非静默且目标已存在时先备份 `<final>.<uuid>.bak` 再换入，成功后删 meta；返回 `{success:true, filePath}`；用户取消保存框返回 `{success:false, reason:'canceled'}` —— `PezMax-Desktop/src/main/index.js:564-576,711-736`。
- 401：读响应体前 8KB 解析 `msg`/`message`，发 IPC `session-expired` 并 reject —— `PezMax-Desktop/src/main/index.js:625-644`。
- 取消：**未找到**（无取消下载 IPC/AbortController；只有上传有 `cancel-upload` —— `PezMax-Desktop/src/main/index.js:965-977`）。
- 并发：**无队列、无并发上限**；一次 IPC 一次下载。渲染端「批量下载」是 `for + await` 串行 —— `PezMax-Desktop/src/renderer/views/home/index.vue:370-427`。
- 完成后落库（渲染端写本地 SQLite）：`fileId, fileName, fileUrl, fileSize, fileFormat, fileSchool, fileSubject, fileYear, fileType, localPath, userId` —— `PezMax-Desktop/src/renderer/views/home/index.vue:1296-1314`；电子书记录用**负数 `fileId`** 标记 —— `PezMax-Desktop/src/renderer/views/datum/favorite/index.vue:388-400`、`PezMax-Desktop/src/renderer/views/datum/download/index.vue:124-125`。
- 本地表 `download_records`（含 `local_path`,`user_id`,`download_time` 默认本地时间）—— `PezMax-Desktop/src/main/main-utils/database.js:27-43`。
- **「我的下载」不与服务端列表合并**：页面只读本地 SQLite —— `PezMax-Desktop/src/renderer/views/datum/download/index.vue:159`；去重靠 SQL `id IN (SELECT MAX(id) … GROUP BY file_id)` —— `PezMax-Desktop/src/main/main-utils/database.js:125-146`；删除按 `(user_id, file_id)` 全删 —— `database.js:149-158`。服务端 `/datum/desktop/download/list/{userId}` 只用来取计数（§2.4）。打开文件时依次尝试 `localPath` → 下载目录 → 提示「本地文件已删除」并按 §5 重新下载 —— `PezMax-Desktop/src/renderer/views/datum/download/index.vue:208-236`。

## 6. 通知

- 取数端点与参数：`popup` = `userId` + 可选 `hash`；`scroll` = 仅 `hash`（全局，无 userId）—— `PezMax-Desktop/src/renderer/api/datum/notification.js:51-69`。
- `hash`/`unchanged` 协议：客户端带上上次的 `hash`；服务端对本次结果算内容哈希，未变则回 `unchanged:true` 且 `data:null`（并回显 `hash`）—— `k_admin/internal/kadmin/datum_notification.go:281-309`。客户端在 `unchanged` 时**沿用上次数据**，不用空 `data` 覆盖 —— `PezMax-Desktop/src/renderer/components/NotificationCenter/index.vue:86-100`、`PezMax-Desktop/src/renderer/components/TitleHeader/index.vue:190-193`、`PezMax-Desktop/src/renderer/views/home/index.vue:738-741`、`PezMax-Desktop/src/renderer/views/rank/index.vue:257-260`。
- 首次不带 `hash`，拿到 `hash` 后写入内存/本地缓存 —— `PezMax-Desktop/src/renderer/components/NotificationCenter/index.vue:56-59,84,94`；滚动通知每 60s 轮询，登录态切换时清空 `hash` —— `PezMax-Desktop/src/renderer/components/TitleHeader/index.vue:200-220`。
- popup 拉取时机：首页启动、通知中心挂载/打开面板、铃铛挂载 —— `PezMax-Desktop/src/renderer/views/home/index.vue:264-296`、`PezMax-Desktop/src/renderer/components/NotificationCenter/index.vue:143-146`、`PezMax-Desktop/src/renderer/layout/components/HeaderNotice/index.vue:151`。
- 客户端过滤（会被丢弃的通知）：`notifyType === '1'`（强制更新）或 `'4'`（资料下架）时，`notifyId` 已在 `localStorage['processedNotifications']` 中的整条跳过；其它类型每次都弹 —— `PezMax-Desktop/src/renderer/views/home/index.vue:279-290`；写入点 `PezMax-Desktop/src/renderer/components/NotificationDialog/index.vue:92-96,113-118`（该数组只增不减）。
- 通知中心把 popup + scroll **直接拼接**（不按 `notifyId` 去重）再按 `createTime` 倒序 —— `PezMax-Desktop/src/renderer/components/NotificationCenter/index.vue:102-112`。
- 已读存哪：本地 SQLite 表 `notice_reads(id, user_id, notify_id, read_time, UNIQUE(user_id,notify_id))` —— `PezMax-Desktop/src/main/main-utils/database.js:54-63`；IPC `notice-read:list` / `notice-read:mark` —— `PezMax-Desktop/src/main/index.js:495-515`、`PezMax-Desktop/src/preload/index.js:77-81`；铃铛读取已读集合算未读数、点击预览即写已读、「全部已读」批量写 —— `PezMax-Desktop/src/renderer/layout/components/HeaderNotice/index.vue:106-198`。
- 原因：`ptmj_notification` 是广播表，没有 per-user 已读 —— `PezMax-Desktop/src/renderer/layout/components/HeaderNotice/index.vue:55-57`、`PezMax-Desktop/src/main/main-utils/database.js:54`。
- 服务端侧的过滤（时间窗、用户定向）由 `ActivePopup`/`ActiveScroll` 完成 —— `k_admin/internal/kadmin/datum_notification.go:311-328,330-341`；类型域只允许 `1..5` —— `k_admin/internal/kadmin/datum_notification.go:23`。

## 7. 官方端明显的坑

1. `/datum/file/list` 客户端传 `fileName`（`PezMax-Desktop/src/renderer/views/datum/file/index.vue:221`），后端只读 `keyword`/`fileType`/`fileSubject`/`fileSchool`/`fileYear`/`yearFrom`/`yearTo`/`userId`（`k_admin/internal/kadmin/datum_file.go:314-336`）→ 后台列表按文件名搜索实际无效。
2. 书签搜索框传 `title`，后端把 `title` 当 `keyword` 的兜底（`k_admin/internal/kadmin/datum_bookmark.go:73-77`），且 `url`/`resourceType`/`collection` 都是精确筛选。
3. 两个基址混用（§1.1）：`VITE_APP_BASE_API` vs `VITE_APP_TARGET_URL`，写死默认 `http://127.0.0.1:9033`（`PezMax-Desktop/src/renderer/utils/url.js:18`、`PezMax-Desktop/src/renderer/store/modules/upload.js:88`）。
4. `normalizeFileUrl` 与文件树内联拼接是两份不同实现（§4）。
5. `code` 双判 200/0 散落各处（§1.3）；拦截器完全不校验 `code`，任何「HTTP 200 + code≠0」都会被当成功。
6. `getInfo` 解包不一致：store 读 `res.data || res` 再取 `payload.user`（`PezMax-Desktop/src/renderer/store/modules/user.js:62-63`），页面有的读 `res?.data?.user`（`PezMax-Desktop/src/renderer/views/datum/ptmjUser/index.vue:485-491`），有的直接读 `res?.user?.userId`（同文件 `:496`）——后者在 `{data:{user}}` 信封下拿到空串。
7. 多步接口：登录（captcha→login）、找回密码（captcha→securityQuestions→resetPasswordBySecurity）、改密保（password/verify→security GET→security PUT）、书签+封面（addBookmark→uploadCover→updateBookmark）。
8. 书签 id 兜底回查：`POST /datum/bookmark` 若没给出 id，用 `url`/`title`/`resourceType`/`collection` 再查一次列表 —— `PezMax-Desktop/src/renderer/views/home/components/SidePanel.vue:1032-1067,1098-1106`。
9. `GET /datum/bookmark/favorite/list` 的行里含 `userId`，但旧端直接全量建集合、不筛当前用户 —— `PezMax-Desktop/src/renderer/views/home/index.vue:512-514`。
10. 主进程上传/下载的 401 不走 axios，只靠 IPC 事件（§1.4）。
11. `noMessage` 请求头未被实现（§1.4）。
12. 文件树缓存带 `version`，旧缓存直接丢弃 —— `PezMax-Desktop/src/renderer/views/home/index.vue:720-729`。
13. 单页 100 上限导致旧的 `pageSize=1000` 全量拉法失效，才有了 `fetchAllPages` —— `PezMax-Desktop/src/renderer/utils/pagination.js:1-3`。
14. 定义但从未调用的 API：`searchFileList`、`listEbookSubjects`、`fetchEbookContent`、`getEbookFavoriteStatus`、`listUser`、服务端下载记录 CRUD、书签举报除 add 外的全部、`delEbook`（见 §2 各处「无调用点」）。

---

## 8. 旧端与现行 Go 契约的差异清单

> 读法：旧端做法 → 现行 Go 的真实行为 → 新客户端该怎么办。**契约以 Go 为准**。

### 8.1 分页形状
- 旧端：保留 `data.items/total` → `data.rows/total` 的归一（`PezMax-Desktop/src/renderer/utils/request.js:67-73`），列表页到处 `res.code===200||0` 双判，`pageSize` 曾用 1000。
- 现行 Go：`/datum/*` 列表**只有**顶层 `rows[]`/`total`（`k_admin/internal/kadmin/datum_file.go:347-354`、`k_admin/internal/kadmin/datum_activity.go:484-493`、`k_admin/internal/kadmin/datum_user_admin.go:59-66`、`k_admin/internal/kadmin/datum_ebook.go:169-171`）；参数 `pageNum`/`pageSize`，并兼容别名 `page`/`size`，默认 `1/20`，上限 100（`k_admin/internal/kadmin/datum_file.go:268-293`）。全仓 `/datum` 端点无 `data.items` 形状。
- 新客户端：直接按 `code==0` + 顶层 `rows/total` 建模；`items` 归一分支可删；`page`/`size` 别名不必实现，但要知道后端接受。

### 8.2 URL 前缀与「双基址」
- 旧端：剥 `/dev-api`、`/prod-api`、`/stage-api` 再拼 `VITE_APP_TARGET_URL`（`PezMax-Desktop/src/renderer/utils/url.js:22-26`），开发期靠 vite 代理（`PezMax-Desktop/electron.vite.config.mjs:80-85`）。
- 现行 Go：路由直接挂在**站点根** `/datum/...`、`/system/...`（`k_admin/internal/kadmin/datum_auth.go:16-34`、`k_admin/internal/kadmin/datum_notification.go:35-39`）；deploy 侧 nginx 直接 `proxy_pass` 到后端（`deploy/nginx.conf.template:86-90,97-98`）。全仓 `dev-api|prod-api|stage-api` 只出现在 `PezMax-Desktop/` 与其测试里，加上 `README.md:266,270` 对「桌面渲染进程开发代理前缀」的说明；后端与 deploy 无任何前缀约定。
- 新客户端：只需一个站点根 base URL，不要实现 api 前缀剥离与 host 替换逻辑。

### 8.3 会话失效
- 旧端：用文案正则 `/账号已被停用|账号已被封禁/` 判定「直接清会话」（`PezMax-Desktop/src/renderer/utils/request.js:33-43`），其余 401 弹模态框（`PezMax-Desktop/src/renderer/utils/request.js:44-63`）。
- 现行 Go：`requireDatumAuth` 对「无 token / 会话无效」回 **401**「会话已过期，请重新登录」，对「账号被停用」回 **401**「账号已被停用，请重新登录」，身份存储故障回 **503**「身份存储不可用」（`k_admin/internal/kadmin/datum_user_auth.go:462-489`）；停用时同时吊销会话（`k_admin/internal/kadmin/datum_user_auth.go:451-454`）。全部错误体 `code == HTTP 状态码`（`k_admin/internal/kadmin/transport/httpx/response.go:18-25`）。
- 新客户端：按**状态码**（401/403/503）分派，不要匹配中文文案；「账号已被封禁」这一文案在 Go 里不存在（`k_admin/internal/kadmin/datum_user_auth.go:473` 只有「账号已被停用」）。

### 8.4 文件 URL
- 旧端：`normalizeFileUrl` 只换 hostname、保留端口，还处理相对路径与 `minio://`（§4）。
- 现行 Go 写入契约：`Builder.ObjectURL` = `publicBase/bucket/objectKey`（`k_admin/internal/kadmin/platform/fileurl/fileurl.go:1-12,69-75`），内部地址不得落库；读侧 `Parse` 对 `minio://bucket/key` 与任意 host 的 `http(s)` 都能解析（`fileurl.go:89-128`），**相对路径直接不支持**（`fileurl.go:23-26,117-121`）。
- 下发时后端只做一件改写：把 `minio://` 重建为当前 publicBase，其它（含内部 http host）**原样返回**，脏行交给一次性 ETL（`k_admin/internal/kadmin/datum_file.go:462-484`）。
- 下载侧若 URL 解析不出 bucket/key → **404「文件内容不可用」**（`k_admin/internal/kadmin/datum_activity.go:536-541`、`k_admin/internal/kadmin/datum_activity.go:608-612`）。
- 新客户端：`normalizeFileUrl` 里的 host 替换是历史兼容（等 ETL 完可删）；`minio://` 分支已由后端兜住；**相对路径一律当成不可用**，不要自行拼 base。协议不做 http→https 改写。

### 8.5 电子书模块
- 旧端：`PezMax-Desktop/src/renderer/api/datum/` 下**确有** `ebook.js`，覆盖 list/subjects/content/favorite(\*)/report(\*)/upload/desktop favorite/delete（`PezMax-Desktop/src/renderer/api/datum/ebook.js:1-121`）。
- 现行 Go：`/datum/ebook/*` 是独立且完整的端点组（GET 通配分发 list/content/subjects/favorite/favorite/list/report/list，POST favorite/report/upload，DELETE 通配分发，`k_admin/internal/kadmin/datum_ebook.go:38-70`），下载走 `/datum/download/ebook`（`k_admin/internal/kadmin/datum_activity.go:589-652`）。
- 结论：不存在「占位表 / 旧端没有 ebook api」的情况——旧端 ebook 调用与现行后端基本一致；只是旧端有 3 个端点定义了没用（`subjects`、`favorite` 单本状态、`fetchEbookContent`，见 §2.3）。

### 8.6 下载（Range/ETag/416/302）
- 旧端：`.part` 续传 + `Range`/`If-Range` + 416 重下（§5）。
- 现行 Go：`GET /datum/download/file` 由 `http.ServeContent` 承接（`k_admin/internal/kadmin/datum_activity.go:568-573`），因此 Range/If-Range/`Content-Length`/416 由标准库提供；ETag 由后端显式设置，形态是**强标签 `"datum-<fileId>-<size>"`**（`k_admin/internal/kadmin/datum_activity.go:562-566`），电子书是 `"ebook-<ebookId>-<size>"`（`:641-643`）；`Content-Disposition: attachment` + `Content-Type` 由后端给（`:554-561`）。
- **没有 302**：字节从后端代理出去（`k_admin/internal/kadmin/datum_activity.go:542-578`），nginx 也是 `proxy_pass` 到上游（`deploy/nginx.conf.template:86-90`）。
- 每次流式下载都会追加一条下载记录，续传的每个 Range 请求各记一条（`k_admin/internal/kadmin/datum_activity.go:549-552,568-570`）。
- 新客户端：ETag 直接按不透明字符串存取即可（不要解析格式）；`If-Range` 用完整强标签；本地 `.part` 命名与并发上限自定（后端不做去重/限流，限速在 nginx）。

### 8.7 旧端调用但现行口径已变 / 不存在的端点
1. **`GET /datum/favorite/{fileId}` 语义变了**：旧端当「收藏详情」（`PezMax-Desktop/src/renderer/api/datum/favorite.js:16-22`），现行返回 `{favorited: bool}`（`k_admin/internal/kadmin/datum_activity.go:279-293`）。
2. **`PUT /datum/favorite` 不存在**：旧端有 `updateFavorite`（`PezMax-Desktop/src/renderer/api/datum/favorite.js:33-40`），现行只有 `POST /datum/favorite` 与 `GET /datum/favorite/:fileId`（`k_admin/internal/kadmin/datum_activity.go:47-49`）。
3. **管理写接口全部改成管理端 JWT**：`POST/PUT/DELETE /datum/user`（`k_admin/internal/kadmin/datum_user_auth.go:362-366`）、`POST/PUT/DELETE /datum/notification`（`k_admin/internal/kadmin/datum_notification.go:31-33`）、`POST /datum/report/audit/:id` 与 `POST /datum/bookmarkReport/audit/:id`（`k_admin/internal/kadmin/datum_report.go:29,37`）、`DELETE /datum/file-tree/cache`（`k_admin/internal/kadmin/datum_file.go:213`）。旧端把这些管理页挂在 datum 会话下（`PezMax-Desktop/src/renderer/views/datum/notification/index.vue:451`、`PezMax-Desktop/src/renderer/views/datum/report/index.vue:175`），现行会 401。
4. **`POST /datum/bookmark` 现在明确回 `{bookmarkId, id, status}`**（`k_admin/internal/kadmin/datum_bookmark.go:213-214`）——旧端的「回查列表拿 id」兜底（§7.8）可以删。
5. **`POST /datum/bookmark/uploadCover` 自己就落 `cover_image`**（`k_admin/internal/kadmin/datum_bookmark.go:420-429`），旧端随后的 `PUT /datum/bookmark {id, coverImage}`（`PezMax-Desktop/src/renderer/views/home/components/SidePanel.vue:1154-1157`）只是冗余保险（Go 注释亦承认，`:348-350`）。
6. **个人中心统计有专有端点**：`GET /datum/desktop/user/profile` 与 `GET /datum/desktop/user/profile/stats`（`k_admin/internal/kadmin/datum_user_profile.go:33-34`），返回 `uploadCount/downloadCount/favoriteCount/fileFavoriteCount/bookmarkFavoriteCount`（`:74-80`）——旧端用三次 `pageSize=1` 列表取 total（`PezMax-Desktop/src/renderer/views/datum/ptmjUser/index.vue:501-514`）是历史产物。
7. **`getInfo` 不再有初始密码/过期密码提示字段**：旧端读 `payload.isDefaultModifyPwd`、`payload.isPasswordExpired`（`PezMax-Desktop/src/renderer/store/modules/user.js:85-95`），现行 `getInfo` 只回 `user{userId,userName,nickName,avatar,count,status}` + `roles` + `permissions`（`k_admin/internal/kadmin/datum_user_auth.go:787-815`），这两个字段不存在（`k_admin/internal/kadmin/datum_user_auth.go` 内未找到）。
8. **`/datum/file/list` 的筛选字段**：旧端发 `fileName/fileUrl/fileSize/fileFormat/reviewer/fileStatus`（§7.1），现行只认 `fileType/fileSubject/fileSchool/fileYear/keyword/userId/yearFrom/yearTo`（`k_admin/internal/kadmin/datum_file.go:314-336`）；`fileName` 无效。
9. **找回密码是「票据」两步**：第一步消费验证码并签发一次性 reset ticket（`k_admin/internal/kadmin/datum_user_auth.go:693-699`），第二步必须带同一 `uuid`+`username` 才能消费（`:731-741`）——旧端流程一致，但新客户端不能跳过第一步。
10. **新增/未使用的端点**（旧端没有）：`GET /datum/knowledge/manifest`、`GET /datum/knowledge/source`（`k_admin/internal/kadmin/datum_knowledge.go:17-24`）、`DELETE /datum/file-tree/cache`、`DELETE /datum/user/rank/cache`（`k_admin/internal/kadmin/datum_file.go:213`、`k_admin/internal/kadmin/datum_user_auth.go:402-415`）、`POST /datum/user/resetSecurityAnswers`（`k_admin/internal/kadmin/datum_user_auth.go:366`）。
11. **`/system/notification/user/*` 是刻意保留的旧路径**，不要「纠正」成 `/datum/notification/*` —— 后端注释明确客户端逐字调用（`k_admin/internal/kadmin/datum_notification.go:18-21,35-39`）。
