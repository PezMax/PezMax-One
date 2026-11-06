# `fixtures/url/` —— 文件 URL 归一（过渡规则）

> **权威依据是 PezMax-Go 的 `platform/fileurl`**。`PezMax-Desktop/src/renderer/utils/url.js`
> 是**旧接口时代**客户端的做法，只作参考；两者冲突时以 Go 源码为准。

上游写入库的应当是**可直接访问的公网地址**：`KADMIN_MINIO_PUBLIC_BASE/<bucket>/<objectKey>`
（`k_admin/internal/kadmin/platform/fileurl/fileurl.go:1-13`、`68-75`）。历史行里还留着内网 host、
`minio://` 这些形态，上游把「批量改库」交给一次性 ETL，而不是运行时改写（`fileurl.go:8-12`）。

客户端因此要有一层过渡归一——**只改能改的**，改不了的让用户看到真实错误。

## 规则（自上而下，先命中先返回）

| # | 输入形态 | 处理 | 依据 |
|---|---|---|---|
| 1 | 空串 / 纯空白 | 返回空串，调用方按「无地址」处理；**不要**拼成站点根 | `fileurl.go:100-102` 对空值同样拒绝 |
| 2 | `data:` / `blob:` 开头 | 原样返回（内联图、本端生成的临时对象） | `fileurl.go:89-97`：服务端对象地址只有 `minio://` 与 `http(s)` |
| 3 | `minio://bucket/key` | 换成 `BaseUrl + /bucket/key` | `fileurl.go:109-114` |
| 4 | 相对路径（不以 `http://` / `https://` 开头） | 补 `BaseUrl`，缺前导 `/` 时补一个 | `fileurl.go:23-26`：相对路径没有 bucket，服务端取流会 404；封面 / 头像等静态文件仍要能显示，所以按站点根拼 |
| 5 | 绝对地址且 host 是内网（`localhost`、`127.0.0.1`、`minio`、`server`、`172.*`） | **保留 path 与 query，换成当前 `BaseUrl` 的 scheme + host + port** | `fileurl.go:4-12`：存量行可能带内网 host，由 ETL 修 |
| 6 | 其它绝对地址（公网 host、CDN、外链） | 原样返回，一个字符都不动 | `fileurl.go:1-13`：写入即公网地址，无需客户端插手 |

## 与旧客户端有意不同的两处

1. **内网改写换掉整个 origin（scheme + host + port）**。旧端只替换 hostname、端口沿用原来的
   （`PezMax-Desktop/src/renderer/utils/url.js:39-42`），于是 `http://minio:9000/x` 会变成
   `http://<目标机>:9000/x`——目标机上没有 9000。我们按「这台机器的地址就是设置里那个 BaseUrl」一次换干净。
2. **不剥 `/dev-api` / `/prod-api` / `/stage-api` 前缀**（旧端 `url.js:23-24`）。那是旧端开发期走 Vite 代理的残留；
   原生客户端直连站点根，不做这层猜测。真出现这种行说明库里脏了，该在上游修。

## 撤除条件

上游 ETL 完成、`ptmj_file` / `ptmj_ebook` / 书签封面里不再有内网 host 与 `minio://` 之后，
规则 3、5 整段删除；规则 1、2、4、6 保留（属于正常的 URL 拼装，不是兼容层）。

## 形状

```json
{
  "id": "url.internal.localhost-with-port",
  "description": "内网 localhost 带端口：换成 BaseUrl 的 origin，端口不保留",
  "source": "PezMax-Go@061a888 k_admin/internal/kadmin/platform/fileurl/fileurl.go:4-12",
  "input": { "url": "http://localhost:9000/kadmin/a.pdf", "baseUrl": "https://pz.example.com:8443" },
  "expect": { "url": "https://pz.example.com:8443/kadmin/a.pdf" }
}
```

- `input.baseUrl` 是用户在设置里填的站点根地址（可能带路径前缀与末尾斜杠）。
- `expect.url` 必须**逐字符相等**；归一规则里没有「大致正确」。

## 注意：预览与下载不走这条规则

`GET /datum/download/file`、`/datum/download/ebook`、`/datum/ebook/content`、`/datum/file/content`
都是**带 Bearer 的流式端点**，走 `Range` / `ETag`（见 `fixtures/download/`）。`fileUrl` 只用于
「直连对象存储」这种少数场景，别拿它当预览地址。

## 不覆盖的

- 路径里的中文 / 空格百分号编码：上游 `ObjectURL` 已按段转义（`fileurl.go:151-156`），客户端不再编码，以免双重转义。
- 需要鉴权的流式预览：见上一节。
