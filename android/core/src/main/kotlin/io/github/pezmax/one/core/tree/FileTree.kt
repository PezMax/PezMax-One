package io.github.pezmax.one.core.tree

import io.github.pezmax.one.core.json.asArrayOrNull
import io.github.pezmax.one.core.json.longOrNull
import io.github.pezmax.one.core.json.asObjectOrNull
import io.github.pezmax.one.core.json.stringOrNull
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject

/**
 * 树响应 → 导航模型（spec/fixtures/file-tree/）。
 *
 * 树是一次给全量的，客户端只做「把大 JSON 变成能滚的行」：
 * 文件夹在前、文件在后，各自保持服务端顺序；年份的 DESC 顺序不能重排。
 * 叶子的 `id` 是 "file-N"，`fileId` 才是数字，行模型只读 `fileId`。
 */
data class TreeNode(
    val id: String,
    val label: String,
    /** "folder" / "file"；上游只会有这两种。 */
    val type: String,
    val fileId: Long? = null,
    val children: List<TreeNode> = emptyList(),
    /** 叶子携带的完整 file payload；详情面板直接用它。 */
    val payload: JsonObject? = null,
) {
    val isFolder: Boolean get() = type == "folder"
}

enum class RowKind { Folder, File }

data class TreeRow(
    val kind: RowKind,
    val label: String,
    val node: TreeNode,
) {
    val fileId: Long? get() = node.fileId
}

data class RowsResult(
    val rows: List<TreeRow>,
    /** 路径在当前树里找不到：树刷新后旧路径会失效，界面应回根层。 */
    val notFound: Boolean = false,
)

object FileTree {

    const val ROOT_LABEL = "全部"

    fun parse(array: JsonArray?): List<TreeNode> = array?.mapNotNull { parseNode(it) } ?: emptyList()

    fun parseRoot(element: JsonElement?): List<TreeNode> = parse(element.asArrayOrNull())

    private fun parseNode(element: JsonElement): TreeNode? {
        val obj = element.asObjectOrNull() ?: return null
        val type = obj.stringOrNull("type").orEmpty()
        return TreeNode(
            id = obj.stringOrNull("id").orEmpty(),
            label = obj.stringOrNull("label").orEmpty(),
            type = type,
            // 只读 fileId：叶子的 id 是 "file-12"，从字符串里 parse 数字遇到改前缀就全崩。
            fileId = obj.longOrNull("fileId"),
            children = parse(obj["children"].asArrayOrNull()),
            payload = if (type == "file") obj else null,
        )
    }

    /** 标签路径 → 行。path 是标签不是 id：id 是路径哈希，改名就变。 */
    fun rowsAt(tree: List<TreeNode>, path: List<String>): RowsResult {
        var level = tree
        for (segment in path) {
            val next = level.firstOrNull { it.label == segment }
                ?: return RowsResult(rows = emptyList(), notFound = true)
            if (!next.isFolder) return RowsResult(rows = emptyList(), notFound = true)
            level = next.children
        }
        val folders = level.filter { it.isFolder }.map { TreeRow(RowKind.Folder, it.label, it) }
        val files = level.filterNot { it.isFolder }.map { TreeRow(RowKind.File, it.label, it) }
        return RowsResult(rows = folders + files)
    }

    /** 面包屑：根「全部」+ 标签路径原样。 */
    fun breadcrumb(path: List<String>): List<String> = listOf(ROOT_LABEL) + path

    /** 叶子行：标题取 label（服务端已兜底「未知科目」等），fileId 来自数字字段。 */
    fun leafRow(node: TreeNode): TreeRow = TreeRow(RowKind.File, node.label, node)

    /** 展示标签：服务端给什么显什么，客户端不再兜底。 */
    fun displayLabel(node: TreeNode): String = node.label
}
