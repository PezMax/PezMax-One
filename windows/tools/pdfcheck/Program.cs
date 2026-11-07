// pdfcheck —— 用客户端阅读器同一套栈（Windows.Data.Pdf）验一份 PDF 到底能不能开。
//
// 为什么要它：阅读器报「文件可能损坏」时，分不清是文件坏了还是我们喂给
// PdfDocument 的流有问题。这个工具只吃文件路径、不碰 UWP、不碰打包，
// 一秒给出「能开 / 不能开 + 页数 + 首屏渲染耗时」。
//
//   dotnet run --project windows\tools\pdfcheck -- path\to\file.pdf [--pages 0-4]

using System.Diagnostics;
using Windows.Data.Pdf;
using Windows.Storage;
using Windows.Storage.Streams;

var path = args.Length > 0 ? args[0] : null;
if (path == null || !File.Exists(path))
{
    Console.Error.WriteLine("用法: pdfcheck <file.pdf> [--pages 0-4]");
    return 2;
}

var file = await StorageFile.GetFileFromPathAsync(Path.GetFullPath(path));

// 两条路都试：StorageFile（UWP 里最自然）与纯内存流（阅读器实际用的那条）。
var failures = new List<string>();

try
{
    using var stream = await file.OpenReadAsync();
    var document = await PdfDocument.LoadFromStreamAsync(stream);
    Console.WriteLine($"StorageFile 流: 打开成功，{document.PageCount} 页");
    await RenderAsync(document, "StorageFile", ParseRange(args));
}
catch (Exception ex)
{
    failures.Add($"StorageFile 流 → {ex.GetType().Name}: {ex.Message} (0x{ex.HResult:X8})");
}

try
{
    var bytes = await File.ReadAllBytesAsync(file.Path);
    var memory = new InMemoryRandomAccessStream();
    using (var writer = memory.AsStreamForWrite())
    {
        await writer.WriteAsync(bytes);
        await writer.FlushAsync();
    }

    memory.Seek(0);
    var document = await PdfDocument.LoadFromStreamAsync(memory);
    Console.WriteLine($"内存流: 打开成功，{document.PageCount} 页，{bytes.Length} 字节");
    await RenderAsync(document, "内存流", ParseRange(args));
}
catch (Exception ex)
{
    failures.Add($"内存流 → {ex.GetType().Name}: {ex.Message} (0x{ex.HResult:X8})");
}

foreach (var failure in failures) Console.Error.WriteLine("失败: " + failure);
return failures.Count == 0 ? 0 : 1;

static (int From, int To) ParseRange(string[] args)
{
    var index = Array.IndexOf(args, "--pages");
    if (index < 0 || index + 1 >= args.Length) return (0, 3);
    var parts = args[index + 1].Split('-');
    var from = int.TryParse(parts[0], out var f) ? f : 0;
    var to = parts.Length > 1 && int.TryParse(parts[1], out var t) ? t : from;
    return (from, to);
}

static async Task RenderAsync(PdfDocument document, string label, (int From, int To) range)
{
    var count = (int)document.PageCount;
    for (var i = Math.Max(0, range.From); i <= Math.Min(range.To, count - 1); i++)
    {
        var watch = Stopwatch.StartNew();
        using var page = document.GetPage((uint)i);
        using var stream = new InMemoryRandomAccessStream();
        await page.RenderToStreamAsync(stream);
        stream.Seek(0);
        Console.WriteLine($"  [{label}] 第 {i} 页: {page.Size.Width:0}×{page.Size.Height:0} pt，" +
                          $"渲染 {watch.ElapsedMilliseconds} ms，PNG {stream.Size} 字节");
    }
}
