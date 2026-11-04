# 端点目录

> **上游基线**：`PezMax-Go@061a888`（2026-11-04）。来源：`k_admin/internal/kadmin/docs/swagger.json`（摘要）、
> `k_admin/internal/kadmin/datum_*.go`（路由、门禁与响应形状）、`k_admin/internal/kadmin/modules/datum/*`（查询）。
> 上游变化时先改本文与夹具，再改各端。
>
> **状态**：M1 读接口与账户 / 个人中心的参数与响应形状已补齐；写接口只补了上游源码能确认的部分。
> 「形状」列：✅ 已写清参数与响应；**—** 客户端不调用（管理端）。未标 ✅ 的写接口在对应端实现时补，并同步补夹具。
>
> 信封、错误与 401 见 `envelope.md`；账户流程与校验规则见 `auth.md`；分页见 `fixtures/pagination/`。

## 通用约定

- **Base URL**：站点根地址（后端或 edge），客户端自行拼 `/datum/*`，**不要追加 `/api`**。默认值随构建写入，设置里可改（`ARCHITECTURE.md` D13）。
- **鉴权**：`Authorization: Bearer <token>`，token 来自登录响应；HTTP 401 → 清会话、回登录页（保留当前页以便登录后返回）。
- **信封**：成功 `{code:0, data, message:"ok", msg:"ok"}`；列表 `{code:0, rows, total, message, msg}`；错误以真实 HTTP 状态码承载，正文 `{code:<status>, message, msg, data:null}`。详见 `envelope.md`。
- **列表**：`pageNum` + `pageSize`（别名 `page` / `size`），默认 1 / 20，**单页上限 100**。
- **哈希缓存**：标注 `#` 的端点接受 `?hash=`，未变化时回 `unchanged: true` 且 `data` 为空（`datum_file.go:179-195`）。
- **流式**：标注 `⇣` 的端点返回文件流，支持 `Range` / `ETag`；不要按 JSON 解析。
- **门禁**：
  - **匿名**：不需要 token。
  - **会话**：`requireDatumAuth`，无效 / 过期 / 停用 / 改密后回 401（`datum_user_auth.go:436-489`）。
  - **可选会话**：不带 token 时按匿名处理；带 token 时解析用户，用于属主可见待审内容。
  - **管理**：管理端 JWT，客户端不调用，列出只为避免误用。

## 账户 `/datum/user`

| 方法 | 路径 | 门禁 | 用途 | 形状 |
|---|---|---|---|---|
| GET | `/datum/user/captchaImage` | 匿名 | 登录验证码 | ✅ `data{captchaEnabled:true, uuid, img}`；`img` 为不带前缀的 base64 JPEG |
| POST | `/datum/user/login` | 匿名 | 登录 | ✅ 请求 `{username,password,code,uuid}` → `data{token,expiresAt(Unix ms)}` |
| POST | `/datum/user/register` | 匿名 | 注册（含密保三问三答，原子创建） | ✅ 请求见 `auth.md` → `data{userId}` |
| POST | `/datum/user/logout` | 会话 | 登出，服务端作废 token | ✅ `data:true` |
| GET | `/datum/user/getInfo` | 会话 | 当前用户信息 | ✅ `data{user{userId,userName,nickName,avatar,count,status}, roles:["ptmj_user"], permissions:[]}` |
| GET | `/datum/user/securityQuestions` | 匿名 | 找回密码第一步：校验账号与验证码，返回密保问题 | ✅ 查询 `userName,uuid,code` → `data[{question}]` |
| POST | `/datum/user/resetPasswordBySecurity` | 匿名 | 找回密码第二步：验重置工单与密保答案后重置 | ✅ 请求见 `auth.md` → `data:true` |
| GET | `/api/system/config/login` | 匿名（但在 `/api` 下） | 管理端公开登录配置 | — **客户端不调用**；桌面验证码恒开，见 `auth.md` §1 |

> 现有初稿把 `/system/config/login` 当成客户端可用端点，这是错的：它注册在 `/api` 组
> （`system_config.go:168-170`），且只影响管理端登录。删除该误列，客户端不要带 `/api`。

## 个人中心 `/datum/desktop/user/profile`（全部会话）

| 方法 | 路径 | 用途 | 形状 |
|---|---|---|---|
| GET | `/datum/desktop/user/profile` | 个人资料 | ✅ `data{userId,userName,nickName,avatar,count,status}` |
| GET | `/datum/desktop/user/profile/stats` | 上传 / 下载 / 收藏计数 | ✅ `data{uploadCount,downloadCount,favoriteCount,fileFavoriteCount,bookmarkFavoriteCount}`；`favoriteCount = file + bookmark` |
| PUT | `/datum/desktop/user/profile/username` | 改用户名 | ✅ 请求 `{userName}` → `data{userName}`；1–64 字符；重复 409 |
| PUT | `/datum/desktop/user/profile/avatar` | 改头像地址 | ✅ 请求 `{avatar}` → `data{avatar}`；1–500 字节；非空 |
| POST | `/datum/desktop/user/profile/avatar/upload` | 上传头像 | ✅ multipart `file`（≤ 2 MB，JPG / PNG / GIF）→ `data{url,storage,name}` |
| POST | `/datum/desktop/user/profile/password/verify` | 敏感操作前校验登录密码 | ✅ 请求 `{password}` → `data:true`；错误 400「密码错误」 |
| PUT | `/datum/desktop/user/profile/password` | 改密码 | ✅ 请求 `{oldPassword,newPassword}` → `data:true`；**旧 token 立即失效** |
| PUT | `/datum/desktop/user/profile/password/by-security` | 用密保答案重置密码 | ✅ 请求 `{securityAnswerOne..Three,newPassword}` → `data:true`；**旧 token 立即失效** |
| GET | `/datum/desktop/user/profile/security` | 密保问题（不含答案） | ✅ `data{questions:[string]}`；未设置时为空数组 |
| POST | `/datum/desktop/user/profile/security/answer/verify` | 校验单条密保答案 | ✅ 请求 `{answer}` → `data{verified}`；与三问中任意一条匹配即 true |
| PUT | `/datum/desktop/user/profile/security` | 设置 / 更换三问三答 | ✅ 请求 `{password,securityQuestionOne..Three,securityAnswerOne..Three}` → `data:true` |

改密 / 用密保重置成功后，客户端应主动清会话并回登录页（会话绑定密码哈希，见 `auth.md` §6）。

## 资料（试卷）`/datum/file`

| 方法 | 路径 | 门禁 | 用途 | 形状 |
|---|---|---|---|---|
| GET | `/datum/file/tree` # | 匿名 | 文件树 | ✅ 见下 |
| GET | `/datum/file/list` | 可选会话 | 列表 | ✅ 见下 |
| GET | `/datum/file/search?keyword=` | 匿名 | 搜索 | ✅ `data[file]`，最多 50 条；`keyword` 为空回 `[]` |
| GET | `/datum/file/subjects` | 匿名 | 科目及计数 | ✅ `data[{Subject,Total}]`（注意首字母大写，结构体无 JSON tag） |
| GET | `/datum/file/schools?keyword=` | 匿名 | 学校列表 | ✅ `data[string]`，按名称排序，去重 |
| GET | `/datum/file/schools/check?schoolName=` | 匿名 | 校验学校是否在册 | ✅ `data{exists:bool}`；`schoolName` 空 400 |
| GET | `/datum/file/content?fileId=` ⇣ | 可选会话 | TXT / MD 文本预览 | ✅ `text/plain; charset=utf-8`；仅 `txt/md`；> 10 MiB 回 413 |
| GET | `/datum/file/{fileId}` | 可选会话 | 详情 | ✅ `data` 为单个 file payload；待审件仅属主可见，否则 404 |
| POST | `/datum/file` | 会话 | 上传（multipart） | ✅ 见下 |
| PUT | `/datum/file` | 会话 | 属主编辑元信息 | ✅ 请求 `{fileId,fileName,fileYear,fileType,fileSchool,fileSubject,remark}` → `data:true`；**整组字段都要给，不是局部合并** |
| DELETE | `/datum/file/{fileId}` | 会话 | 属主软删除 | ✅ `data:true`；不存在或非属主 404 |
| DELETE | `/datum/file-tree/cache` | 管理 | 清树缓存 | — |

### 文件对象（file payload）

列表、详情、树叶子、收藏、下载记录共用：

```json
{
  "id": 12, "fileId": 12, "label": "期中试卷.pdf", "fileName": "期中试卷.pdf",
  "fileUrl": "https://…", "fileSize": 123456, "fileFormat": "pdf",
  "fileYear": 2025, "fileType": 1, "fileSchool": "实验中学", "fileSubject": "数学",
  "fileStatus": 1, "reviewer": "", "delFlag": 0, "userId": 7,
  "createBy": "", "createTime": "2025-01-01 00:00:00",
  "updateBy": "", "updateTime": "2025-01-01 00:00:00",
  "remark": "", "type": "file"
}
```

- 树叶子会把 `id` 覆盖成 `"file-<fileId>"`（`fileId` 仍是数字）；列表 / 详情里的 `id` 是数字。
- `fileStatus`：0 待审、1 通过、2 未通过、3 被举报下架（与电子书同语义）。
- `fileType`：1 期末、2 期中、3 补考、4 资料、5 其他学校、6 神秘文件、7 电子书（`datum_file.go:48-51`）。
- `fileUrl` 可能是 `http(s)` 外链或相对路径；`minio://` 已由服务端改写为公网地址。内网 host 的过渡归一另见 `fixtures/url/`（待写）。

### 树

`data` 是节点数组；节点只有两种：

```json
{ "id": "folder-…", "label": "数学", "type": "folder", "children": [] }
{ "id": "file-12", "fileId": 12, "label": "期中试卷.pdf", "type": "file", "children": [], … }
```

- 层级固定为 科目 → 学校 → 类型 → 年份，再拼接上传时的 `remark` 目录段。
- 空科目 / 空学校显示为「未知科目」/「未知学校」；年份 ≤ 0 用服务端当前年份。
- 文件夹 id 由路径段哈希生成，不保证跨版本稳定；客户端只把它当节点键，不要解析。
- 只包含已审核（`file_status=1`、`del_flag=0`）文件。

### 列表参数

`pageNum` / `pageSize`、`fileType`、`fileSubject`、`fileSchool`、`fileYear`、`yearFrom`、`yearTo`、`keyword`、`userId`。

- `yearFrom` / `yearTo` 为新增的年份区间过滤（`061a888`）：值须在 1900–2100，且 `yearFrom ≤ yearTo`，否则 400；
  与精确的 `fileYear` 同时出现时两者都生效（区间与等值取交集）。
- 不带 `userId`：匿名口径，只出已审核。
- 带 `userId`：必须是当前会话用户（「我的上传」），可见待审 / 未通过；不匹配 403。
- 返回顶层 `rows` / `total`，行是 file payload。

### 上传

multipart 字段：

| 字段 | 必填 | 规则 |
|---|---|---|
| `file` | 是 | 单文件；默认 ≤ 200 MiB（`KADMIN_DATUM_UPLOAD_MAX_MB` 可配 1–4096） |
| `fileName` | 否 | 缺省用上传文件名；1–255 字符 |
| `fileSubject` | 是 | 1–64 字符 |
| `fileSchool` | 否 | ≤ 100 字符 |
| `fileYear` | 是 | 1900–2100 |
| `fileType` | 是 | 1–7，越界 400 |
| `remark` | 否 | 文件夹路径；按 `/` 分段清理后作为树的自定义目录 |

扩展名白名单：`doc docx ppt pptx pdf jpg jpeg png webp txt md epub mobi azw3`。

- `doc/docx/ppt/pptx` 会转成 PDF 存储，`fileFormat` 落 `pdf`，`fileSize` 是转换后的字节数。
- `txt/md` 必须是 UTF-8，否则 400「文本文件仅支持 UTF-8 编码」。
- 上传成功 `data{fileId,fileUrl,fileName}`；新文件 `fileStatus=0`，不计入公开列表 / 树，直到审核通过。

## 电子书 `/datum/ebook`

| 方法 | 路径 | 门禁 | 用途 | 形状 |
|---|---|---|---|---|
| GET | `/datum/ebook/list` | 匿名 | 列表（仅已上架） | ✅ 查询 `pageNum,pageSize,keyword,subject,ebookType` → `rows` / `total` |
| GET | `/datum/ebook/subjects` | 匿名 | 学科及计数 | ✅ `data[{Subject,Total}]`（同样首字母大写） |
| GET | `/datum/ebook/content?ebookId=` ⇣ | 可选会话 | 在线预览流（inline） | ✅ 文件流；`ETag: "ebook-<id>-<size>"`；不落下载记录 |
| GET | `/datum/ebook/favorite?ebookId=` | 可选会话 | 收藏状态 | ✅ `data{favorited}`；匿名恒 false |
| GET | `/datum/ebook/favorite/list` | 可选会话 | 当前用户收藏的电子书 id | ✅ `data[ebookId]`；匿名 `[]` |
| POST | `/datum/ebook/favorite` | 会话 | 收藏 | ✅ 请求 `{ebookId,userId?}` → `data:true`；重复 409 |
| DELETE | `/datum/ebook/favorite/{ebookId}` | 会话 | 取消收藏 | ✅ `data:true`；关系不存在 404 |
| POST | `/datum/ebook/report` | 会话 | 举报 | ✅ 请求 `{ebookId,userId?,reason,remark}` → `data{reportId}` |
| GET | `/datum/ebook/report/list?userId=` | 会话 | 我的电子书举报 | ✅ `userId` 可省，必须等于会话用户；`rows{reportId,ebookId,ebookName,reason,result,remark,createTime,updateTime}` / `total` |
| POST | `/datum/ebook/upload` | 会话 | 上传（进入待审） | ✅ 见下 |
| DELETE | `/datum/ebook/{ebookId}` | 会话 | 属主软删除 | ✅ `data:true` |

### 电子书对象（ebook payload）

```json
{
  "id": 3, "ebookId": 3, "label": "高等数学", "ebookName": "高等数学",
  "author": "", "publisher": "", "coverUrl": "", "ebookUrl": "https://…",
  "ebookSize": 123456, "ebookFormat": "pdf", "ebookSubject": "数学",
  "ebookType": 1, "ebookStatus": 1, "userId": 7, "remark": "", "type": "ebook"
}
```

- `ebookStatus`：0 待审、1 上架、2 未通过、3 被举报下架。
- `ebookType`：1 教材、2 教辅/参考书、3 课外读物、4 其他（`datum_ebook.go:357`）。
- 匿名列表只出 `ebookStatus=1`；属主凭会话可在详情 / 流里看到自己的待审书，但列表不额外返回。

### 电子书上传

multipart 字段：`file`（必填）、`ebookName`（缺省用文件名，1–255 字符）、`author`（≤ 255 字符）、
`publisher`（≤ 255 字符）、`ebookSubject`（必填，≤ 64 字符）、`ebookType`（1–4）。

- 格式仅 `pdf epub mobi azw3`；大小限制与资料上传相同。
- 成功 `data{ebookId,ebookUrl,ebookName}`；`ebookStatus=0` 待审。

## 书签 `/datum/bookmark`

| 方法 | 路径 | 门禁 | 用途 | 形状 |
|---|---|---|---|---|
| GET | `/datum/bookmark/list` | 可选会话 | 列表 | ✅ 查询 `pageNum,pageSize,keyword`（或 `title`）、`subject,resourceType,collection,url,userId` → `rows` / `total` |
| GET | `/datum/bookmark/{bookmarkId}` | 可选会话 | 详情 | ✅ 单个 bookmark payload；待审仅属主可见 |
| POST | `/datum/bookmark` | 会话 | 新增 | ✅ 请求 `{url,title,description,coverImage,subject,resourceType,collection,remark}` → `data{bookmarkId,id,status:0}` |
| PUT | `/datum/bookmark` | 会话 | 编辑（局部合并） | ✅ 请求同上字段任选 + `id` 或 `bookmarkId`（数字 / 数字字符串均可）→ `data:true` |
| DELETE | `/datum/bookmark/{bookmarkId}` | 会话 | 删除 | ✅ `data:true` |
| POST | `/datum/bookmark/uploadCover` | 会话 | 上传封面 | ✅ multipart `bookmarkId`（或 `id`）+ `file`（≤ 5 MB，JPG / PNG / GIF / WEBP）→ `data{bookmarkId,url,fileUrl}` |

### 书签对象（bookmark payload）

```json
{
  "id": 5, "bookmarkId": 5, "userId": 7, "url": "https://…", "title": "标题",
  "description": "", "coverImage": "", "subject": "", "resourceType": "other",
  "collection": "", "status": 1, "remark": "",
  "createBy": "", "createTime": "…", "updateTime": "…"
}
```

- `status`：0 待审、1 通过、2 未通过、3 被举报下架。
- 创建 / 编辑校验：`url` 1–512 字节、`title` 1–500 字符、`coverImage` ≤ 1000 字符、
  `subject` ≤ 64 字符、`resourceType` ≤ 32 字符（空时落 `other`）、`collection` ≤ 64 字符。
- 列表里 `keyword` 为空时回落到 `title`；两者都是标题 / 描述 / URL 模糊匹配。

## 收藏

| 方法 | 路径 | 门禁 | 用途 | 形状 |
|---|---|---|---|---|
| POST | `/datum/favorite` | 会话 | 收藏资料 | ✅ `{fileId,userId?}` → `true`；重复 409 |
| GET | `/datum/favorite/{fileId}` | 会话 | 资料收藏状态 | ✅ `data{favorited}` |
| POST | `/datum/bookmark/favorite` | 会话 | 收藏书签 | ✅ `{bookmarkId,userId?}` → `true`；重复 409 |
| GET | `/datum/bookmark/favorite/list` | 会话 | 书签收藏关系列表 | ✅ `rows{bookmarkId,userId}` / `total`；**返回全站关系**，客户端按 `userId` 过滤 |
| GET | `/datum/desktop/favorite/list/{userId}` | 会话（本人） | 我的资料收藏 | ✅ file payload rows / `total` |
| DELETE | `/datum/desktop/favorite/{userId}/{fileId}` | 会话（本人） | 取消资料收藏 | ✅ `true` |
| GET | `/datum/desktop/ebook/favorite/list/{userId}` | 会话（本人） | 我的电子书收藏 | ✅ ebook payload rows / `total` |
| DELETE | `/datum/desktop/ebook/favorite/{userId}/{ebookId}` | 会话（本人） | 取消电子书收藏 | ✅ `true` |
| GET | `/datum/desktop/bookmark/favorite/list/{userId}` | 会话（本人） | 我的书签收藏 | ✅ bookmark 子集 rows / `total` |
| DELETE | `/datum/desktop/bookmark/favorite/{userId}/{bookmarkId}` | 会话（本人） | 取消书签收藏 | ✅ `true` |

- 路径里的 `{userId}` 必须等于会话用户，否则 403「没有权限」。
- `GET /datum/ebook/favorite/list` 返回 **id 数组**；`/datum/desktop/ebook/favorite/list/{userId}` 返回 **完整对象行**。两者不要混用。

## 下载

| 方法 | 路径 | 门禁 | 用途 | 形状 |
|---|---|---|---|---|
| GET | `/datum/download/file?fileId=` ⇣ | 会话 | 下载资料流；服务端落下载记录 | ✅ 见下 |
| GET | `/datum/download/ebook?ebookId=` ⇣ | 会话 | 下载电子书流；落 `ptmj_ebook_download` | ✅ 见下 |
| GET | `/datum/desktop/download/list/{userId}` | 会话（本人） | 我的下载记录 | ✅ file payload + `firstDownloadTime` rows / `total` |
| DELETE | `/datum/desktop/download/{userId}/{fileId}` | 会话（本人） | 删除下载记录 | ✅ `true`；不存在 404 |
| POST / PUT / GET / DELETE | `/datum/download`、`/datum/download/{id}`、`/datum/download/{ids}` | 会话 | 下载记录 CRUD（属主） | — **客户端不手动写记录**；流式端点已自动记录，列出备查 |

### 下载流

- `Content-Disposition: attachment; filename=…`；`Content-Type` 由格式决定。
- `ETag`：资料 `"datum-<fileId>-<size>"`，电子书 `"ebook-<ebookId>-<size>"`；内容写入后不变。
- 支持 `Range`；`http.ServeContent` 负责 `206` / `416` / `Content-Length`。
- 资料 / 电子书未审核时仅属主可下载，其余 404。
- 每次成功建立流都会写一条下载记录；**Range 续传每次也会写一条**，这是上游接受的口径
  （`datum_activity.go:520-585`）。客户端「我的下载」按 `fileId` 合并去重，不要按记录条数展示。
- 经 edge 时每 IP 只允许 1 个下载连接（`deploy/nginx.conf.template`）：**下载队列默认串行**，并发 > 1 会被拒。

## 举报

| 方法 | 路径 | 门禁 | 用途 | 形状 |
|---|---|---|---|---|
| POST | `/datum/report` | 会话 | 举报资料 | ✅ `{fileId,userId?,reason,remark}` → `{reportId}` |
| GET | `/datum/report/list` | 会话 | 我的资料举报 / 按文件或结果查询 | ✅ 见下 |
| GET | `/datum/report/{reportId}` | 会话 | 举报详情 | ✅ `{reportId,fileId,userId,reason,result,remark,createTime,updateTime}`；仅本人 |
| PUT | `/datum/report` | 会话 | 编辑举报（仅待处理） | ✅ `{reportId,reason,remark}` → `true` |
| DELETE | `/datum/report/{ids}` | 会话 | 撤回举报（逗号分隔批量） | ✅ `data{deleted}`；一条都没删到 404 |
| GET | `/datum/report/timeline/{reportId}` | 会话 | 举报处理时间线 | ✅ `data[{step,time,result,reason?/remark?/conclusion?}]` |
| POST | `/datum/bookmarkReport` | 会话 | 举报书签 | ✅ `{bookmarkId,userId?,reason,remark}` → `{reportId}` |
| GET | `/datum/bookmarkReport/list` | 会话 | 我的书签举报 | ✅ 行含 `bookmarkTitle`、`bookmarkUrl` |
| GET | `/datum/bookmarkReport/{reportId}` | 会话 | 书签举报详情 | ✅ 仅本人 |
| PUT | `/datum/bookmarkReport` | 会话 | 编辑书签举报 | ✅ 仅待处理 |
| DELETE | `/datum/bookmarkReport/{ids}` | 会话 | 撤回书签举报 | ✅ `{deleted}` |
| GET | `/datum/bookmarkReport/timeline/{reportId}` | 会话 | 书签举报时间线 | ✅ 同上，文案改为书签 |
| POST | `/datum/report/audit/{reportId}`、`/datum/bookmarkReport/audit/{reportId}` | 管理 | 审核 | — |

### 举报列表

- `?fileId=`：该文件的全部举报（含他人），`total = 行数`，不分页。
- `?result=`：按审核结果过滤，分页。
- 不带参数：当前用户提交的举报，联查 `fileName`、`fileSubject`、`fileSchool`，分页。
- `?userId=` 可省；一旦提供必须等于会话用户，否则 403。
- 行形状：资料 `{reportId,fileId,userId,reason,result,remark,createTime,updateTime,fileName?,fileSubject?,fileSchool?}`；
  书签把 `fileId` 换成 `bookmarkId`，联查字段换成 `bookmarkTitle`、`bookmarkUrl`。

### 举报规则

- `reason` 1–500 字符；不能举报自己的内容（400）。
- 同一用户对同一目标同一时间只能有一条待处理举报；重复提交 409
  「举报已受理，请勿重复举报，若遇到问题请联系管理员」。
- `result`：0 待处理、1 属实（内容下架）、2 不属实。
- 时间线是**按举报 id**，不是按文件 id；初稿写成 `/timeline/{fileId}` 是错的，已按源码修正。

## 社区

| 方法 | 路径 | 门禁 | 用途 | 形状 |
|---|---|---|---|---|
| GET | `/datum/user/rank` # | 匿名 | 上传排行（Top 50） | ✅ `data[{userId,userName,avatar,count,uploads}]`；Redis 故障 503，客户端显示旧缓存 |
| GET | `/datum/user/{userId}` | 匿名 | 用户公开资料 | ✅ `data{userId,userName,nickName,avatar,count,status,createTime,updateTime,remark}` |
| GET | `/datum/user/list` | 匿名 | 平台用户列表（管理向） | — 客户端暂无用途 |

## 通知

| 方法 | 路径 | 门禁 | 用途 | 形状 |
|---|---|---|---|---|
| GET | `/system/notification/user/popup?userId=` # | 会话 | 弹窗通知（铃铛 + 首页弹窗） | ✅ `data[notification]`；`userId` 可省，提供时必须等于会话用户 |
| GET | `/system/notification/user/scroll` # | 会话 | 滚动提醒 | ✅ `data[notification]` |
| GET | `/datum/notification/list` | 会话 | 通知列表 | ✅ `rows` / `total`；查询 `notifyType,title,status,displayMode,uploadUserId,materialId` |
| GET | `/datum/notification/{notifyId}` | 会话 | 通知详情 | ✅ 单个 notification payload |
| POST / PUT / DELETE | `/datum/notification…` | 管理 | 通知 CRUD | — |

### 通知对象（notification payload）

```json
{
  "notifyId": 1, "notifyType": "1", "title": "", "content": "",
  "status": "1", "sort": 0, "displayMode": "popup",
  "faultStartTime": "", "faultEndTime": "",
  "maintenanceStartTime": "", "maintenanceEndTime": "",
  "remindBeforeMinutes": 0, "uploadUserId": 0, "materialId": 0,
  "materialTitleSnapshot": "", "publishStart": "", "publishEnd": "",
  "scrollTimeInterval": 0, "remark": "", "createTime": "", "updateTime": ""
}
```

- `notifyType`：1 版本更新、2 系统故障、3 系统维护、4 资料下架、5 公告。
- `displayMode` 与可见窗口由服务端筛选；客户端只按返回结果展示，不自行判断时间段。
- 已读状态**只在客户端本地**按 `(userId, notifyId)` 记录；服务端没有 per-user 已读模型。
- `popup` / `scroll` 用内容哈希：同一组通知未变时 `unchanged=true`、`data=null`，客户端沿用本地。

## 本地 Agent 接口（非客户端 M1 范围）

上游 `061a888` 新增 `/datum/knowledge/*`，供本地 RAG / 阅读助手读取已审核资源的目录与正文：

| 方法 | 路径 | 门禁 | 用途 |
|---|---|---|---|
| GET | `/datum/knowledge/manifest` | 会话 | `{sources, total, partial}`；按 `updated` 倒序，最多返回前 200 条 |
| GET | `/datum/knowledge/source?kind=&id=&revision=` | 会话 | 单资源元信息；`kind` 为 `file` / `ebook`，仅已审核；`revision` 不匹配 409 |
| GET | `/datum/knowledge/content?kind=&id=&revision=` ⇣ | 可选会话 | 正文流；仅 `pdf` / `epub` / `txt` / `md`，≤ 16 MiB，不落下载记录 |

这是独立 Agent 模块的契约，Windows / Linux / Android 客户端**不调用**；列出只为避免与文件预览混淆。
后续若要接本地阅读助手，先补 `spec/api/knowledge.md` 与夹具，再在端上实现。

## 不在客户端范围内（管理）

`POST/PUT/DELETE /datum/user`、`POST /datum/user/resetSecurityAnswers`、`DELETE /datum/user/rank/cache`、
`DELETE /datum/file-tree/cache`、`/datum/notification` 的写方法、两类举报审核、`/api/*`。
`/datum/knowledge/*` 由本地 Agent 使用，不在客户端 M1 范围。
