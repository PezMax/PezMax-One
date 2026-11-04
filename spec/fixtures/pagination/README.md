# fixtures/pagination

分页参数归一与「按 total 拉全量」的跨端判据。信封与列表形状见 `spec/api/envelope.md` §6。

## 参数规则

| 规则 | 说明 |
|---|---|
| 页码 | 先看 `pageNum`，再看 `page`；第一个非空值生效 |
| 页大小 | 先看 `pageSize`，再看 `size`；第一个非空值生效 |
| 缺省 | `pageNum=1`、`pageSize=20` |
| 非法值 | 非整数或 ≤ 0 时保持缺省值，不报错 |
| 上限 | `pageSize` 最大 100，超出按 100 处理 |
| 空页 | 页码超出总页数时 `rows=[]`、`total` 不变 |

来源：`k_admin/internal/kadmin/datum_file.go:268-295`。

`total` 缺失时按本页 `rows.length` 兜底（防御性规则，取自 Electron `fetchAllPages`）。

## 全量拉取算法

```text
fetchAllPages(fetcher, baseQuery, { pageSize = 100, maxPages = 100 }):
  first = fetcher({...baseQuery, pageNum: 1, pageSize})
  rows = first.rows ?? []
  total = number(first.total) || rows.length
  for pageNum = 2..maxPages while rows.length < total:
      part = fetcher({...baseQuery, pageNum, pageSize}).rows ?? []
      if part.length == 0: break
      rows.push(...part)
  return { rows, total }
```

- `pageSize` 默认 100（服务端上限），不要传 1000。
- `maxPages` 是死循环保险；达到上限就停，返回已拿到的部分。
- 不做去重；服务端在翻页期间新增 / 删除数据时结果可能重复或漏项，调用方需要按业务键自行处理。
- 请求失败时**不要**返回半份列表冒充成功；把错误向上抛。

## 用例形状

### 参数归一

```jsonc
{
  "id": "pagination.defaults.missing",
  "description": "…",
  "source": "PezMax-Go@f79a4e0 …:行号",
  "input": { "query": { "pageNum": "2", "pageSize": "50" } },
  "expect": { "page": 2, "pageSize": 50 }
}
```

`input.query` 的键值都是**字符串**，模拟 URL 查询参数。

### 列表响应

在参数之外带 `input.response`（与 `fixtures/envelope` 的 `input` 同形），`expect` 增加：

```jsonc
{
  "kind": "list",
  "rows": [],
  "total": 42,
  "error": false
}
```

### 全量拉取

```jsonc
{
  "input": {
    "baseQuery": {},
    "pageSize": 100,
    "maxPages": 100,
    "pages": [
      { "pageNum": 1, "rows": 100, "total": 230 },
      { "pageNum": 2, "rows": 100, "total": 230 },
      { "pageNum": 3, "rows": 30, "total": 230 }
    ]
  },
  "expect": {
    "requestedPages": [1, 2, 3],
    "rowCount": 230,
    "total": 230,
    "stopReason": "reached-total"
  }
}
```

`pages[].rows` 是数字，表示该页返回的条数；夹具不关心行内容。
`stopReason` 取值：`reached-total`、`empty-page`、`max-pages`。

## 不覆盖的范围

- 各端 HTTP 层如何拼查询串（编码、重复键）不在夹具内。
- 服务端在翻页期间数据变化导致的不一致，属于业务容忍范围，不由夹具断言。
