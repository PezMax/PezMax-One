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
    public double PdfScale { get; set; } = 1.0;
    public string? DownloadDirectory { get; set; }
    public string ServerUrl { get; set; } = Net.DatumHttp.DefaultBaseUrl;
    public bool AutoCheckUpdate { get; set; } = true;

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
            settings.PdfScale = root.GetDouble("pdfScale", 1.0);
            settings.DownloadDirectory = root.GetString("downloadDirectory");
            var server = root.GetString("serverUrl");
            if (!string.IsNullOrWhiteSpace(server)) settings.ServerUrl = server!;
            settings.AutoCheckUpdate = root.GetBool("autoCheckUpdate", true);
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
            .Raw("pdfScale", PdfScale.ToString(System.Globalization.CultureInfo.InvariantCulture))
            .String("serverUrl", ServerUrl)
            .Bool("autoCheckUpdate", AutoCheckUpdate);

        if (DownloadDirectory == null) builder.Null("downloadDirectory");
        else builder.String("downloadDirectory", DownloadDirectory);

        return builder.ToString();
    }
}
