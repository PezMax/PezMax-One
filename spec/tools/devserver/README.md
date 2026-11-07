# `devserver/` —— 本地开发服务器

按 `spec/api/endpoints.md` 的形状提供 `/datum/*` 的一份**最小实现**，用来在没有测试服务器、
也没装 Redis 的机器上把客户端跑起来。零依赖：只用 Go 标准库。

```powershell
cd spec\tools\devserver
go run . -addr 127.0.0.1:9033
# 客户端「设置 → 高级 → 服务器地址」填 http://127.0.0.1:9033
# 登录：任意用户名 + 任意密码 + 任意验证码（token 固定为 pezmax-dev-token）
```

参数：

| 参数 | 默认 | 说明 |
|---|---|---|
| `-addr` | `127.0.0.1:9033` | 只绑本机。**不要**改成 `0.0.0.0`：它没有任何鉴权 |
| `-token` | `pezmax-dev-token` | 会话 token；登录接口会把它发回客户端 |
| `-ref` | 空 | 未实现的路径转发到这个真后端，例如 `http://127.0.0.1:8080`，用来「本地 + 真后端」混着调 |

## 它是什么、不是什么

**是**：一份能跑的形状对照表。种子里有 6 份资料（含 1 份待审）、3 本电子书、3 条排行，
目录树按真实后端的层级（科目 / 学校 / 年份 / 文件）拼出来，`hash` + `unchanged` 语义也在。

**不是**：后端。数据全在内存、重启即空、没有鉴权、没有并发写保护以外的任何防线。

- 契约的权威永远是 `spec/api/endpoints.md`：**这里和 spec 不一致时改这里**，不是改 spec。
- 它不进客户端代码，也不参与发布。

## 覆盖到的端点

| 组 | 端点 | 说明 |
|---|---|---|
| 账户 | `captchaImage`、`login`、`logout`、`getInfo`、`securityQuestions` | 验证码返回 `captchaEnabled:false`；登录不校验密码 |
| 资料 | `tree`（含 `hash` / `unchanged`）、`list`（含 `userId` 的「我的上传」与 403 闸门）、`search`、`subjects`、`schools`、`schools/check`、`content`（txt/md）、`{fileId}` 详情 | 待审件只有属主可见（否则 404），和上游一致 |
| 资料写 | `POST /datum/file`（multipart）、`PUT`（**整组覆盖**）、`DELETE`（软删） | 上传进「待审」，编辑不改变状态，删除后不再出现 |
| 下载 | `/datum/download/file`、`/datum/download/ebook` | 走 `http.ServeContent`：`Range` → 206、`ETag` 与上游同格式（`"datum-<id>-<size>"`） |
| 电子书 | `list`、`subjects`、`content` | 电子书 201 是 PDF，202/203 是现场生成的 EPUB（三章 + nav 目录） |
| 排行 | `/datum/user/rank` | 3 条固定数据 |
| 空列表 | 通知、我的下载、收藏、举报、收藏状态 | 形状对、内容空：这些页面的空态能验证，内容验证要等真后端 |

## 现场生成的两个文件

这两个是「让 W2 / W3 有东西可测」而写的，都只有一个最小合法实现，不是完整排版：

- **PDF**：`/datum/download/file?fileId=101&pages=120` 现写一个 N 页的 PDF（默认 4 页）。
  用来验「100 页以上试卷」的首屏时间、连续滚动与内存——W2。
- **EPUB**：`/datum/ebook/content?ebookId=202` 现写一个三章 EPUB（`mimetype` + container + OPF + nav + 三章 XHTML）。
  用来验解压、Core 的 `EpubPackage`、虚拟主机映射与中文排版——W3。

## 已知差别（别拿它当上游行为）

| 差别 | 原因 |
|---|---|
| 没有鉴权，token 固定 | 本地调试要的是「能进」，不是安全 |
| 通知 / 收藏 / 我的下载恒为空 | 还没做界面验证；接了真实后端才有意义 |
| 上传不落盘、不转档 | 转档要 LibreOffice；这里只验客户端流程与 `remark` 目录 |
| 分页 `total` 用整表条数 | 上游「我的下载」的 `total` 与 `rows` 口径不一致是特例，这里不模拟 |
