> **这份文件是什么**：PezMax-Go@061a888（2026-11-04）时点的核对快照，只作**排期输入**。
> 长期有效的契约结论已经并进 spec/api/endpoints.md；基线升级后重做本文，不要把它当契约查。
> 口径：上游两份任务清单（2026-09）写在后端重置之前，**「待办意图」可参考，「现状判断」一律以当时的 Go 代码为准**。

# 上游功能面与待办提炼（PezMax-Go @ 061a888）

> 用途：原生客户端家族（Windows C#/UWP、Linux Rust、Android Kotlin）排期输入。
> 来源：只读快照 `PezMax-Go` @ `061a888`（2026-11-04 工作树）。全部路径相对 PezMax-Go 根。
> 方法：先读 `docs/pezmax-next-tasks.md`、`docs/pezmax-remaining-tasks.md`、`PezMax-Desktop/docs/*`（9 篇），
> 再对照 `k_admin/internal/kadmin/*.go` 与 `PezMax-Desktop/src/**` 逐条核实。
> **重要前提：任务清单写于 2026-09-29，代码已前进约 5 周，文档与代码存在漂移（见 §0）。**

---

## 0. 先看这条：文档已过时的地方（否则会做错）

| 文档说法 | 061a888 代码事实 | 对客户端的后果 |
| --- | --- | --- |
| 电子书 = `ptmj_file.fileType=7`；`ptmj_ebook` 等四张表是占位、**不要启用**（`docs/pezmax-next-tasks.md:50,52`） | 电子书已改为 `ptmj_ebook` 单源，有独立仓储、列表/预览/上传/收藏/举报/下载端点（`k_admin/internal/kadmin/datum_ebook.go:21-48`、`modules/datum/schema.go:199`） | 照旧文档做「电子书复用试卷链路」会与后端不兼容 |
| 电子书「下载不设专有端点，统一走 `/datum/download/file`」（`docs/pezmax-next-tasks.md:50`） | 已有专有端点 `GET /datum/download/ebook?ebookId=`，记录落 `ptmj_ebook_download`（`datum_activity.go:589-592`） | 电子书下载必须用 ebook 端点，否则记错记录表 |
| 桌面端电子书「仅占位，静态功能建设中」（`docs/pezmax-next-tasks.md:51`） | 已实现：`api/datum/ebook.js`（121 行）、活动栏入口（`src/renderer/views/home/components/ActivityBar.vue:7-9`）、上传表单（`UploadPanel.vue:35-40,551`） | 任务二对上游已基本完成，客户端可直接对标 |
| 下载记录「P0 字段名不一致：主进程返回 `savedPath`，渲染层读 `filePath`」（`docs/pezmax-next-tasks.md:217`） | 已统一为 `filePath`（`src/main/index.js:735`、`home/index.vue:408`、`views/datum/download/index.vue:237`） | 不要为新客户端复制兼容分支，直接读 `filePath` |
| 任务三「全局令牌桶缺失」（`docs/pezmax-next-tasks.md:83`） | 已实现应用侧出站令牌桶（`qos.go:29-117`），并把下载/头像/封面/上传物划入同一 bulk 桶（`qos.go:45-50`） | 多连接下载**不会**更快，只会互抢同一个桶 |
| 旧 #19「封禁后存量 datum 会话仍有效」（`docs/pezmax-remaining-tasks.md:82`） | 已修：封禁/删除用户后存量会话被拒（`datum_user_auth.go:462-473`，测试 `datum_session_status_test.go:9-58`） | 客户端把 401「账号已被停用」当登出处理即可 |
| `.env.production` 里 `127.0.0.1:9033` 待换（`docs/pezmax-next-tasks.md:226`、`docs/pezmax-remaining-tasks.md:96`） | 该文件已不存在；但 `src/renderer/utils/url.js:18` 仍硬编码 `http://127.0.0.1:9033` 作兜底，`.env:1` 的 base API 也是本机 | 客户端不许把任何主机地址写死 |

**结论：以任务清单定「做什么」，以 061a888 代码定「怎么接」。**
> 完整的逐条核对（文档说法 → 代码事实 → 结论）见文末 **§7「文档与当前代码的偏差」**；§0 只是最容易踩的 7 条。

---

## 1. 功能面清单（用户可见能力）

### 1.1 账户与安全
- **能力**：注册、登录（图形验证码）、登出、密保三问设置/校验、按密保重置密码、改用户名、改头像（直传）、改密。
- **后端就绪度：就绪**。`POST /datum/user/{login,register,logout,resetPasswordBySecurity}`（`k_admin/internal/kadmin/datum_user_auth.go:367-370`）；个人中心全套（`datum_user_profile.go:31-44`）。
- **官方桌面端：已实现**。`api/datum/desktopUser.js:5-71` 覆盖改用户名/头像/上传头像/密码校验/改密/密保读写。
- **坑**：
  - 封禁即时生效，返回 `401 账号已被停用，请重新登录`（`datum_user_auth.go:462-473`），部分路径 `403 账号已被停用`（同文件 `:541`）；必须按文案区分「会话过期」与「账号停用」。
  - 改密/重置后旧会话是否失效：**未核实**（未见对应测试断言）。
  - 登录锁定与验证码在 `security.go`（Redis 计数），Redis 故障时的行为**未核实**。

### 1.2 资料（试卷）浏览与预览
- **能力**：文件树（按 学科→学校→类型→年份）、分页列表、多条件搜索（关键词/学科/学校/年份/类型）、年份区间 `yearFrom/yearTo`、科目与学校联想、文件详情、文本类在线预览（txt/md ≤10MB）。
- **后端就绪度：就绪**。读路由集中在一个通配分发器 `/datum/file/{list|tree|subjects|schools|schools/check|search|content|:id}`（`datum_file.go:216-243`）；年份区间在 SQL 分页前过滤（`datum_search.go:13-31`）。
- **官方桌面端：已实现**。资源树 + `MainEditor` 预览 + `FileInfoDrawer` 详情；树/排行走 hash 协商（`unchanged=true` 时复用本地副本，`datum_file.go:31-35`）。
- **坑**：
  - `pageSize` 服务端硬上限 100，需要全量只能按 `total` 循环（`datum_file.go:286-292`）——桌面端为此写了 `fetchAllPages`，最坏 100 页串行。
  - **树接口在 Redis 故障时直接 503，无降级**（`datum_file.go:357-364`），见 §3.1。
  - 树/排行响应带 `hash`+`unchanged` 协议，客户端若忽略会重复拉全量。

### 1.3 下载
- **能力**：单文件下载（服务端代理流），支持 Range 断点续传、强 ETag、下载记录、我的下载列表、按用户删除记录。
- **后端就绪度：就绪**。`GET /datum/download/file?fileId=`（`datum_activity.go:520-579`）：会话鉴权 → 校验可见性（`only approved` 或属主，`:532-534`）→ 写 `ptmj_file_download`（`:549-552`）→ `ServeContent` 提供 Range/ETag（`:562-573`）。我的下载 `GET /datum/desktop/download/list/:userId`（`:38-39`）。
- **官方桌面端：已实现**，且是**主进程 `.part` 续传**：`Range` + `If-Range` + 416 兜底 + 原子改名（`src/main/index.js:595-610,616-623,711-735`）。
- **坑**：
  - 每次 Range 恢复都会各记一条下载记录（`datum_activity.go:568-570`），统计口径会偏大。
  - **无下载取消**：全链路没有 `request.abort()`（`src/main/index.js` 仅有上传的 `uploadAbortController:965-987`）。任务三.4 要求补取消（`docs/pezmax-next-tasks.md:85`）。
  - 无 presigned URL、无 X-Accel-Redirect、无边缘缓存（`docs/pezmax-next-tasks.md:10`，`deploy/nginx.conf.template:86-89` 只有 `limit_conn`/`limit_rate`）。

### 1.4 收藏（文件 / 书签 / 电子书）
- **能力**：收藏/取消收藏、收藏状态、我的收藏（三类）、收藏 id 全量集合。
- **后端就绪度：就绪**。`POST /datum/favorite`、`GET /datum/favorite/:fileId`、书签收藏 `POST /datum/bookmarkFavorite`（`datum_activity.go:48-54`）；三类「我的收藏」列表 `GET /datum/desktop/{favorite,bookmark/favorite,ebook/favorite}/list/:userId`（`:38-45`）；电子书收藏独立端点（`datum_ebook.go:44-47,194-280`）。
- **官方桌面端：已实现**。收藏页 + 收藏 id 全量刷新（`home/index.vue:487`）。
- **坑**：
  - 收藏 id 全量靠翻页累积（上限 100 页），任务六.3 建议后端出「收藏 id 全量端点」（`docs/pezmax-next-tasks.md:181`），**尚未提供**。
  - 管理端缺 `ptmj_file_favorite` CRUD（旧 #18，`docs/pezmax-remaining-tasks.md:73-75`）——与客户端无关，但说明该表在后台不可见。

### 1.5 电子书
- **能力**：电子书列表（关键词=书名/作者/出版社、学科、类型）、学科聚合、在线预览（pdf→内建、epub→epub.js）、专有下载端点、收藏、举报、上传（pdf/epub/mobi/azw3）、属主软删。
- **后端就绪度：就绪（且已超出 9-29 文档范围）**。`ptmj_ebook` 单源（`modules/datum/schema.go:199-266`）；路由 `datum_ebook.go:38-89`；列表分页 `:143-173`；上传白名单子集 `:354-365`；下载 `datum_activity.go:589-620`。
- **官方桌面端：已实现**。`api/datum/ebook.js:1-121`（含 `ebookPreviewUrl` 用 query token、`ebookDownloadUrl`）、`EbookInfoDrawer`、上传入口（`UploadPanel.vue:551-572`）。
- **坑**：
  - 列表筛选参数叫 `subject`，返回字段叫 `ebookSubject`（`datum_ebook.go:153`、`:132`），易错。
  - mobi/azw3 无内建预览，只能下载后本地打开（`datum_ebook.go:33-35`）。
  - 上游没有电子书内容运营数据；**上线即空栏**的风险仍在（`docs/pezmax-next-tasks.md:61`）。

### 1.6 书签（外部链接）
- **能力**：匿名列表（三级分类：专栏→科目→书签）、关键词/资源类型/专栏/科目过滤、详情、属主增删改、封面上传（≤5MB，图片嗅探）。
- **后端就绪度：就绪**。`datum_bookmark.go:32-36`（GET 通配 + POST/PUT/DELETE + `/uploadCover`）。
- **官方桌面端：已实现**（`SidePanel.vue` 书签视图，Bento 布局）。
- **坑**：
  - 封面必须在拿到 `bookmarkId` 后才能上传（旧 changelog 记录的二步协同，`PezMax-Desktop/docs/changelog.md:27`）——新客户端要实现「先建后传」。
  - 待审书签仅属主可见（`docs/pezmax-remaining-tasks.md:18`）。

### 1.7 上传（含转档）
- **能力**：试卷/资料上传（doc/docx/pdf/jpg/png/webp/txt/md/ppt/pptx）、Office 格式自动转 PDF、电子书上传（pdf/epub/mobi/azw3）、属主改元数据/删除。
- **后端就绪度：就绪**。`POST /datum/file`（`datum_file.go:206,723-865`）：默认 200MB、可由 `KADMIN_DATUM_UPLOAD_MAX_MB` 调到 4096（`:725-728`、常量 `:40-43`）；白名单 `:667-671`；转档 `doc/docx/ppt/pptx`→PDF（`:807-825`），soffice 起不来返回 **503「转档服务不可用」**（`:814-815`）；txt/md 仅 UTF-8（`:790-799`）；入库即 `fileStatus=0` 待审（`:855`）。
- **官方桌面端：已实现**，并叠加 BYOK「上传命名/分类建议」（`PezMax-Desktop/docs/assistant.md:21-31`）。
- **坑**：
  - 转档默认 2 分钟超时、2 并发（`k_admin/internal/kadmin/platform/convert/convert.go:34-47`），大 PPT 会排队；客户端要有长耗时提示与重试。
  - 转档后存的是 PDF，但 `fileName` 保留原名（`datum_file.go:823,853-855`），下载时扩展名与内容可能不一致。
  - 上传取消只有桌面端做（`src/main/index.js:965-987`），后端无取消语义。

### 1.8 举报与审核时间线
- **能力**：举报文件/电子书/书签、我的举报列表、单条举报生命周期时间线（提交→审核结论）、审核结论联动下架 + type-4 通知。
- **后端就绪度：就绪（审核侧仅管理端）**。举报读/写 `datum_report.go:23-26`；时间线 `:238-277`；书签同构 `:333-340,527`；电子书举报 `datum_ebook.go:45,65-66`；审核是**管理端 JWT + `datum:report:audit`**（`datum_report.go:29,37`），客户端无此权限。
- **官方桌面端：已实现**。`ReportTimelinePanel.vue` 已合并电子书类型（`:29-30`）。
- **坑**：
  - 审核**没有封禁分支**（`datum_report.go:283-333` 只有「属实→下架→通知」），旧 #19 的审核联动封禁未做（`docs/pezmax-remaining-tasks.md:81`）——别向用户承诺「举报会封号」。
  - 同目录存在拼写错的重复组件 `ReporTimeLinePanel.vue`（无任何引用，`docs/pezmax-next-tasks.md:223` 要求删除，仍未删）——属于上游死文件，客户端不要参照。

### 1.9 通知
- **能力**：弹窗通知（按 sort）、滚动公告、5 种类型（1 版本更新 2 系统故障 3 系统维护 4 资料下架 5 公告）、已读状态。
- **后端就绪度：就绪（无 per-user 已读模型）**。客户端 feed 挂在**旧 RuoYi 根路径** `/system/notification/user/{popup|scroll}`，datum 会话鉴权（`datum_notification.go:35-39`）；管理 CRUD 在 `/datum/notification`（`:30-33`）。
- **官方桌面端：已实现**，已读是**本地记账**：主进程 SQLite `notice_reads` 表（`src/main/main-utils/database.js:56,176,189`）。
- **坑**：
  - 已读**只在本地**，不跨端、不跨设备（`docs/pezmax-remaining-tasks.md:41`）。
  - feed 路径不在 `/datum` 下，做一个统一的「anonymous/datum」两档 host 抽象会被它破例。

### 1.10 社区 / 排行
- **能力**：上传排行榜（Top50）、hash 协商缓存、点击榜行看他人公开资料（头像/昵称/上传数/加入时间）。
- **后端就绪度：部分**。排行 `GET /datum/user/rank`（分发在 `datum_file.go:634-641`），真数据是 SQL `WHERE status='1' AND count>0` 的 top50（`modules/datum/stats.go:28-36`），Redis 只是缓存层；**Redis 故障 → 503，零降级**（`datum_file.go:634-639`）。
- **官方桌面端：已实现**。 `views/rank/index.vue:42,291-314` 先渲染列表数据再异步 `getUser` 刷新，tab 内嵌详情面板。
- **坑**：
  - 503 已被误读为「排行榜显示异常」的头号嫌疑（`docs/pezmax-next-tasks.md:134,138-143`），客户端必须自带降级。
  - `count` 与 `uploads` 双字段并存（`datum_file.go:651-653`），别只读一个。
  - 新库/重上线库若 `count` 未迁或为 0，榜为空（`docs/pezmax-next-tasks.md:139`）。

### 1.11 个人中心
- **能力**：我的资料、统计（上传/下载/收藏数，区分文件收藏与书签收藏）、改资料/头像、改密、密保管理。
- **后端就绪度：就绪**。`/datum/desktop/user/profile`、`/profile/stats`（`datum_user_profile.go:31-44,67-81`）。
- **官方桌面端：已实现写路径**（`api/datum/desktopUser.js:5-71`）；`GET /profile` 与 `/profile/stats` 在渲染层**无调用方**（已 grep），桌面端用登录态 store 的值。
- **坑**：统计把文件收藏与书签收藏合并成一个 `favoriteCount` 又单独给两项（`:74-80`），口径需与产品对齐；**不含电子书收藏**。

### 1.12 设置
- **能力**：主题/深浅色、壁纸与模糊、下载目录、静默下载、组件尺寸等。
- **后端就绪度：未做（设计如此）**。后端没有任何「用户偏好」端点；`system_config` 是管理端配置（`k_admin/internal/kadmin/system_config.go`）。
- **官方桌面端：已实现，全本地**。下载目录/静默下载由主进程设置驱动（`src/main/index.js:557-579`）。
- **坑**：设置全本地 → 三端不共享，也无法跨设备同步；`.env` 里的 base API 是构建期固定的本机地址（`src/main/index.js` 与 `PezMax-Desktop/docs/assistant.md:129`），换部署地址要重新构建。

---

## 2. 明确标为「桌面端 / 客户端」的待办条目

| 原文标题 | 一句话要求 | 出处 |
| --- | --- | --- |
| 任务一 · CDN 剪切机制定案与落地 | 内网无 CDN 环境下定案：边缘缓存 / presigned 直连 / 真 CDN 三选一，客户端只受影响于「下载是否直连」 | `docs/pezmax-next-tasks.md:15` |
| 任务二 · 电子书模块桌面端开发 | 电子书列表/详情/收藏/举报/下载/上传全链路打通（上游 061a888 已基本完成，可作参考实现） | `docs/pezmax-next-tasks.md:44` |
| 任务三.4 · 下载取消 | 桌面端补 IPC 取消 + 进度条取消按钮，取消即释放连接与配额（上游**未做**） | `docs/pezmax-next-tasks.md:85` |
| 任务三.5 · 电子书独立档位评估 | 电子书与试卷共享 `/datum/download/`，按体积热度决定是否独立档位 | `docs/pezmax-next-tasks.md:86` |
| 任务六 · 桌面端加载策略：可中断 + 可跳转 | 加载中可中断、可切页；启动请求并行化、去假延时、失败降级（上游**未做**：`GlobalLoader.vue:47-61` 仍全屏吞点击，`home/index.vue:951-957` 仍串行 + 固定 400ms） | `docs/pezmax-next-tasks.md:165` |
| 任务七 · 桌面端用户详情 | 排行行/贡献者/上传者三入口打开同一 Drawer，展示部分公开数据 + TA 的公开文件列表 | `docs/pezmax-next-tasks.md:192` |
| 任务八.2 · `code===200` 硬判定 | 桌面端信封兼容（kadmin `code=0`），列出 5 处遗漏（上游已修，grep 无残留） | `docs/pezmax-next-tasks.md:218-222` |
| 任务八.5 · `normalizeFileUrl` 弃用计划 | 服务端不再产出脏 URL 后，客户端 host 改写逻辑标记弃用并观察一个版本 | `docs/pezmax-next-tasks.md:225` |
| 任务八.6 · `.env.production` 域名 | 上线前换真实域名（文件已删，但 `utils/url.js:18` 仍硬编码本机地址） | `docs/pezmax-next-tasks.md:226` |
| 任务九 #24 · `/common/upload` 遗留清理 | `FileUpload`/`ImageUpload`/`Editor` 默认 action 指向不存在的端点，改接或删组件 | `docs/pezmax-next-tasks.md:243` |
| 旧 #5 · 通知铃铛切换（已完成） | 铃铛改接 datum popup 端点 + 本地已读记账，可作客户端对标答案 | `docs/pezmax-remaining-tasks.md:36` |
| 旧 #16 · 内嵌 admin 入口下线 | `VITE_AUTH_ENTRY_MODE=admin` 改为跳转外链或移除（桌面端专属） | `docs/pezmax-remaining-tasks.md:105` |
| 旧 #17 · 前端长期项 B1/B4/B7/D3 | 记住密码改系统凭据库、权限指令对齐、`/dev-api` 更名等（桌面端专属） | `docs/pezmax-remaining-tasks.md:106` |

> 备注：任务四（URL 映射）、任务五（Redis）、任务一/三（CDN/QoS）、任务九其余项均为**后端/管理端**工作，
> 客户端只在「下载行为」「URL 兜底」两处被动受影响，已并入 §3。

---

## 3. 后端尚未就位 / 客户端必须等或规避的项

### 3.1 Redis 故障 → 排行与文件树 503（**最高优先规避**）
- 事实：`stateStore.load` 对非 miss 错误直接上抛，排行/树随即 503（`datum_file.go:83-96,357-364,634-639`）。修复方向是「Redis 错误时记日志并直查 SQL」（`docs/pezmax-next-tasks.md:143`），**截至 061a888 未修**。
- **客户端该怎么做**：把 503 当「暂时不可用」而不是登录失效；排行榜/树失败要出空态 + 重试，并用本地缓存渲染上次结果；启动链里绝不能因树失败阻塞首屏。

### 3.2 限速与「无 CDN」
- 事实：应用侧有**全局 bulk 令牌桶**（默认总上行 10Mbps × 80%，`qos.go:29-50,71-117`），把 `/datum/download/`、`/api/files/`、`/api/uploads/`、`/uploads` 全划进同一桶（`qos.go:45-50`）；边缘 nginx 有弹性分档 + `limit_conn`（`docker-compose.yml:206-209`、`deploy/nginx.conf.template:86-89`），但**零 `proxy_cache`**（`docs/pezmax-next-tasks.md:10`）。
- **客户端该怎么做**：不要用多连接/分片下载「加速」——同一个桶，只会互相抢；批量下载要串行并给出总进度与 ETA；必须实现取消，否则占用全局额度直到服务端超时。预估带宽按「单连接 = 剩余桶容量」算，不按线路带宽算。

### 3.3 内网 / 本机 URL 脏数据
- 事实：`KADMIN_MINIO_PUBLIC_BASE` 默认注入 `http://127.0.0.1:29000`（`docker-compose.yml:133`）；读取侧只重建 `minio://` 形态，http 内网 host **原样透传**（`datum_file.go:462-484`）；批量修数工具（任务四）**未实现**。
- **客户端该怎么做**：必须在核心层保留一个「URL 兜底改写」开关（撤除条件：任务四两阶段完成）。注意别照抄 Electron 的实现——`utils/url.js:39-42` **只改 hostname 不改 port**，`http://minio:9000/...` 会被改成 `<目标host>:9000`，仍是错地址；正确做法是 host+port+scheme 整体按配置重写，且默认关闭、可注入。

### 3.4 匿名用户投影过宽
- 事实：`GET /datum/user/:userId` 的公开投影返回 `remark`（运营备注）与 `status`（`datum_user_admin.go:26-40`）；任务七.1 要求收敛，**未做**。
- **客户端该怎么做**：不要渲染 `remark`；`status` 只用于显示「该用户不可见」，不要展示原值；抓包验证自己的详情页不泄漏这两个字段。

### 3.5 缺「TA 的公开文件列表」端点
- 事实：任务七.2 规划的 `GET /datum/desktop/user/{userId}/files` **不存在**（`datum_user_profile.go:31-44` 只有本人 profile；`datum_activity.go:38-45` 只有本人的列表）。
- **客户端该怎么做**：M1 阶段的用户详情只做「头像/昵称/上传数/加入时间」，把「TA 上传过什么」列为阻塞项等待后端。

### 3.6 无 presigned URL，下载与预览都必须带鉴权
- 事实：下载走服务端代理流 + datum 会话（`datum_activity.go:520-536`）；没有 302/presigned（`docs/pezmax-next-tasks.md:30`）。电子书在线预览在桌面端靠 **query token** 走 iframe（`api/datum/ebook.js:32-35`）。
- **客户端该怎么做**：原生客户端统一用 Authorization 头拉流，不要模仿 query token；预览器需要能接受自定义 header 的 HTTP 客户端（WPF/Rust/Kotlin 都能做到），别把 token 拼进 URL（会进日志与 WebView 历史）。

### 3.7 电子书数据模型与旧文档冲突
- 事实：见 §0 第一行。`ptmj_ebook` 已是主源，`ptmj_file` 的 `fileType=7` 只在类型字典里留名（`datum_file.go:48-51`）。
- **客户端该怎么做**：先改本仓 `spec/`（AGENTS.md 铁律 2），再实现；不要同时支持两套电子书模型。

### 3.8 明确不做、客户端不要接的东西
- `PUT /datum/file/approvePendingByUser/{userId}` 批量过审：延后，未实现（`docs/pezmax-remaining-tasks.md:28-30`）。
- `/common/upload`、`/common/download*`、`/datum/*/export`、`/unlockscreen`：从未实现或已取消（`docs/pezmax-remaining-tasks.md:26,46-47,56-58`）。
- 审核类端点（`/datum/report/audit/:id` 等）需要**管理端 JWT + 权限码**（`datum_report.go:29,37`），桌面端无权调用。
- Excel 导出、管理端注册、在线用户强退、岗位/数据权限：决策不迁移（`docs/pezmax-remaining-tasks.md:116-128`）。

### 3.9 其它契约细节（易踩）
- 分页上限 100、`rows/total` 在顶层（`datum_file.go:286-292,347-354`）。
- 已读通知无服务端模型，客户端要自建本地表（`main-utils/database.js:56`）。
- 举报审核不封号（§1.8），封禁生效走 401 文案（§1.1）。
- `savedPath` 类字段 bug 已被上游修复，**不要**为它写兼容层（§0）。

---

## 4. 新增的 AI / 助手能力面

### 4.1 组成
- `PezMax-Agent/`：Python + LangGraph 独立模块，三种任务图：**找资料**（`search`）、**上传建议**（`upload`）、**阅读增强**（`reading`，PDF/EPUB 所选页问答/摘要/提纲）。
- 桌面端通过**主进程 stdio 子进程**调用，每轮一个进程、结束后退出，**不开本机 HTTP 端口**（`PezMax-Desktop/docs/assistant.md:89-92`、`agent-engineering.md:24`）。
- 另有一个**可选**的独立 FastAPI + MCP 服务（`PezMax-Agent/docs/service.md`），桌面端默认不启动（`agent-engineering.md:85`）。

### 4.2 Go 侧暴露的接口（知识库/平台 RAG）
| 端点 | 鉴权 | 行为 | 证据 |
| --- | --- | --- | --- |
| `GET /datum/knowledge/manifest` | **datum 会话** | 返回已上架、未删、格式/大小受支持的资料与电子书目录（文件 ID、分类、版本），**不返回存储地址**；`total > len(rows)` 时 `partial=true` | `k_admin/internal/kadmin/datum_knowledge.go:17-34` |
| `GET /datum/knowledge/source?kind=file\|ebook&id=&revision=` | **datum 会话** | 重新核验单个来源与版本；`revision` 不匹配返回 **409**「资源版本已变化」 | 同文件 `:36-88` |
| `GET /datum/knowledge/content?kind=&id=&revision=` | **匿名可读** | 只读当前公开上架资源正文，服务端存储直读；不产生下载记录；`no-store` + `nosniff` + `inline` | 同文件 `:21-23,90-121` |

约束：单文件 ≤16 MiB（`datum_knowledge.go:13,70-74`）；正文格式仅 `pdf/epub/txt/md`（`:70-74`）；超限 413/422。
平台侧索引是本机 SQLite 缓存，最多 200 篇、有片段/字符配额、无 OCR（`PezMax-Agent/docs/platform-knowledge.md:45-49`）。

### 4.3 与普通客户端功能是否耦合
- **不耦合、完全可选**：
  - 找资料只用**已有的只读目录接口**（`/datum/file/list`、`/datum/ebook/list`、`/datum/bookmark/list`），唯一后端改动是 `file/list` 支持 `yearFrom/yearTo`（`docs/…next-tasks.md:97` 所述，代码已就位 `datum_search.go:13-31`）。
  - 上传建议与阅读增强**不需要后端**：`task:"upload"` 不构造目录客户端、不携带平台令牌、不提供工具；`task:"reading"` 同理（`assistant.md:90-91`）。
  - 知识库（平台 RAG）需要上面三个端点，但**只读**，不写业务表。
- **鉴权要点**：模型 API Key 是 **BYOK**，存主进程内存或 OS 加密（safeStorage），**不进 PezMax Go 后端、不进 localStorage**（`assistant.md:64-67`）；平台登录令牌与模型 Key 分开使用，平台令牌不进模型请求/图状态（`assistant.md:65`）。
- **可选择性结论**：普通客户端功能（浏览/下载/收藏/通知/个人中心/设置）**不依赖**任何 AI 端点；不实现助手不损失任何 M1 能力。

### 4.4 若原生客户端要做，成本在哪
- 找资料：中成本（模型调用 + 白名单工具 + 真实卡片投影 + 预算 5 模型/8 检索，`assistant.md:95`）。
- 上传建议：低成本（1 次模型调用、无工具、结果默认不勾选，`assistant.md:21-31,33`）。
- 阅读增强：高成本（需要 PDF.js/PDFium 选页提取 + epub 解析 + 严格 JSON 结构校验 + 引文核验 + 最多一次纠正，`agent-engineering.md:38-56`），且验证记录明确「未确认完整 PDF 引用跳转、未跑完整 EPUB 原生界面流程」（`reading-assistant-verification.md:35`）。
- 知识库：高成本（SQLite 索引、BM25/向量、`.platform` 分库、账号隔离），且已验证范围只是「本机有界平台 RAG」（`platform-knowledge.md:45`）。

---

## 5. 给原生客户端的排期建议

M1 已定：日常浏览 + 阅读 + 下载 + 收藏 + 通知 + 个人中心 + 设置。下表只排 **M1 之外**的条目。

### 5.1 必须有（补齐产品闭环，后端已就绪）
| 条目 | 理由 | 后端 | 成本 |
| --- | --- | --- | --- |
| 电子书模块（列表/详情/预览/收藏/下载/上传） | 产品四大内容形态之一，**后端已就绪**，桌面端已有参考实现 | 就绪（`datum_ebook.go:38-173`） | 中 |
| 举报提交 + 我的举报时间线（试卷/电子书/书签三类） | 内容治理闭环，缺它则用户无处投诉；时间线端点已就绪 | 就绪（`datum_report.go:238-277`） | 低 |
| 排行（含 503 降级） | 社区入口，「排行榜显示异常」是上游已知投诉 | 部分（`datum_file.go:634-641`） | 低 |
| 下载取消 + 串行批量下载 | 上游未做且直接决定体验与带宽占用 | 就绪（`datum_activity.go:520-579`） | 低 |

### 5.2 应当有（体验/一致性，有依赖）
| 条目 | 依赖 | 成本 |
| --- | --- | --- |
| 启动可中断 + 视图可跳转（并行启动、超时降级、骨架屏） | 无，但需要客户端自建「请求取消令牌池」——上游三年没做，正是集中踩坑处（`docs/…next-tasks.md:169-186`） | 中 |
| 统一用户详情（排行/贡献者/上传者同源） | 缺 `TA 的公开文件列表` 端点（§3.5），先做无列表版本 | 中 |
| 书签完整能力（若 M1 未含封面/三级分类） | 就绪（`datum_bookmark.go:32-36`） | 中 |
| URL 兜底改写开关 | 任务四进度（§3.3） | 低 |

### 5.3 可以没有（后置或不做）
| 条目 | 结论 |
| --- | --- |
| AI 助手三件套（找资料/上传建议/阅读增强/知识库） | 完全可选、BYOK、与 M1 无耦合；阅读增强与知识库成本高、验证边界窄（§4.4） |
| mobi/azw3 内建预览 | 上游也只给「下载后本地打开」（`datum_ebook.go:33-35`） |
| 多连接/分片加速下载 | 与全局 bulk 令牌桶冲突，做了反而更慢（§3.2） |
| 审核类、管理端、批量过审、Excel 导出 | 客户端无权限或后端不存在（§3.8） |
| 电子书独立 QoS 档位 | 取决于后端排期（任务三.5） |

### 5.4 低成本高价值（建议插在 M1 尾部一起做）
1. **电子书只读版**（列表/详情/预览/收藏/下载）——后端零改动、桌面端可直接对标、补齐内容形态。
2. **举报提交 + 我的举报时间线**——三个 POST/GET，两天量级，补治理闭环。
3. **排行的 503 降级与本地缓存**——不是新功能，但能消掉上游最大的体验投诉类别。
4. **下载取消 + 串行队列 + 总进度**——原生客户端比 Electron 更容易做好，且直接缓解带宽争抢。
5. **URL 兜底改写（host+port 整体重写、默认关、可配置）**——任务四完成前是硬需求，工程量小。
6. **401 分流：会话过期 vs 账号停用**——按后端文案区分（`datum_user_auth.go:462-473`），避免误报「登录过期」。

### 5.5 建议的排期顺序
1. M1 收尾 + 5.4 的 1/2/6（同一批：内容形态 + 治理 + 鉴权细节）。
2. 5.4 的 3/4/5（稳定件，可与 M1 并行）。
3. 5.2 的启动可中断（客户端架构件，越早越省后续返工）。
4. 用户详情（等 `TA 的公开文件列表` 端点，先做占位）。
5. AI 助手按「上传建议 → 找资料 → 阅读增强 / 知识库」排序，每步都保持可独立关闭。

---

## 6. 未核实项（写排期前需向上游确认）
- 改密/重置密码后旧 datum 会话是否立即失效（未见断言）。
- Redis 故障时登录/验证码/锁定链路的行为（`security.go` 未通读）。
- 库里是否仍有 `file_type=7` 存量行（代码无迁移逻辑，见 §7.1；需查实库）。
- 电子书 `/datum/ebook/content` 已核实：已过审匿名可读、属主可看自己未过审的书、不落下载记录（`datum_ebook.go:480-499`）。
- `GET /datum/knowledge/content` 的 CORS 是否覆盖原生客户端（桌面端依赖 `/uploads/*.pdf` 公开 CORS，`assistant.md:48`）。
- 生产环境是否启用 `edge` profile（决定 §3.2 的限速是「应用层桶」还是「桶 + nginx 档位」双层）。

---

## 7. 文档与当前代码的偏差（逐条核对）

> 口径：`docs/pezmax-next-tasks.md`（2026-09-29）与 `docs/pezmax-remaining-tasks.md`（2026-09-23）的「现状 / 未做 / 缺失」段落写在**后端重置之前**，
> 此后代码修掉了大量历史遗留问题。**只有待办「意图」还有参考价值**；凡涉及「现状」的判断，一律以本节为准。
> 结论取值：已完成 / 部分完成 / 仍缺 / 已作废。

### 7.1 电子书到底存在哪张表（**`ptmj_ebook` 成立**）
- 文档说法：「电子书 = `ptmj_file.fileType=7`」；「`ptmj_ebook`/`ptmj_ebook_download`/`ptmj_ebook_favorite` 三张占位表**无任何运行时消费方**，不要启用」（`docs/pezmax-next-tasks.md:50,52`）。
- 当前代码事实：电子书是独立表 + 独立仓储，所有读写只落 `ptmj_ebook`——`FindByID` `modules/datum/ebook_repository.go:38`、`Insert` `:64`、`List` `:117-124`、`Subjects` `:170`；下载记录 `:193`、收藏 `:209`、举报 `:285`；表结构 `modules/datum/schema.go:199-266`。端点 `/datum/ebook/{list,content,subjects,favorite,report,upload}`（`datum_ebook.go:38-89`），上传只认 `ebookName/ebookSubject/ebookType`（`:384-417`）。
- 另有**专有下载端点** `GET /datum/download/ebook?ebookId=`，落 `ptmj_ebook_download`（`datum_activity.go:589-592`），与文档「下载不设专有端点」相反。
- 结论：**文档已作废，`ptmj_ebook` 单源成立**。`fileType=7` 只剩类型字典里的历史名字（`datum_file.go:48-51`）：普通上传若显式传 `fileType=7` 仍会写进 `ptmj_file`（`:773`），但**不会出现在电子书列表**——这是唯一残留二义性。代码中无把 `file_type=7` 迁到 `ptmj_ebook` 的迁移（`modules/datum/schema_migration_test.go` 只有书签收藏迁移）。

### 7.2 CDN / 限速 / QoS
- 文档说法：任务一「下载 100% 回源 kadmin，边缘 nginx 无任何缓存能力」（`docs/pezmax-next-tasks.md:10,22`）；任务三「全局令牌桶缺失，多连接可叠加突破总带宽」（`:83`）。
- 当前代码事实：
  - **应用层令牌桶已实现**：`KADMIN_QOS_UPLINK_MBPS` × `BULK_SHARE`（默认 10Mbps×80%），bulk 路径白名单 `/datum/download/`、`/api/files/`、`/api/uploads/`、`/uploads`（`qos.go:29-50,71-117,119-144`），装配开关 `docker-compose.yml:127-129`。
  - **边缘模板已重写并自述三层**：每连接弹性档位（`deploy/nginx.conf.template:51-58`）+ `limit_conn`（`:63-64,86-89`）+ 网卡 `tc` 全局预算（`:1-30`，env `KADMIN_EDGE_EGRESS_RATE/BURST_BYTES`）。
  - **仍无 CDN/缓存**：全模板 `proxy_cache`/`X-Accel`/`presign` 命中 0 次。
  - **仍无每用户维度、无热配置**：`qos.go:45-50` 只看路径不看 userID；档位仍是 env（`docker-compose.yml:206-209`），改档需重启 edge。
- 结论：任务三 **部分完成**（全局桶✅ / 每用户❌ / 热配置❌）；任务一 **仍缺**（无边缘缓存、无 presigned）。
- **客户端要知道的不止「串行下载」**：① 下载与头像/封面/上传物**共用同一个 bulk 桶**，浏览图片高峰期下载会被拖慢；② 实际速率 = min(nginx 档位, 桶余量)，不能按线路带宽做 UI 预估；③ 取消必须客户端侧实现，否则占着全局额度；④ 若上游将来采方案 B（presigned 直连），下载鉴权模型整体改变，客户端应预留「直连 URL」抽象。

### 7.3 Redis 无降级 → 排行/树 503
- 文档说法：「`stateStore.load` 对非 miss 的 Redis 错误直接上抛，Redis 不可用时排行直接 503」，修复方案是降级直查（`docs/pezmax-next-tasks.md:11,134,143`）。
- 当前代码事实：**仍是原样**。`load` 在 `cached()` 返回非 nil 错误时直接上抛（`datum_file.go:83-96`；`cached` 对非 `errRedisNil` 的错误返回错误 `:98-110`）；树 `:357-364`、排行 `:634-639` 各自 `fail(503)`。
- 新增了**预热/维护器**（启动同步预热 + 30s 周期，`datum_state_warmup.go:19-51`），但预热同样经过 `cached()`，**Redis 故障时照样失败**，不构成降级。
- 结论：**仍缺**。客户端必须自带降级（§3.1）。

### 7.4 下载链路与记录语义
- 文档说法：「主进程返回 `savedPath`、渲染层读 `filePath`，`localPath` 恒为空」（`docs/pezmax-next-tasks.md:217`）；「Range 恢复各记一条下载记录」。
- 当前代码事实：
  - **`savedPath` bug 已完成**：主进程返回 `{ success: true, filePath }`（`src/main/index.js:735`），渲染层读 `result.filePath`（`home/index.vue:408`、`views/datum/download/index.vue:237`、`views/datum/favorite/index.vue:340,398`）。**不要移植这个兼容分支。**
  - 端点实际行为：会话鉴权 → 仅已过审或属主自己的文件（`datum_activity.go:532-534`）→ **先写记录再出流**（`:549-552`，写失败只记日志不阻断）→ `attachment` + 强 ETag `"datum-<fileId>-<size>"`（`:560-566`）→ `ServeContent` 给 Range/206/416（`:568-574`）。
  - 记录语义：`Create` 是**纯 INSERT、无去重**（`modules/datum/activity_repository.go:19-23`），重复下载与每次 Range 续传都新增一行；「我的下载」按 `file_id` GROUP BY 去重、时间取 `MIN(creat_time)`、排序取 `MAX(download_id)`（`:52-65`）→ **列表不重复，下载次数统计偏大**（`CountByUser` 数行 `:78-79`）。
  - 电子书下载另表：`ptmj_ebook_download`（`modules/datum/ebook_repository.go:193`）。
- 结论：字段 bug **已完成**；记录口径 **部分完成**。

### 7.5 上传白名单 / 转档 / 文本编码
- 文档说法：白名单已含 `pdf/epub/mobi/azw3`（引用行 `datum_file.go:509-513`，现已移位）；转档走 LibreOffice。
- 当前代码事实：白名单 14 种 `doc/docx/pdf/jpg/jpeg/png/webp/txt/md/ppt/pptx/epub/mobi/azw3`（`datum_file.go:667-671`）；默认 200MB、`KADMIN_DATUM_UPLOAD_MAX_MB` 可调至 4096（`:40-43,725-728,734-737`）；转档仅 `doc/docx/ppt/pptx`（`:53,807`），soffice 不可用 → **503**、文件损坏 → **400**（`:812-820`），默认 2 分钟超时 / 2 并发（`platform/convert/convert.go:34-47`）；txt/md 强制 UTF-8（`datum_file.go:790-799`）；入库 `fileStatus=0` 待审（`:855`）。
- 电子书上传播音器是**另一条**端点：白名单更窄（`datum_ebook.go:355,414-417`）、必填 `ebookSubject` 与 `ebookType 1-4`（`:402-411`）、**不转档**（`:437-441`）。
- 结论：**与文档一致，且比文档更严**。客户端注意字段名不同（`fileName/fileType` vs `ebookName/ebookType`）。

### 7.6 「文档说要做 / 说没做，代码里已经能用」——可直接排进 M1/M2
| 能力 | 文档说法 | 当前代码事实 | 结论 |
| --- | --- | --- | --- |
| 电子书全链路 | 桌面端仅占位，待开发（`next-tasks:44-61`） | 后端 `datum_ebook.go:38-173`；桌面 `api/datum/ebook.js` 全量 121 行 | **已完成** |
| 批量过审 + 审核工作台端点 | 批量过审延后、审核页待建（`remaining-tasks:28-30,104`） | `/api/exam-file-audit/{pending,reported,approve,approve-user,reject,audit}` + 权限码（`exam_file_audit.go:19-40`） | **已完成**（管理端） |
| 电子书管理端 | 占位表勿用（`next-tasks:52`） | generated 新增 `ebook/ebook_download/ebook_favorite/ebook_report` 四模块（`k_admin/internal/kadmin/generated/`） | **已完成** |
| 封禁即时失效 | 存量会话可用至 TTL（旧 #19） | `TestDatumDisabledUserRejectsExistingSessions`（`datum_session_status_test.go:9-58`）→ 401（`datum_user_auth.go:462-473`） | **已完成** |
| 年份区间筛选 | AI 需要 `yearFrom/yearTo`（`next-tasks:97`） | `datum_search.go:13-31`，列表 SQL 前置过滤（`datum_file.go:319-336`） | **已完成** |
| 平台知识库端点 | 旧清单未列 | `/datum/knowledge/{manifest,source,content}`（`datum_knowledge.go:17-24`） | **已完成**，见 §4 |
| 下载字段 bug | P0，`localPath` 恒空 | `src/main/index.js:735` + 三处读 `filePath` | **已完成** |
| `code===200` 硬判定 | 8 处遗漏（`next-tasks:218-222`） | 渲染层 grep `code === 200` 零命中 | **已完成** |
| `/common/upload` 遗留 | 三组件指向未实现端点（旧 #24） | 渲染层 grep `common/upload` 零命中 | **已完成** |

### 7.7 仍然缺（客户端要等、要自己做、或不要承诺）
| 条目 | 证据 |
| --- | --- |
| Redis 降级（排行/树 503） | `datum_file.go:83-96,357-364,634-639` |
| 边缘缓存 / presigned | `deploy/nginx.conf.template` 无 `proxy_cache`（grep 0 命中） |
| QoS 每用户维度 / 热配置 | `qos.go:45-50`（仅路径）；`docker-compose.yml:206-209`（env） |
| 下载取消（IPC） | `src/main/index.js` 仅上传有 `AbortController:965-987` |
| 可中断加载 / 请求取消池 | `GlobalLoader.vue:47-61` 全屏吞点击；`home/index.vue:951-957` 串行 + 400ms |
| 多文件下载进度 | `home/index.vue:1333-1335` 单个 `downloadPercent` 被覆盖 |
| 统一用户详情 + TA 公开文件列表 | 仅 `views/rank/index.vue:291-314` 内嵌；无 `/desktop/user/{id}/files` |
| 公开用户投影收敛（去 `remark/status`） | `datum_user_admin.go:26-40` |
| URL 映射工具（任务四） | 内网 host 原样透传：`datum_file.go:462-484` |
| `normalizeFileUrl` 弃用 | `src/renderer/utils/url.js:15-49`，20+ 处引用仍在 |
| 管理端 `ptmj_file_favorite` CRUD（旧 #18） | `generated/` 下无 `file_favorite` |
| `electronAPI.saveFile` 死代码（任务八.4） | `src/preload/index.js:44` 仍在 |
| 拼写重复组件（任务八.3） | `ReporTimeLinePanel.vue` 仍在且无引用 |

### 7.8 已作废（不要再排）
- `.env.production` 换域名（`next-tasks:226`）：文件已不存在；但 `utils/url.js:18` 硬编码 `http://127.0.0.1:9033` 仍需清理。
- 「电子书 = `fileType=7`、勿启用 ebook 表」（`next-tasks:50-52`）：**完全反了**，以 §7.1 为准。
- 「电子书下载走 `/datum/download/file`」（`next-tasks:50`）：已改专有端点 `/datum/download/ebook`。

**给排期的一句话**：两份任务文档当「意图清单」用；凡涉及现状的判断先查 §7。§7.6 的 9 项可直接进 M1/M2，§7.7 的 13 项不要写进对外承诺。
