using System.Text.Json;
using PezMax.Core.Validation;
using Xunit;

namespace PezMax.Core.Tests;

/// <summary>上传前的本地校验，逐条对齐 `spec/fixtures/upload/cases.json`。</summary>
public class UploadFixtureTests
{
    [Fact]
    public void Validation_MatchesUploadFixtures()
    {
        using var document = Spec.LoadJson("fixtures", "upload", "cases.json");
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
                case "file":
                {
                    var verdict = UploadValidation.ValidateFile(
                        Str(input, "fileName"),
                        input.GetProperty("sizeBytes").GetInt64(),
                        Int(input, "maxUploadMb", UploadValidation.DefaultMaxUploadMb));

                    AssertVerdict(id, expect, verdict.Accepted, verdict.Reason);
                    if (expect.TryGetProperty("convertsToPdf", out var converts))
                        Assert.True(converts.GetBoolean() == verdict.ConvertsToPdf, $"{id}: convertsToPdf");
                    if (expect.TryGetProperty("needsUtf8Check", out var utf8))
                        Assert.True(utf8.GetBoolean() == verdict.NeedsUtf8Check, $"{id}: needsUtf8Check");
                    if (expect.TryGetProperty("warnsNotEbook", out var warns))
                        Assert.True(warns.GetBoolean() == verdict.WarnsNotEbook, $"{id}: warnsNotEbook");
                    break;
                }

                case "meta":
                {
                    var fileName = Str(input, "fileName");
                    if (input.TryGetProperty("fileNameRunes", out var runes) && runes.ValueKind == JsonValueKind.Number)
                    {
                        // fileNameRunes 是**含扩展名**的总字符数：服务端量的是整个 fileName。
                        var extension = Str(input, "fileNameExt");
                        var unit = string.IsNullOrEmpty(fileName) ? "题" : fileName;
                        var padding = runes.GetInt32() - extension.Length;
                        fileName = string.Concat(Enumerable.Repeat(unit, padding)) + extension;
                    }

                    var verdict = UploadValidation.ValidateFileMeta(
                        fileName,
                        Str(input, "fileSubject"),
                        Str(input, "fileSchool"),
                        Long(input, "fileYear"),
                        Long(input, "fileType"));

                    AssertVerdict(id, expect, verdict.Accepted, verdict.Reason);
                    if (expect.TryGetProperty("warnsNotEbook", out var warns))
                        Assert.True(warns.GetBoolean() == verdict.WarnsNotEbook, $"{id}: warnsNotEbook");
                    break;
                }

                case "ebook":
                {
                    string? author = input.TryGetProperty("author", out var a) && a.ValueKind == JsonValueKind.String
                        ? a.GetString()
                        : null;
                    if (author != null && input.TryGetProperty("authorRunes", out var authorRunes) &&
                        authorRunes.ValueKind == JsonValueKind.Number)
                    {
                        author = string.Concat(Enumerable.Repeat(author, authorRunes.GetInt32()));
                    }

                    var verdict = UploadValidation.ValidateEbookMeta(
                        Str(input, "fileName"),
                        Str(input, "ebookName"),
                        Str(input, "ebookSubject"),
                        Long(input, "ebookType"),
                        author,
                        null);

                    AssertVerdict(id, expect, verdict.Accepted, verdict.Reason);
                    break;
                }

                case "batch":
                {
                    var files = input.GetProperty("files").EnumerateArray()
                        .Select(f => new KeyValuePair<string, long>(Str(f, "fileName"), Long(f, "sizeBytes")))
                        .ToList();
                    var results = UploadValidation.ValidateBatch(
                        files,
                        Int(input, "maxUploadMb", UploadValidation.DefaultMaxUploadMb));

                    var expected = expect.GetProperty("results").EnumerateArray().ToList();
                    Assert.True(expected.Count == results.Count, $"{id}: 结果条数 {results.Count}");
                    for (var i = 0; i < expected.Count; i++)
                    {
                        Assert.True(Str(expected[i], "name") == results[i].Name, $"{id}: [{i}] name = {results[i].Name}");
                        Assert.True(expected[i].GetProperty("accept").GetBoolean() == results[i].Accepted,
                            $"{id}: [{i}] accept = {results[i].Accepted}");
                    }
                    break;
                }

                default:
                    throw new Xunit.Sdk.XunitException($"{id}: 未知 kind {kind}");
            }

            checkedCases++;
        }

        Assert.True(checkedCases >= 28, $"上传夹具跑到的用例太少：{checkedCases}");
    }

    [Fact]
    public void File_UnsupportedMessageDoesNotMatchWhitelist()
    {
        // 服务端那句文案只列 10 种，白名单是 14 种：jpeg 与三种电子书格式照样能传。
        Assert.True(UploadValidation.ValidateFile("a.jpeg", 1024).Accepted);
        Assert.True(UploadValidation.ValidateFile("a.epub", 1024).Accepted);
        var verdict = UploadValidation.ValidateFile("a.exe", 1024);
        Assert.False(verdict.Accepted);
        Assert.Equal(UploadValidation.UnsupportedFileMessage, verdict.Reason);
    }

    private static void AssertVerdict(string id, JsonElement expect, bool accepted, string? reason)
    {
        Assert.True(expect.GetProperty("accept").GetBoolean() == accepted, $"{id}: accept = {accepted}");
        if (expect.TryGetProperty("reason", out var expectedReason))
            Assert.True(expectedReason.GetString() == reason, $"{id}: reason = {reason ?? "(null)"}");
    }

    private static string Str(JsonElement element, string name) =>
        element.TryGetProperty(name, out var value) && value.ValueKind == JsonValueKind.String ? value.GetString()! : "";

    private static long Long(JsonElement element, string name) =>
        element.TryGetProperty(name, out var value) && value.ValueKind == JsonValueKind.Number ? value.GetInt64() : 0;

    private static int Int(JsonElement element, string name, int fallback) =>
        element.TryGetProperty(name, out var value) && value.ValueKind == JsonValueKind.Number ? value.GetInt32() : fallback;
}
