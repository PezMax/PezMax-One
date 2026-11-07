using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.RegularExpressions;
using System.Xml.Linq;

namespace PezMax.Core.Reading;

/// <summary>书里的一章（= spine 里的一个 XHTML 文档）。</summary>
public sealed class EpubChapter
{
    /// <summary>manifest 里写的原始 href（含百分号编码，不含 fragment）。</summary>
    public string Href { get; init; } = "";

    /// <summary>解出来的 zip 内路径（已解码、已消掉 <c>../</c>）。</summary>
    public string Path { get; init; } = "";

    /// <summary>目录里给的标题；目录没列这一章时为空，由界面编号。</summary>
    public string Title { get; init; } = "";

    /// <summary>在 spine 里的顺序（0 基）。</summary>
    public int Order { get; init; }
}

/// <summary>解析出来的书：元信息、OPF 所在目录、章节顺序。失败时 <see cref="Failure"/> 非空。</summary>
public sealed class EpubBook
{
    public string Title { get; init; } = "";

    public string Author { get; init; } = "";

    /// <summary>OPF 所在目录（空字符串表示在包根）。拼虚拟主机 URL 时要带上。</summary>
    public string RootDirectory { get; init; } = "";

    public IReadOnlyList<EpubChapter> Chapters { get; init; } = Array.Empty<EpubChapter>();

    /// <summary>失败原因（找不到 OPF、XML 坏了等）；成功时为 null。</summary>
    public string? Failure { get; init; }

    public bool IsUsable => Failure == null && Chapters.Count > 0;
}

/// <summary>
/// EPUB 包解析：<c>META-INF/container.xml</c> → OPF（元信息 + manifest + spine）→ 目录（nav / NCX）。
/// 逐条对齐 <c>spec/fixtures/epub/</c>。只管**读结构**：解压、CSS、图片、分页都不在这里。
/// </summary>
public static class EpubPackage
{
    public const string ContainerPath = "META-INF/container.xml";

    private const string XhtmlMediaType = "application/xhtml+xml";
    private const string HtmlMediaType = "text/html";
    private const string NcxMediaType = "application/x-dtbncx+xml";

    /// <param name="files">zip 条目 → 文本内容。二进制条目（图片、字体）不必给。</param>
    public static EpubBook Parse(IReadOnlyDictionary<string, string>? files)
    {
        if (files == null || files.Count == 0) return Fail("包是空的");

        var opfPath = FindPackagePath(files);
        if (opfPath == null) return Fail("找不到 .opf（包结构不完整）");
        if (!files.TryGetValue(opfPath, out var opfText)) return Fail("找不到 " + opfPath);

        XDocument opf;
        try
        {
            opf = XDocument.Parse(opfText);
        }
        catch (Exception)
        {
            return Fail("OPF 不是合法 XML：" + opfPath);
        }

        var rootDirectory = DirectoryOf(opfPath);
        var title = FirstElementText(opf, "metadata", "title");
        var author = FirstElementText(opf, "metadata", "creator");
        var manifest = ReadManifest(opf);
        var spine = ReadSpine(opf);

        var chapters = new List<EpubChapter>();
        foreach (var idref in spine)
        {
            if (!manifest.TryGetValue(idref, out var item)) continue;

            // spine 里混进图片 / CSS 的包是有的：当成章节打开只会得到一张图。
            var mediaType = item.MediaType;
            if (!IsHtml(mediaType)) continue;

            chapters.Add(new EpubChapter
            {
                Href = item.Href,
                Path = ResolveAgainst(rootDirectory, item.Href),
                Order = chapters.Count,
            });
        }

        if (chapters.Count == 0) return Fail("spine 里没有可显示的章节");

        var titles = ReadTitles(files, opf, manifest, rootDirectory, SpineTocId(opf));
        for (var i = 0; i < chapters.Count; i++)
        {
            if (titles.TryGetValue(chapters[i].Path, out var chapterTitle) && chapterTitle.Length > 0)
            {
                chapters[i] = new EpubChapter
                {
                    Href = chapters[i].Href,
                    Path = chapters[i].Path,
                    Order = chapters[i].Order,
                    Title = chapterTitle,
                };
            }
        }

        return new EpubBook
        {
            Title = title,
            Author = author,
            RootDirectory = rootDirectory,
            Chapters = chapters,
        };
    }

    /// <summary>相对 URL 按基准目录解析成 zip 内路径：去掉 fragment / query、解码百分号、消掉 <c>./</c> 与 <c>../</c>。</summary>
    public static string ResolveAgainst(string? baseDirectory, string? href)
    {
        var text = (href ?? "").Trim();
        var hash = text.IndexOf('#');
        if (hash >= 0) text = text.Substring(0, hash);
        var query = text.IndexOf('?');
        if (query >= 0) text = text.Substring(0, query);

        try
        {
            text = Uri.UnescapeDataString(text);
        }
        catch (Exception)
        {
            // 非法的百分号序列：按原样用，别整本打不开。
        }

        text = text.Replace('\\', '/');
        if (text.StartsWith("/", StringComparison.Ordinal)) return Collapse(text.Substring(1));

        var basePath = (baseDirectory ?? "").Trim('/');
        return Collapse(basePath.Length == 0 ? text : basePath + "/" + text);
    }

    public static string DirectoryOf(string? path)
    {
        var text = (path ?? "").Replace('\\', '/');
        var slash = text.LastIndexOf('/');
        return slash <= 0 ? "" : text.Substring(0, slash);
    }

    private static string? FindPackagePath(IReadOnlyDictionary<string, string> files)
    {
        if (files.TryGetValue(ContainerPath, out var container))
        {
            try
            {
                var document = XDocument.Parse(container);
                var rootfile = document.Descendants()
                    .FirstOrDefault(element => element.Name.LocalName == "rootfile");
                var fullPath = rootfile?.Attribute("full-path")?.Value;
                if (!string.IsNullOrWhiteSpace(fullPath)) return fullPath!.Trim();
            }
            catch (Exception)
            {
                // container.xml 坏了就退回「找一个 .opf」。
            }
        }

        // 手工打的包常常缺 container.xml：按名字挑第一个 .opf。
        return files.Keys
            .Where(name => name.EndsWith(".opf", StringComparison.OrdinalIgnoreCase))
            .OrderBy(name => name, StringComparer.Ordinal)
            .FirstOrDefault();
    }

    private sealed class ManifestItem
    {
        public string Href { get; init; } = "";
        public string MediaType { get; init; } = "";
        public string Properties { get; init; } = "";
    }

    private static Dictionary<string, ManifestItem> ReadManifest(XDocument opf)
    {
        var items = new Dictionary<string, ManifestItem>(StringComparer.Ordinal);
        foreach (var item in Elements(opf, "manifest", "item"))
        {
            var id = item.Attribute("id")?.Value;
            var href = item.Attribute("href")?.Value;
            if (string.IsNullOrEmpty(id) || string.IsNullOrEmpty(href)) continue;

            items[id!] = new ManifestItem
            {
                Href = href!,
                MediaType = (item.Attribute("media-type")?.Value ?? "").Trim(),
                Properties = (item.Attribute("properties")?.Value ?? "").Trim(),
            };
        }

        return items;
    }

    private static List<string> ReadSpine(XDocument opf)
    {
        var ids = new List<string>();
        foreach (var itemref in Elements(opf, "spine", "itemref"))
        {
            var idref = itemref.Attribute("idref")?.Value;
            if (!string.IsNullOrWhiteSpace(idref)) ids.Add(idref!.Trim());
        }

        return ids;
    }

    private static string? SpineTocId(XDocument opf)
    {
        var spine = Elements(opf, null, "spine").FirstOrDefault();
        return spine?.Attribute("toc")?.Value;
    }

    private static Dictionary<string, string> ReadTitles(
        IReadOnlyDictionary<string, string> files,
        XDocument opf,
        Dictionary<string, ManifestItem> manifest,
        string rootDirectory,
        string? spineTocId)
    {
        var titles = new Dictionary<string, string>(StringComparer.Ordinal);

        // 先看 EPUB 3 的 nav，再退回 EPUB 2 的 NCX；两处都没有就只剩空白标题。
        var navItem = manifest.Values.FirstOrDefault(item => HasToken(item.Properties, "nav"));
        if (navItem != null && TryReadNavTitles(files, navItem, rootDirectory, titles)) return titles;

        var ncxId = spineTocId;
        if (string.IsNullOrWhiteSpace(ncxId))
        {
            var item = manifest.FirstOrDefault(pair => pair.Value.MediaType.Equals(NcxMediaType, StringComparison.OrdinalIgnoreCase));
            ncxId = item.Key;
        }

        if (!string.IsNullOrWhiteSpace(ncxId) && manifest.TryGetValue(ncxId!, out var ncxItem))
        {
            TryReadNcxTitles(files, ncxItem, rootDirectory, titles);
        }

        return titles;
    }

    private static bool TryReadNavTitles(
        IReadOnlyDictionary<string, string> files,
        ManifestItem navItem,
        string rootDirectory,
        Dictionary<string, string> titles)
    {
        // manifest 的 href 相对 OPF；nav 里的链接相对 nav 自己——两者不是同一个基准。
        var navPath = ResolveAgainst(rootDirectory, navItem.Href);
        if (!files.TryGetValue(navPath, out var navText)) return false;

        XDocument nav;
        try
        {
            nav = XDocument.Parse(navText);
        }
        catch (Exception)
        {
            return false;
        }

        var navDirectory = DirectoryOf(navPath);
        var toc = nav.Descendants()
            .FirstOrDefault(element => element.Name.LocalName == "nav" && HasToken(element.Attribute("type")?.Value, "toc"));
        toc ??= nav.Descendants().FirstOrDefault(element => element.Name.LocalName == "nav");
        if (toc == null) return false;

        var added = false;
        foreach (var link in toc.Descendants().Where(element => element.Name.LocalName == "a"))
        {
            var href = link.Attribute("href")?.Value;
            if (string.IsNullOrWhiteSpace(href)) continue;

            var path = ResolveAgainst(navDirectory, href);
            if (path.Length == 0) continue;

            // 同一章出现两次时第一次为准：卷首 / 卷末各列一遍很常见。
            if (!titles.ContainsKey(path)) titles[path] = NormalizeText(link.Value);
            added = true;
        }

        return added;
    }

    private static void TryReadNcxTitles(
        IReadOnlyDictionary<string, string> files,
        ManifestItem ncxItem,
        string rootDirectory,
        Dictionary<string, string> titles)
    {
        var ncxPath = ResolveAgainst(rootDirectory, ncxItem.Href);
        if (!files.TryGetValue(ncxPath, out var ncxText)) return;

        XDocument ncx;
        try
        {
            ncx = XDocument.Parse(ncxText);
        }
        catch (Exception)
        {
            return;
        }

        var navDirectory = DirectoryOf(ncxPath);
        foreach (var point in ncx.Descendants().Where(element => element.Name.LocalName == "navPoint"))
        {
            var label = point.Elements().FirstOrDefault(element => element.Name.LocalName == "navLabel")
                ?.Elements().FirstOrDefault(element => element.Name.LocalName == "text");
            var source = point.Elements().FirstOrDefault(element => element.Name.LocalName == "content")
                ?.Attribute("src")?.Value;
            if (label == null || string.IsNullOrWhiteSpace(source)) continue;

            var path = ResolveAgainst(navDirectory, source);
            if (path.Length == 0 || titles.ContainsKey(path)) continue;
            titles[path] = NormalizeText(label.Value);
        }
    }

    private static IEnumerable<XElement> Elements(XDocument document, string? parentLocalName, string elementLocalName)
    {
        var roots = parentLocalName == null
            ? document.Descendants()
            : document.Descendants().Where(element => element.Name.LocalName == parentLocalName);

        return roots.SelectMany(root => root.Descendants().Where(element => element.Name.LocalName == elementLocalName));
    }

    private static string FirstElementText(XDocument document, string parentLocalName, string elementLocalName) =>
        NormalizeText(Elements(document, parentLocalName, elementLocalName).FirstOrDefault()?.Value);

    private static string NormalizeText(string? value)
    {
        var text = (value ?? "").Replace('\r', ' ').Replace('\n', ' ').Trim();
        return text.Length == 0 ? "" : Regex.Replace(text, "\\s+", " ");
    }

    private static bool IsHtml(string mediaType) =>
        mediaType.Equals(XhtmlMediaType, StringComparison.OrdinalIgnoreCase) ||
        mediaType.Equals(HtmlMediaType, StringComparison.OrdinalIgnoreCase);

    private static bool HasToken(string? value, string token) =>
        (value ?? "").Split(new[] { ' ', '\t', '\r', '\n' }, StringSplitOptions.RemoveEmptyEntries)
            .Any(part => part.Equals(token, StringComparison.OrdinalIgnoreCase));

    private static string Collapse(string path)
    {
        var parts = path.Split('/');
        var stack = new List<string>();
        foreach (var part in parts)
        {
            if (part.Length == 0 || part == ".") continue;
            if (part == "..")
            {
                if (stack.Count > 0) stack.RemoveAt(stack.Count - 1);
                continue;
            }

            stack.Add(part);
        }

        return string.Join("/", stack);
    }

    private static EpubBook Fail(string reason) => new() { Failure = reason };
}
