using System.Text.Json;
using PezMax.Core.Reading;
using Xunit;

namespace PezMax.Core.Tests;

/// <summary>EPUB 包解析（container → OPF → spine → 目录），逐条对齐 `spec/fixtures/epub/cases.json`。</summary>
public class EpubFixtureTests
{
    [Fact]
    public void Parse_MatchesEpubFixtures()
    {
        using var document = Spec.LoadJson("fixtures", "epub", "cases.json");
        var cases = document.RootElement;
        Assert.Equal(JsonValueKind.Array, cases.ValueKind);

        var checkedCases = 0;
        foreach (var item in cases.EnumerateArray())
        {
            var id = item.GetProperty("id").GetString()!;
            var input = item.GetProperty("input");
            var expect = item.GetProperty("expect");

            Assert.True(input.GetProperty("kind").GetString() == "package", $"{id}: 未知 kind");

            var files = new Dictionary<string, string>(StringComparer.Ordinal);
            foreach (var file in input.GetProperty("files").EnumerateObject())
            {
                files[file.Name] = file.Value.GetString()!;
            }

            var book = EpubPackage.Parse(files);

            if (expect.TryGetProperty("failure", out var failure))
            {
                Assert.False(string.IsNullOrEmpty(book.Failure), $"{id}: 应该解析失败");
                Assert.Contains(failure.GetString()!, book.Failure);
                Assert.Empty(book.Chapters);
                checkedCases++;
                continue;
            }

            Assert.True(book.Failure == null, $"{id}: 不该失败（{book.Failure}）");
            Assert.True(expect.GetProperty("title").GetString() == book.Title, $"{id}: title = {book.Title}");
            Assert.True(expect.GetProperty("author").GetString() == book.Author, $"{id}: author = {book.Author}");
            Assert.True(expect.GetProperty("rootDirectory").GetString() == book.RootDirectory,
                $"{id}: rootDirectory = {book.RootDirectory}");

            var expected = expect.GetProperty("chapters").EnumerateArray().ToList();
            Assert.True(expected.Count == book.Chapters.Count,
                $"{id}: 章节数 期望 {expected.Count}、实际 {book.Chapters.Count}");
            for (var i = 0; i < expected.Count; i++)
            {
                Assert.True(expected[i].GetProperty("href").GetString() == book.Chapters[i].Href,
                    $"{id}: [{i}] href = {book.Chapters[i].Href}");
                Assert.True(expected[i].GetProperty("path").GetString() == book.Chapters[i].Path,
                    $"{id}: [{i}] path = {book.Chapters[i].Path}");
                Assert.True(expected[i].GetProperty("title").GetString() == book.Chapters[i].Title,
                    $"{id}: [{i}] title = {book.Chapters[i].Title}");
                Assert.Equal(i, book.Chapters[i].Order);
            }

            checkedCases++;
        }

        Assert.True(checkedCases >= 13, $"EPUB 夹具跑到的用例太少：{checkedCases}");
    }

    [Fact]
    public void ResolveAgainst_HandlesFragmentsEncodingAndParentSegments()
    {
        Assert.Equal("OEBPS/ch 1.xhtml", EpubPackage.ResolveAgainst("OEBPS", "ch%201.xhtml#top"));
        Assert.Equal("Text/ch1.xhtml", EpubPackage.ResolveAgainst("OEBPS", "../Text/ch1.xhtml"));
        Assert.Equal("ch1.xhtml", EpubPackage.ResolveAgainst("", "./ch1.xhtml"));
        Assert.Equal("ch1.xhtml", EpubPackage.ResolveAgainst("OEBPS", "/ch1.xhtml"));
        Assert.Equal("OEBPS/ch1.xhtml", EpubPackage.ResolveAgainst("OEBPS/", "ch1.xhtml?v=2"));
    }

    [Fact]
    public void ResolveAgainst_WalksOutOfRootWithoutThrowing()
    {
        // ../../.. 越过包根：夹在根上，不要拼出 "../" 这种 zip 里不存在的路径。
        Assert.Equal("ch1.xhtml", EpubPackage.ResolveAgainst("OEBPS", "../../../ch1.xhtml"));
    }

    [Fact]
    public void DirectoryOf_HandlesRootAndNestedPaths()
    {
        Assert.Equal("", EpubPackage.DirectoryOf("content.opf"));
        Assert.Equal("OEBPS", EpubPackage.DirectoryOf("OEBPS/content.opf"));
        Assert.Equal("OEBPS/nav", EpubPackage.DirectoryOf("OEBPS/nav/nav.xhtml"));
    }
}
