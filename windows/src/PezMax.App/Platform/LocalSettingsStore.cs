using PezMax.Core.Platform;
using Windows.Storage;

namespace PezMax.Platform;

public sealed class LocalSettingsStore : ISettingsStore
{
    private static ApplicationDataContainer Settings => ApplicationData.Current.LocalSettings;

    public string? GetString(string key) =>
        Settings.Values.TryGetValue(key, out var value) && value is string text ? text : null;

    public void SetString(string key, string? value)
    {
        if (value == null) Settings.Values.Remove(key);
        else Settings.Values[key] = value;
    }

    public int GetInt(string key, int fallback) =>
        Settings.Values.TryGetValue(key, out var value) && value is int number ? number : fallback;

    public void SetInt(string key, int value) => Settings.Values[key] = value;

    public bool GetBool(string key, bool fallback) =>
        Settings.Values.TryGetValue(key, out var value) && value is bool flag ? flag : fallback;

    public void SetBool(string key, bool value) => Settings.Values[key] = value;
}
