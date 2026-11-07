using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Text;

namespace PezMax.Core.Validation;

/// <summary>单个文件的上传判定结果。</summary>
public sealed class UploadFileVerdict
{
    public bool Accepted { get; init; }
    public string? Reason { get; init; }

    /// <summary>该格式会被服务端转成 PDF 存储（`doc/docx/ppt/pptx`）。</summary>
    public bool ConvertsToPdf { get; init; }

    /// <summary>上传前要自己验 UTF-8（`txt` / `md`）。</summary>
    public bool NeedsUtf8Check { get; init; }

    /// <summary>格式是电子书，但走的是资料上传：只会进 `ptmj_file`，不会出现在电子书列表。</summary>
    public bool WarnsNotEbook { get; init; }
}

/// <summary>元信息表单的判定结果。</summary>
public sealed class UploadMetaVerdict
{
    public bool Accepted { get; init; }
    public string? Reason { get; init; }
    public bool WarnsNotEbook { get; init; }
}

/// <summary>批量上传里单个文件的结果。</summary>
public sealed class UploadBatchItem
{
    public string Name { get; init; } = "";
    public bool Accepted { get; init; }
    public string? Reason { get; init; }
}

/// <summary>
/// 上传前的本地校验，逐条对齐 <c>spec/fixtures/upload/</c> 与
/// `datum_file.go:723-825`、`datum_ebook.go:354-417`。
/// <para>
/// 判定顺序照抄服务端：体积 → 文件名 → 科目 → 学校 → 年份 → 类型 → 扩展名 → 编码 → 转档体积。
/// **字节与 rune 混用**是这里最容易写错的地方：`fileName` 按 rune 计，`fileSubject` / `fileSchool`
/// 按字节计（一个汉字 3 字节，中文科目实际只有 21 个字）。
/// </para>
/// </summary>
public static class UploadValidation
{
    public const int DefaultMaxUploadMb = 200;
    public const int ConvertMaxMb = 200;

    public static readonly IReadOnlyList<string> FileExtensions = new[]
    {
        "doc", "docx", "pdf", "jpg", "jpeg", "png", "webp", "txt", "md", "ppt", "pptx", "epub", "mobi", "azw3",
    };

    public static readonly IReadOnlyList<string> EbookExtensions = new[] { "pdf", "epub", "mobi", "azw3" };

    private static readonly HashSet<string> ConvertSet = new(StringComparer.OrdinalIgnoreCase)
    {
        "doc", "docx", "ppt", "pptx",
    };

    private static readonly HashSet<string> EbookSet = new(StringComparer.OrdinalIgnoreCase)
    {
        "epub", "mobi", "azw3",
    };

    /// <summary>服务端那句文案只列了 10 种，但白名单有 14 种——照抄文案去限制选择器会砍掉合法格式。</summary>
    public const string UnsupportedFileMessage =
        "不支持的文件格式：仅支持 doc/docx/pdf/jpg/png/webp/txt/md/ppt/pptx";

    public const string UnsupportedEbookMessage = "不支持的电子书格式：仅支持 pdf/epub/mobi/azw3";

    public static UploadFileVerdict ValidateFile(string? fileName, long sizeBytes, int maxUploadMb = DefaultMaxUploadMb)
    {
        var limit = maxUploadMb <= 0 ? DefaultMaxUploadMb : maxUploadMb;

        if (sizeBytes <= 0 || sizeBytes > (long)limit * 1024 * 1024)
            return Reject("文件过大：不能超过 " + limit.ToString(CultureInfo.InvariantCulture) + "MB");

        var runes = RuneCount(fileName);
        if (runes == 0 || runes > 255) return Reject("文件名不能为空且不超过 255 个字符");

        var extension = ExtensionOf(fileName, FileExtensions);
        if (extension == null) return Reject(UnsupportedFileMessage);

        if (ConvertSet.Contains(extension) && sizeBytes > (long)ConvertMaxMb * 1024 * 1024)
            return Reject("转档文件不能超过 " + ConvertMaxMb.ToString(CultureInfo.InvariantCulture) + "MB");

        return new UploadFileVerdict
        {
            Accepted = true,
            ConvertsToPdf = ConvertSet.Contains(extension),
            NeedsUtf8Check = extension.Equals("txt", StringComparison.OrdinalIgnoreCase) ||
                             extension.Equals("md", StringComparison.OrdinalIgnoreCase),
            WarnsNotEbook = EbookSet.Contains(extension),
        };
    }

    public static UploadMetaVerdict ValidateFileMeta(
        string? fileName,
        string? fileSubject,
        string? fileSchool,
        long fileYear,
        long fileType)
    {
        var runes = RuneCount(fileName);
        if (runes == 0 || runes > 255) return MetaReject("文件名不能为空且不超过 255 个字符");

        // 按字节：一个汉字 3 字节，64 字节实际只有 21 个字。
        var subject = (fileSubject ?? "").Trim();
        if (subject.Length == 0 || Encoding.UTF8.GetByteCount(subject) > 64)
            return MetaReject("科目不能为空且不超过 64 个字符");

        if (Encoding.UTF8.GetByteCount((fileSchool ?? "").Trim()) > 100)
            return MetaReject("学校名称不能超过 100 个字符");

        if (fileYear < 1900 || fileYear > 2100) return MetaReject("年份需在 1900 到 2100 之间");
        if (fileType < 1 || fileType > 7) return MetaReject("文件类型不正确");

        // fileType=7 能传，但只会进 ptmj_file：界面应引导到电子书上传。
        return new UploadMetaVerdict { Accepted = true, WarnsNotEbook = fileType == 7 };
    }

    public static UploadMetaVerdict ValidateEbookMeta(
        string? fileName,
        string? ebookName,
        string? ebookSubject,
        long ebookType,
        string? author = null,
        string? publisher = null)
    {
        var name = (ebookName ?? "").Trim();
        if (name.Length == 0)
        {
            name = (fileName ?? "").Trim();
        }

        var runes = RuneCount(name);
        if (runes == 0 || runes > 255) return MetaReject("书名不能为空且不超过 255 个字符");
        if (RuneCount(author) > 255) return MetaReject("作者不能超过 255 个字符");
        if (RuneCount(publisher) > 255) return MetaReject("出版社不能超过 255 个字符");

        var subject = (ebookSubject ?? "").Trim();
        if (subject.Length == 0 || Encoding.UTF8.GetByteCount(subject) > 64)
            return MetaReject("学科分类不能为空且不超过 64 个字符");

        if (ebookType < 1 || ebookType > 4) return MetaReject("电子书类型不正确");

        if (ExtensionOf(fileName, EbookExtensions) == null) return MetaReject(UnsupportedEbookMessage);

        return new UploadMetaVerdict { Accepted = true };
    }

    /// <summary>
    /// 批量：逐个判、逐个出结果。服务端是单文件端点，一个坏文件不该拖垮整批。
    /// </summary>
    public static IReadOnlyList<UploadBatchItem> ValidateBatch(
        IEnumerable<KeyValuePair<string, long>>? files,
        int maxUploadMb = DefaultMaxUploadMb)
    {
        var results = new List<UploadBatchItem>();
        if (files == null) return results;

        foreach (var file in files)
        {
            var verdict = ValidateFile(file.Key, file.Value, maxUploadMb);
            results.Add(new UploadBatchItem
            {
                Name = file.Key,
                Accepted = verdict.Accepted,
                Reason = verdict.Reason,
            });
        }

        return results;
    }

    /// <summary>扩展名（最长后缀匹配）；不在白名单里返回 null。</summary>
    public static string? ExtensionOf(string? fileName, IReadOnlyList<string> extensions)
    {
        if (string.IsNullOrEmpty(fileName)) return null;
        var text = fileName!;
        foreach (var extension in extensions)
        {
            if (text.EndsWith("." + extension, StringComparison.OrdinalIgnoreCase)) return extension;
        }
        return null;
    }

    public static int RuneCount(string? value)
    {
        if (string.IsNullOrEmpty(value)) return 0;
        var count = 0;
        for (var i = 0; i < value!.Length; i++)
        {
            if (char.IsHighSurrogate(value[i]) && i + 1 < value.Length && char.IsLowSurrogate(value[i + 1])) i++;
            count++;
        }
        return count;
    }

    private static UploadFileVerdict Reject(string reason) => new() { Accepted = false, Reason = reason };
    private static UploadMetaVerdict MetaReject(string reason) => new() { Accepted = false, Reason = reason };
}
