using PezMax.Core.Json;

namespace PezMax.Core;

public enum ThemeMode
{
    System,
    Light,
    Dark,
}

public sealed class AppSettings
{
    public ThemeMode ThemeMode { get; set; } = ThemeMode.System;
    public int AccentIndex { get; set; }
    public bool AutoLaunch { get; set; }
    public bool SilentDownload { get; set; }
    /// <summary>PDF 默认视图：<c>continuous</c> / <c>grid</c>；其它值按连续处理。</summary>
    public string PdfView { get; set; } = "continuous";
    /// <summary>PDF 默认缩放；0 表示「适应宽度」。</summary>
    public double PdfScale { get; set; }
    public string? DownloadDirectory { get; set; }
    /// <summary>下载目录的 FutureAccessList token；UWP 靠它跨启动访问用户选的目录。</summary>
    public string? DownloadFolderToken { get; set; }
    public string ServerUrl { get; set; } = Net.DatumHttp.DefaultBaseUrl;
    public bool AutoCheckUpdate { get; set; } = true;
    /// <summary>更新下载的镜像前缀（D12，默认关闭）；空表示直连 GitHub。</summary>
    public string? UpdateMirrorBase { get; set; }
    /// <summary>上次检查更新的 Unix 秒；0 表示没查过。启动时按 24 小时节流。</summary>
    public long UpdateCheckedAtSeconds { get; set; }

    public static AppSettings Load(string? json)
    {
        var settings = new AppSettings();
        if (string.IsNullOrWhiteSpace(json)) return settings;

        try
        {
            var root = JsonValue.Parse(json!);
            if (!root.IsObject) return settings;
            settings.ThemeMode = root.GetString("themeMode") switch
            {
                "light" => ThemeMode.Light,
                "dark" => ThemeMode.Dark,
                _ => ThemeMode.System,
            };
            settings.AccentIndex = root.GetInt("accentIndex");
            settings.AutoLaunch = root.GetBool("autoLaunch");
            settings.SilentDownload = root.GetBool("silentDownload");
            settings.PdfView = NormalizeView(root.GetString("pdfView"));
            settings.PdfScale = root.GetDouble("pdfScale", 0);
            settings.DownloadDirectory = root.GetString("downloadDirectory");
            var folderToken = root.GetString("downloadFolderToken");
            if (!string.IsNullOrWhiteSpace(folderToken)) settings.DownloadFolderToken = folderToken;
            var server = root.GetString("serverUrl");
            if (!string.IsNullOrWhiteSpace(server)) settings.ServerUrl = server!;
            settings.AutoCheckUpdate = root.GetBool("autoCheckUpdate", true);
            var mirror = root.GetString("updateMirrorBase");
            if (!string.IsNullOrWhiteSpace(mirror)) settings.UpdateMirrorBase = mirror;
            settings.UpdateCheckedAtSeconds = root.GetLong("updateCheckedAt");
        }
        catch (JsonParseException)
        {
        }

        return settings;
    }

    public string ToJson()
    {
        var builder = new JsonObjectBuilder()
            .String("themeMode", ThemeMode switch
            {
                ThemeMode.Light => "light",
                ThemeMode.Dark => "dark",
                _ => "system",
            })
            .Number("accentIndex", AccentIndex)
            .Bool("autoLaunch", AutoLaunch)
            .Bool("silentDownload", SilentDownload)
            .String("pdfView", PdfView)
            .Raw("pdfScale", PdfScale.ToString(System.Globalization.CultureInfo.InvariantCulture))
            .String("serverUrl", ServerUrl)
            .Bool("autoCheckUpdate", AutoCheckUpdate);

        if (UpdateMirrorBase == null) builder.Null("updateMirrorBase");
        else builder.String("updateMirrorBase", UpdateMirrorBase);

        builder.Number("updateCheckedAt", UpdateCheckedAtSeconds);

        if (DownloadFolderToken == null) builder.Null("downloadFolderToken");
        else builder.String("downloadFolderToken", DownloadFolderToken);

        if (DownloadDirectory == null) builder.Null("downloadDirectory");
        else builder.String("downloadDirectory", DownloadDirectory);

        return builder.ToString();
    }

    /// <summary>只认 <c>grid</c> 与 <c>continuous</c>；手改坏的配置回落到连续视图。</summary>
    public static string NormalizeView(string? value) =>
        string.Equals(value, "grid", System.StringComparison.OrdinalIgnoreCase) ? "grid" : "continuous";
}
