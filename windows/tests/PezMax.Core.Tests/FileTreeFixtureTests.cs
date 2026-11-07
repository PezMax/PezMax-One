using System.Text.Json;
using PezMax.Core.Files;
using PezMax.Core.Models;
using Xunit;

namespace PezMax.Core.Tests;

/// <summary>树响应 → 导航模型，逐条对齐 `spec/fixtures/file-tree/cases.json`。</summary>
public class FileTreeFixtureTests
{
    [Fact]
    public void Navigator_MatchesFileTreeFixtures()
    {
        using var document = Spec.LoadJson("fixtures", "file-tree", "cases.json");
        var cases = document.RootElement;
        Assert.Equal(JsonValueKind.Array, cases.ValueKind);

        var checkedCases = 0;
        foreach (var item in cases.EnumerateArray())
        {
            var id = item.GetProperty("id").GetString()!;
            var input = item.GetProperty("input");
            var kind = input.GetProperty("kind").GetString();
            var expect = item.GetProperty("expect");

            switch (kind)
            {
                case "rows":
                {
                    var tree = ReadTree(input.GetProperty("tree"));
                    var path = ReadStrings(input.GetProperty("path"));
                    var children = TreeNavigator.Children(tree, path);

                    var expected = ReadStrings(expect.GetProperty("order"));
                    var actual = children.Rows.Select(r => r.ToString()).ToList();
                    Assert.True(expected.SequenceEqual(actual),
                        $"{id}: 期望 [{string.Join(", ", expected)}]，实际 [{string.Join(", ", actual)}]");

                    if (expect.TryGetProperty("notFound", out var notFound))
                        Assert.True(notFound.GetBoolean() == children.NotFound, $"{id}: notFound = {children.NotFound}");
                    break;
                }

                case "leaf":
                {
                    var node = ReadNode(input.GetProperty("node"));
                    if (expect.TryGetProperty("fileId", out var fileId))
                        Assert.True(fileId.GetInt64() == TreeNavigator.FileIdOf(node), $"{id}: fileId = {node.FileId}");
                    if (expect.TryGetProperty("title", out var title))
                        Assert.True(title.GetString() == node.Label, $"{id}: title = {node.Label}");
                    if (expect.TryGetProperty("kindOfRow", out var kindOfRow))
                    {
                        var row = new TreeRow(node.IsFolder ? TreeRowKind.Folder : TreeRowKind.File, node);
                        Assert.True(kindOfRow.GetString() == (row.Kind == TreeRowKind.File ? "file" : "folder"),
                            $"{id}: kind = {row.Kind}");
                    }
                    if (expect.TryGetProperty("fileFormat", out var format))
                        Assert.True(format.GetString() == (node.File?.FileFormat ?? ""), $"{id}: fileFormat");
                    if (expect.TryGetProperty("fileYear", out var year))
                        Assert.True(year.GetInt64() == (node.File?.FileYear ?? 0), $"{id}: fileYear");
                    if (expect.TryGetProperty("fileSubject", out var subject))
                        Assert.True(subject.GetString() == (node.File?.FileSubject ?? ""), $"{id}: fileSubject");
                    break;
                }

                case "breadcrumb":
                {
                    var labels = TreeNavigator.Breadcrumb(ReadStrings(input.GetProperty("path")));
                    var expected = ReadStrings(expect.GetProperty("labels"));
                    Assert.True(expected.SequenceEqual(labels),
                        $"{id}: 期望 [{string.Join(", ", expected)}]，实际 [{string.Join(", ", labels)}]");
                    break;
                }

                case "fallback":
                {
                    var node = ReadNode(input.GetProperty("node"));
                    Assert.True(expect.GetProperty("label").GetString() == node.Label, $"{id}: label = {node.Label}");
                    break;
                }

                default:
                    throw new Xunit.Sdk.XunitException($"{id}: 未知 kind {kind}");
            }

            checkedCases++;
        }

        Assert.True(checkedCases >= 15, $"文件树夹具跑到的用例太少：{checkedCases}");
    }

    [Fact]
    public void Navigator_NeverParsesFileIdFromNodeId()
    {
        // 叶子的 id 是 "file-12"，fileId 才是 12：拿 id 解析会跟着上游改前缀一起崩。
        var leaf = new FileTreeNode { Id = "file-12", FileId = 12, Label = "卷子.pdf", Type = "file" };
        Assert.Equal(12, TreeNavigator.FileIdOf(leaf));

        var weird = new FileTreeNode { Id = "whatever", FileId = 99, Label = "x.pdf", Type = "file" };
        Assert.Equal(99, TreeNavigator.FileIdOf(weird));
    }

    private static IReadOnlyList<FileTreeNode> ReadTree(JsonElement element) =>
        element.EnumerateArray().Select(ReadNode).ToList();

    private static FileTreeNode ReadNode(JsonElement element)
    {
        var type = element.TryGetProperty("type", out var t) ? t.GetString() ?? "" : "";
        var isFile = string.Equals(type, "file", StringComparison.OrdinalIgnoreCase);

        var children = element.TryGetProperty("children", out var kids) && kids.ValueKind == JsonValueKind.Array
            ? kids.EnumerateArray().Select(ReadNode).ToList()
            : new List<FileTreeNode>();

        return new FileTreeNode
        {
            Id = Str(element, "id"),
            Label = Str(element, "label"),
            Type = type,
            FileId = Long(element, "fileId"),
            Children = children,
            File = isFile
                ? new FileItem
                {
                    FileId = Long(element, "fileId"),
                    Label = Str(element, "label"),
                    FileName = Str(element, "fileName"),
                    FileFormat = Str(element, "fileFormat"),
                    FileSize = Long(element, "fileSize"),
                    FileYear = Long(element, "fileYear"),
                    FileType = Long(element, "fileType"),
                    FileSubject = Str(element, "fileSubject"),
                    FileSchool = Str(element, "fileSchool"),
                    FileStatus = Long(element, "fileStatus"),
                    UserId = Long(element, "userId"),
                }
                : null,
        };
    }

    private static List<string> ReadStrings(JsonElement element) =>
        element.EnumerateArray().Select(e => e.GetString() ?? "").ToList();

    private static string Str(JsonElement element, string name) =>
        element.TryGetProperty(name, out var value) && value.ValueKind == JsonValueKind.String ? value.GetString()! : "";

    private static long Long(JsonElement element, string name) =>
        element.TryGetProperty(name, out var value) && value.ValueKind == JsonValueKind.Number ? value.GetInt64() : 0;
}
