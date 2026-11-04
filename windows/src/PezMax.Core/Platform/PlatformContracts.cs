using System;
using System.Threading.Tasks;

namespace PezMax.Core.Platform;

public interface ISettingsStore
{
    string? GetString(string key);
    void SetString(string key, string? value);
    int GetInt(string key, int fallback);
    void SetInt(string key, int value);
    bool GetBool(string key, bool fallback);
    void SetBool(string key, bool value);
}

public interface ICredentialStore
{
    string? GetToken();
    void SetToken(string token);
    void ClearToken();
}

public interface IFileStore
{
    Task<string?> ReadTextAsync(string relativePath);
    Task WriteTextAsync(string relativePath, string content);
    Task DeleteAsync(string relativePath);
    Task<bool> ExistsAsync(string relativePath);
}

public interface IClock
{
    DateTimeOffset Now { get; }
}

public sealed class SystemClock : IClock
{
    public DateTimeOffset Now => DateTimeOffset.Now;
}
