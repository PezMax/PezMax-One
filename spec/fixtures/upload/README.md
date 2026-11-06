# `fixtures/upload/` —— 上传前的本地校验

> 服务端规则依据 **PezMax-Go@061a888**：`k_admin/internal/kadmin/datum_file.go:667-671`（白名单）、
> `:723-825`（上传校验与转档）、`k_admin/internal/kadmin/datum_ebook.go:354-417`（电子书上传）。

客户端在**发请求之前**要先判一遍，理由和账户表单一样：省一次往返、把错误落到具体那个文件上。
代价是文案与阈值必须跟服务端逐字一致——否则会出现「客户端放行、服务端 400」的挫败感。

## 两套规则不要混

| | 资料上传 `POST /datum/file` | 电子书上传 `POST /datum/ebook/upload` |
|---|---|---|
| 扩展名 | `doc docx pdf jpg jpeg png webp txt md ppt pptx epub mobi azw3`（14 种） | `pdf epub mobi azw3` |
| 转档 | `doc/docx/ppt/pptx` → PDF | **不转档** |
| 名称字段 | `fileName`（≤255 **rune**） | `ebookName`（≤255 **rune**） |
| 学科字段 | `fileSubject`（≤64 **字节**） | `ebookSubject`（≤64 **字节**） |
| 分类字段 | `fileType` 1–7 | `ebookType` 1–4 |
| 其它必填 | `fileYear` 1900–2100 | —— |
| 其它可选 | `fileSchool`（≤100 **字节**）、`remark` 目录 | `author` / `publisher`（各 ≤255 **rune**） |

**字节 vs rune 是真会咬人的**：`fileSubject` 的 64 是 `len()`，一个汉字 3 字节，所以中文科目只有
**21 个字**可用；而 `fileName` 的 255 是 `len([]rune())`，255 个汉字也放得下。客户端两边必须分别计数，
统一按字符数算会放过中文科目、统一按字节算又会砍掉长文件名。

## 三个容易踩的点

1. **资料上传也收 `epub/mobi/azw3`**：这四种会写进 `ptmj_file`，**不会**出现在电子书列表里
   （电子书是独立表 `ptmj_ebook`）。客户端应当把这几种格式引导到电子书上传入口，
   而不是让用户在资料上传里传完发现「电子书里没有」。
2. **错误文案与实际白名单不一致**：不支持的格式回「仅支持 doc/docx/pdf/jpg/png/webp/txt/md/ppt/pptx」，
   文案里没提 `jpeg` 与三种电子书格式，但它们是允许的。客户端**不要**照抄这句文案去限制选择器。
3. **转档的 200MB 是硬上限**，与可配置的 `KADMIN_DATUM_UPLOAD_MAX_MB` 无关：
   只·有 `doc/docx/ppt/pptx` 超过 200MB 才回 **413**「转档文件不能超过 200MB」，
   其它格式超限回 **400**「文件过大：不能超过 {max}MB」。

## 形状

```json
{
  "id": "upload.file.docx-converts",
  "description": "docx 会转成 PDF 存储：本地就要提示「将转换为 PDF」，并允许更大的体积",
  "source": "PezMax-Go@061a888 k_admin/internal/kadmin/datum_file.go:807-825",
  "input": { "kind": "file", "fileName": "复习提纲.docx", "sizeBytes": 1048576, "maxUploadMb": 200 },
  "expect": { "accept": true, "convertsToPdf": true, "needsUtf8Check": false }
}
```

- `kind`：`file` / `meta` / `ebook` / `batch`。
- `expect`：`accept` 必有；`reason` 是**必须逐字相等**的文案；`convertsToPdf` / `needsUtf8Check`
  可选（只对 `file` 有意义）；`batch` 用 `results`（逐文件 `{name, accept}`）。
- `input.maxUploadMb` 缺省 200（服务端默认），用来覆盖「管理员调大了上限」的情况。

## 撤除条件

没有过渡性质。上游改白名单、改阈值、改文案时**先改这份夹具**，三端跟着改。

## 不覆盖的

- 真实转档耗时、soffice 排队：属于服务端行为，客户端只负责「长耗时提示 + 可重试」。
- 分片 / 断点续传上传：上游 `POST /datum/file` 是一次性 multipart，没有分片协议。
- 文件夹上传的目录枚举：由各端 `FolderPicker` 实现，枚举结果按本夹具逐个判。
