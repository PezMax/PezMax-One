# 契约核对记录

这份文件记的是**事实**：哪些端点已经用客户端自己的协议代码跑通过一遍。契约仍以
`endpoints.md` / `envelope.md` / `auth.md` 为准；这里只回答「这条路径有没有被真的走过」。

## 为什么要单独记

- 三端各写各的实现，同一份 spec 会各自读出不同的理解。`fixtures/` 锁的是**判定**，
  锁不住「请求发得对不对、响应解得对不对」，那一段只有真跑才知道。
- Windows 端的界面走查要过 AppContainer 的回环豁免（需管理员），不能当作日常核对手段。
- 所以核对走 `windows/tools/apiwalk/`：net9.0 控制台，直接引用 `PezMax.Core`，
  用 `AuthApi` / `FilesApi` / `DatumHttp` 自己的代码发请求，对端是
  `spec/tools/devserver/`（按本目录形状做的最小实现，与上游有已知差别，见其 README）。

## 核对过（2026-11-10，devserver@74ee766，客户端 `PezMax.Core`）

| # | 端点 | 核对项 | 结果 |
|---|---|---|---|
| 1 | `GET /datum/user/captchaImage` | `captchaEnabled` / `uuid` / `img` 为合法 base64 | ✅ 4.6 KB base64，解出来是 JPEG |
| 2 | `POST /datum/user/register` | 信封 `code: 0`；`data.userId` 为数字 | ✅ `userId = 8` |
| 3 | `POST /datum/user/login` | 拿到 `token`，`expiresAt` 可解析 | ✅ |
| 4 | `GET /datum/user/getInfo` | `data.user.userId` 与注册返回的 id **一致** | ✅ 断言相等 |
| 5 | `GET /datum/user/securityQuestions` | 三个问题原样返回，顺序不变 | ✅ |
| 6 | `POST /datum/user/resetPasswordBySecurity` | 答案错 → `code: 400` 且文案为「密保答案不正确」 | ✅ |
| 7 | 同上 | 答案对 → 成功信封 | ✅ |
| 8 | `POST /datum/user/login` | 旧密码被拒（`code: 400`，用户名或密码错误） | ✅ 说明密码真的换了 |
| 9 | 同上 | 新密码可登录，会话可用 | ✅ |
| 10 | `GET /datum/file/list` | 分页形状：`total` / `rows` / `rows[].fileName` | ✅ 5 份公开资料 |
| 11 | `GET /datum/file/tree` | 学科层级与 `label` | ✅ 5 个学科 |
| 12 | `GET /datum/file/list?userId=` | 带 `userId` 才看得见自己的（含待审） | ✅ 0 份 |
| 13 | `GET /datum/download/file` | 流式下载、`%PDF` 头、`ETag` 为 `"datum-<id>-<size>"` | ✅ 1557 字节，`"datum-101-1557"` |

断言失败即 `exit 1`：这条链是拿来核对契约的，「看起来跑完了」没有意义。

## 由核对发现的偏差（都已按 spec 修对端）

| 偏差 | 表现 | 结论 |
|---|---|---|
| 成功信封写成 `code: 200` | 客户端每步都报「ok」，因为 `msg` 是空的、`code != 0` 判为业务错误 | 对端错。`envelope.md` 早就写定成功是 `code: 0` |
| 注册成功返回裸数字 `8` | 客户端 `data.userId` 读成 0 | 对端错。`auth.md`：「成功：`data = { "userId": 7 }`」 |
| 用户名重复返 400 | 客户端按 `400` 走通用错误文案 | 对端错。`auth.md` 写明 409 |

这三条都是**客户端本来就对**、对端（本地镜像）写偏的例子，记在这里是为了说明：
核对的价值在「谁对」而不是「能不能跑」。

## UWP 界面走查（2026-11-07，同一天，同一个 devserver）

上面那张表是控制台跑的。这一节是在**打包后的 UWP 客户端**里真点了一遍：
先给包加环回豁免（一行命令，见 `docs/ENVIRONMENT.md`），然后从注册一路点到翻资料。

| 步骤 | 结果 |
|---|---|
| 注册（三步：账号 → 密保 → 验证码） | ✅ 「账号 林知远 已创建（编号 9）」——注册返回 `userId` 那条，界面上也看得见 |
| 找回密码：三问答错 | ✅ 退回第一步，提示「密保答案不正确（请重新获取验证码后再试）」 |
| 找回密码：三问答对 | ✅ 「密码已重置，请用新密码登录」 |
| 用旧密码登录 | ✅ 被拒：用户名或密码错误 |
| 用新密码登录 | ✅ 进主界面，昵称是新账号的 |
| 首页 | ✅ 5 份资料；指标卡与 `/profile/stats` 一致 |
| 资料 → 树（化学 → 第二中学 → 神秘文件 → 2024） | ✅ 每下钻一级都带上同一串条件：`fileSubject` / `fileSchool` / `fileType` / `fileYear` |
| 列表里点一行 → 详情面板 | ✅ 标题、信息行、状态行、五个操作按钮都在 |

### 界面走查查出来的六件事

| 缺陷 | 表现 | 修法 |
|---|---|---|
| 点树节点没反应 | 右侧列表一直空，**一个请求都不发** | `ItemInvoked` 给的是 `TreeViewNode` 而不是节点里的 `TreeEntry`；`BuildQuery` 又在判 `Content is FileTreeNode`（实际是 `TreeEntry`），一层都取不到。两处都修 |
| devserver 的树少一层 | 年份顶在类型的位置上 | 补成 科目 → 学校 → 类型 → 年份 |
| `/system/notification/user/scroll` 404 | 界面看不出来，日志里一直在报 | devserver 补上这条（空列表） |
| `/datum/favorite/{fileId}` 404 | 详情面板一打开就 404；客户端兜底成「未收藏」，真收藏过的人会看到错文案 | devserver 按 spec 回 `data{favorited}` |
| `count` 写死 3 | 首页「已上传 3 份资料」与指标卡 0 打架 | 按会话实际拥有的未删件数算 |
| 详情栏被压扁 | 1000 级窗口下标题逐字换行、五个按钮把「复制链接」挤出框外 | 三栏改 220 / * / 260；按钮两列三行；详情栏关横滚（允许横滚时内容按无限宽测量，`TextWrapping` 失效） |

第一条是这一类夹具**测不出来**的：判定表全对，只是事件没接上。
这也是把界面走查和 `fixtures/` 并列记在这儿的原因——一个管判定，一个管接线。

## 未核对

写接口与带状态的路径都还没走过，别当成已验证：

| 路径 | 缺口 |
|---|---|
| `POST /datum/file`（multipart 上传） | 只验过形状，没走过真实文件与转档 |
| `PUT /datum/file`（整组覆盖） | 同上；`remark` 丢失的坑只有真跑才看得见 |
| `DELETE /datum/file/{fileId}` | 未走 |
| `GET /datum/download/ebook`、`/datum/ebook/content` | 未走 |
| `Range` / `If-Range` 续传（206 / 416） | `DownloadPlanner` 的判定有夹具，**字节层未验**——属 W1 |
| 通知 / 收藏 / 我的下载 | devserver 恒为空列表，只有形状 |
| 401 会话失效 | devserver 里 token 固定、不失效 |

## 怎么重跑

```powershell
# 1. 起本地镜像（默认 127.0.0.1:9033，这里换个端口免得和别的东西撞）
cd spec\tools\devserver
go run . -addr 127.0.0.1:8080

# 2. 另一个窗口：用客户端代码走一遍
dotnet run --project windows\tools\apiwalk -- http://127.0.0.1:8080
```

输出末尾是「全部走通。」，退出码 0。中途任一断言不成立会打印 `✗` 并以 1 退出。
新增核对项就改 `apiwalk/Program.cs`，**同时**更新本文件的表格与日期。
