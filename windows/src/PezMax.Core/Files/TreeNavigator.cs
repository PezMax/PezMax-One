using System;
using System.Collections.Generic;
using PezMax.Core.Models;

namespace PezMax.Core.Files;

public enum TreeRowKind
{
    Folder,
    File,
}

/// <summary>导航列表里的一行：文件夹或文件。</summary>
public sealed class TreeRow
{
    public TreeRow(TreeRowKind kind, FileTreeNode node)
    {
        Kind = kind;
        Node = node;
    }

    public TreeRowKind Kind { get; }
    public FileTreeNode Node { get; }
    public string Label => Node.Label;

    /// <summary>文件行才有意义；**不**从 <see cref="FileTreeNode.Id"/> 解析。</summary>
    public long FileId => Kind == TreeRowKind.File ? Node.FileId : 0;

    public override string ToString() => (Kind == TreeRowKind.Folder ? "folder:" : "file:") + Label;
}

/// <summary>某个文件夹下的内容；<see cref="NotFound"/> 表示标签路径已经失效（树刷新过）。</summary>
public sealed class TreeChildren
{
    public TreeChildren(IReadOnlyList<TreeRow> rows, bool notFound)
    {
        Rows = rows;
        NotFound = notFound;
    }

    public IReadOnlyList<TreeRow> Rows { get; }
    public bool NotFound { get; }
}

/// <summary>
/// 树响应 → 导航模型，规则见 <c>spec/fixtures/file-tree/</c>：
/// 文件夹在前、文件在后（各自保持服务端顺序）；面包屑带根「全部」；
/// 行的身份读 <c>fileId</c>，不解析 <c>"file-12"</c> 这种 id；服务端给的兜底标签原样显示。
/// </summary>
public static class TreeNavigator
{
    public const string RootLabel = "全部";

    public static TreeChildren Children(IReadOnlyList<FileTreeNode>? tree, IReadOnlyList<string>? path)
    {
        var nodes = tree ?? Array.Empty<FileTreeNode>();
        var segments = path ?? Array.Empty<string>();

        var current = nodes;
        for (var i = 0; i < segments.Count; i++)
        {
            var next = FindFolder(current, segments[i]);
            if (next == null) return new TreeChildren(Array.Empty<TreeRow>(), true);
            current = next.Children;
        }

        var folders = new List<TreeRow>();
        var files = new List<TreeRow>();
        foreach (var node in current)
        {
            if (node.IsFolder) folders.Add(new TreeRow(TreeRowKind.Folder, node));
            else files.Add(new TreeRow(TreeRowKind.File, node));
        }

        // 服务端 ORDER BY 不含 remark，子目录会夹在兄弟叶子之间；导航里统一文件夹在前，
        // 但**不**重排顺序——年份是 DESC，重排会把「新的在前」弄丢。
        var rows = new List<TreeRow>(folders.Count + files.Count);
        rows.AddRange(folders);
        rows.AddRange(files);
        return new TreeChildren(rows, false);
    }

    public static IReadOnlyList<string> Breadcrumb(IReadOnlyList<string>? path)
    {
        var labels = new List<string> { RootLabel };
        if (path != null) labels.AddRange(path);
        return labels;
    }

    /// <summary>行的身份永远来自 <see cref="FileTreeNode.FileId"/>。</summary>
    public static long FileIdOf(FileTreeNode node) => node.FileId;

    private static FileTreeNode? FindFolder(IReadOnlyList<FileTreeNode> nodes, string label)
    {
        foreach (var node in nodes)
        {
            if (!node.IsFolder) continue;
            if (string.Equals(node.Label, label, StringComparison.Ordinal)) return node;
        }
        return null;
    }
}
