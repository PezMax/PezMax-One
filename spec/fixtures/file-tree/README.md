# `fixtures/file-tree/` —— 树响应 → 导航模型

> 服务端行为依据 **PezMax-Go@061a888**：`k_admin/internal/kadmin/datum_file.go:366-427`（构树）、
> `k_admin/internal/kadmin/modules/datum/file_repository.go` 的 `ListApprovedFlat`（喂树的数据与顺序）、
> `datum_file.go:429-434`（类型名兜底）。

树是**一次给全量**的（所有已审核文件），不是懒加载。所以客户端要做的是「把一棵大 JSON 变成能滚的导航」，
而不是「按层请求子节点」。这份夹具写的就是这层映射。

## 服务端已经定好的（照抄，不要重新推导）

| 事实 | 依据 |
|---|---|
| 层级：`科目 → 学校 → 类型 → 年份`，后面接 `remark` 里按 `/` 拆出的自定义目录段 | `datum_file.go:394-399` |
| 空科目 → `未知科目`；空学校 → `未知学校`；`fileYear ≤ 0` → **服务端当前年** | `datum_file.go:383-393` |
| 类型名来自字典，未知值 → `其他` | `datum_file.go:429-434` |
| 文件夹 `id` = `"folder-" + hash(JSON(该节点的标签路径))`，**只与标签路径有关**，与文件无关 | `datum_file.go:401-412` |
| 叶子 `id` 被覆盖成 `"file-<fileId>"`，`fileId` 仍是数字 | `datum_file.go:421-424` |
| 喂树的数据顺序：`file_subject, file_school, file_type, file_year DESC, file_id` | `file_repository.go` 的 `ListApprovedFlat` |
| 叶子是**完整 file payload** + `children: []` + `type: "file"` | `datum_file.go:436-460` |

两条推论用在后面的规则里：

1. `ORDER BY` **不含 `remark`**，所以带自定义目录的文件会和同层文件交错——同一父节点下出现
   「叶子、文件夹、叶子」这种顺序是正常的。
2. 年份是 `DESC`，所以子节点顺序**天然是新年份在前**。客户端**不要**按标签字母/数字重排，
   一重排就把「新的在前」弄丢了（`2019` 会排到 `2025` 前面）。

## 客户端要判定的四件事

| kind | 输入 | 输出 |
|---|---|---|
| `rows` | 树 + 当前文件夹的标签路径 | 该文件夹下要渲染的行：文件夹在前、文件在后，各自保持服务端顺序 |
| `leaf` | 一个叶子节点 | 行模型：`fileId`（数字）、`title`、`kind=file` |
| `breadcrumb` | 树 + 目标节点的标签路径 | 面包屑标签序列（含根「全部」） |
| `fallback` | 一个节点 | 展示用标签：服务端给什么显什么，客户端不再兜底 |

### 为什么要「文件夹在前」

`ORDER BY` 不含 `remark`，子目录的位置取决于它第一个文件在结果集里的位置，可能夹在兄弟叶子之间。
导航树里叶子混排会让「往下钻」这件事变得没有落点，所以统一：**先文件夹后文件，各自保持原序**。

### 为什么 `fileId` 不能从 `id` 解析

叶子的 `id` 是 `"file-12"` 这种字符串，**列表 / 详情 / 收藏接口里的 `id` 却是数字 12**。
从字符串里 `split("-")` 取数字，遇到哪天上游改前缀就全崩。永远读 `fileId`。

### `folder-…` 不能持久化

文件夹 id 是**标签路径**的哈希：改一次科目名、学校名，整条路径的 id 全变。
所以它只能当「树内唯一键」（`TreeView` 的 `x:Key`），**不要**存进设置 / 收藏 / 下载记录——
那些地方要存路径标签或 `fileId`。

## 形状

```json
{
  "id": "file-tree.rows.folders-before-files",
  "description": "同一父节点下叶子与子目录交错时，渲染成「文件夹在前、文件在后」",
  "source": "PezMax-Go@061a888 k_admin/internal/kadmin/modules/datum/file_repository.go（ORDER BY 不含 remark）",
  "input": {
    "kind": "rows",
    "path": ["数学"],
    "tree": [
      {
        "id": "folder-a", "label": "数学", "type": "folder",
        "children": [
          { "id": "folder-b", "label": "实验中学", "type": "folder",
            "children": [
              { "id": "file-1", "fileId": 1, "label": "期中.pdf", "type": "file", "children": [] },
              { "id": "folder-c", "label": "期末", "type": "folder", "children": [] }
            ] }
        ]
      }
    ]
  },
  "expect": { "order": ["folder:期末", "file:期中.pdf"] }
}
```

- 树节点统一用 `{id, label, type, children}`；叶子另外带 `fileId` 与 file payload 字段（夹具只写判定需要的）。
- `path` 是**标签路径**，不是 id 路径——用 id 会让用例在 id 规则变化时全部失效。
- `expect.order` 里 `folder:<label>` / `file:<label>` 表示行的种类与标题。

## 不覆盖的

- 树的渲染细节（`TreeView` 虚拟化、展开状态持久化）：属于各端 UI，不进夹具。
- 上传后「本地乐观插入」：等下载 / 上传实现时另开夹具，别在这里提前规定。
- 树的 `hash` / `unchanged` 协商：见 `fixtures/state-cache/`。
