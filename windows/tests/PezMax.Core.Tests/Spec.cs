using System.Text.Json;

namespace PezMax.Core.Tests;

internal static class Spec
{
    private static readonly Lazy<string> Root = new(() =>
    {
        var dir = new DirectoryInfo(AppContext.BaseDirectory);
        while (dir != null)
        {
            var candidate = Path.Combine(dir.FullName, "spec", "api", "envelope.md");
            if (File.Exists(candidate)) return Path.Combine(dir.FullName, "spec");
            dir = dir.Parent;
        }
        throw new DirectoryNotFoundException("从测试输出目录向上找不到 spec/api/envelope.md");
    });

    public static string PathOf(params string[] parts) =>
        Path.Combine(new[] { Root.Value }.Concat(parts).ToArray());

    public static JsonDocument LoadJson(params string[] parts) =>
        JsonDocument.Parse(File.ReadAllText(PathOf(parts)));
}
