# `fixtures/epub/` —— EPUB 包解析

> 依据：EPUB 3.2（container.xml §3.3、OPF §3.4、navigation document §5.1）与 EPUB 2.0.1 的 NCX。
> 上游 `PezMax-Go` 不参与：这是客户端把一本书拆成「元信息 + 章节顺序 + 目录」的规则。

三端都用 WebView 系渲染 EPUB，但**打开一本书要先看懂包结构**：`META-INF/container.xml` 指向 OPF，
OPF 里有元信息、manifest、spine（章节顺序），目录在 EPUB 3 的 nav.xhtml 或 EPUB 2 的 NCX 里。
这段和平台无关，抽成夹具。

## 形状

```json
{
  "id": "epub.package.epub3-nav",
  "description": "…",
  "source": "EPUB 3.2 §3.3 container.xml、§3.4.6 spine、§5.1 navigation document",
  "input": {
    "kind": "package",
    "files": { "META-INF/container.xml": "…", "OEBPS/content.opf": "…", "OEBPS/nav.xhtml": "…" }
  },
  "expect": {
    "title": "高等数学（上册）",
    "author": "同济大学数学系",
    "rootDirectory": "OEBPS",
    "chapters": [ { "href": "ch1.xhtml", "path": "OEBPS/ch1.xhtml", "title": "第一章 函数" } ]
  }
}
```

- `files` 是「zip 条目 → 文本内容」：只给 XML / XHTML（解析只读这些），图片与字体不必给。
- `chapters` 按 spine 顺序；`href` 是 manifest 里写的**原始串**，`path` 是解出来的 zip 内路径。
- `expect.rootDirectory` 是 OPF 所在目录（拼虚拟主机 URL 时要用）。
- `expect.failure` 出现时表示这次解析要**失败**：值是失败原因里必须包含的子串，此时 `chapters` 为空。

## 规则与理由

**container.xml 可以缺**。手工拼的包、某些转换工具产出的包就没有它。缺了不能整本打不开：
按名字挑第一个 `.opf`（序数比较，保证可重复）。`container.xml` 存在但 XML 坏了，同样走这条兜底。

**href 一律按 URI 处理**：去掉 `#fragment` 与 `?query`、**解码百分号**（`chapter%201.xhtml` →
`chapter 1.xhtml`，否则 zip 里明明有文件却报章节不存在），再按基准目录消掉 `./` 与 `../`。
基准目录是**该文件自身所在目录**：manifest 的 href 相对 OPF，nav / NCX 里的链接相对 nav / NCX ——
目录文件不在 OPF 同目录时，这两者会不同。

**章节顺序只看 spine**，不看目录。目录只是标题来源：
正文里有附录、目录没更新是常态，按目录建章节会直接丢内容（`chapter-missing-from-toc`）。
目录里出现两次的章以**第一次**为准（卷首卷末各列一遍很常见）。

**spine 里混进非 XHTML 的要跳过**：有的包把封面图或 CSS 塞进 spine，当成章节打开只会得到一张图。
只收 `application/xhtml+xml` 与 `text/html`。

**目录两种来源**：先找 manifest 里 `properties` 含 `nav` 的项，在里面找 `epub:type="toc"` 的 `<nav>`
（没有就取第一个 `<nav>`），按文档顺序拍平嵌套的 `<ol>`；找不到再退回 `spine@toc` 指向的 NCX
（或 manifest 里 `application/x-dtbncx+xml` 那一项）的 `navMap/navPoint`。
标题里的换行与连续空白压成一个空格——那多半是排版用的缩进，不是标题的一部分。

**失败不抛异常**：`Parse` 返回带 `Failure` 的空书。调用方要能区分「书坏了」和「书是空的」，
并且**不要**把半本书当完整的渲染（附录丢掉比打不开更糟）。

## 不覆盖的

- **解压**：zip 怎么读、往哪落盘，各端自己的事（Windows 落 `LocalCache/epub/{ebookId}/`）。
- **加密 / DRM**：`META-INF/encryption.xml` 不处理，含 DRM 的书直接当解析失败。
- **CSS / 图片 / 字体**：解析层不碰，由渲染层经虚拟主机原样提供。
- **分页与排版**：交给 WebView 一侧，见各端阅读器文档。
