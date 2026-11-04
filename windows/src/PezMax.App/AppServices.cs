using System;
using System.Threading.Tasks;
using PezMax.Core;
using PezMax.Core.Api;
using PezMax.Core.Models;
using PezMax.Core.Net;
using PezMax.Core.Platform;
using PezMax.Platform;
using Windows.ApplicationModel.Core;
using Windows.UI.Core;

namespace PezMax;

internal static class AppServices
{
    public static ISettingsStore SettingsStore { get; } = new LocalSettingsStore();
    public static IFileStore Files { get; } = new LocalFileStore();
    public static ICredentialStore Credentials { get; } = new PasswordVaultCredentialStore();

    private static DatumHttp? _http;
    private static AppSettings? _settings;
    private static UserInfo? _currentUser;

    public static DatumHttp Http =>
        _http ?? throw new InvalidOperationException("AppServices.InitializeAsync 尚未完成");

    public static AppSettings Settings =>
        _settings ?? throw new InvalidOperationException("AppServices.InitializeAsync 尚未完成");

    public static UserInfo? CurrentUser => _currentUser;
    public static long UserId => _currentUser?.UserId ?? 0;
    public static bool IsLoggedIn => Http.IsAuthenticated;
    public static bool PreviewMode { get; private set; }

    public static void EnterPreviewMode() => PreviewMode = true;

    public static AuthApi Auth => new(Http);
    public static ProfileApi Profile => new(Http);
    public static FilesApi FilesApi => new(Http);
    public static BookmarksApi Bookmarks => new(Http);
    public static FavoritesApi Favorites => new(Http);
    public static DownloadsApi Downloads => new(Http);
    public static ReportsApi Reports => new(Http);
    public static CommunityApi Community => new(Http);
    public static NotificationsApi Notifications => new(Http);
    public static EbooksApi Ebooks => new(Http);

    public static event Action? SessionChanged;
    public static event Action<string>? SessionExpired;

    public static async Task InitializeAsync()
    {
        _settings = AppSettings.Load(await Files.ReadTextAsync("settings.json").ConfigureAwait(false));
        _http = new DatumHttp(_settings.ServerUrl);
        _http.SessionExpired += () => HandleSessionExpired("登录已失效，请重新登录");
    }

    public static void ApplyTheme(Windows.UI.Xaml.FrameworkElement root)
    {
        root.RequestedTheme = Settings.ThemeMode switch
        {
            ThemeMode.Light => Windows.UI.Xaml.ElementTheme.Light,
            ThemeMode.Dark => Windows.UI.Xaml.ElementTheme.Dark,
            _ => Windows.UI.Xaml.ElementTheme.Default,
        };
    }

    public static async Task<bool> TryRestoreSessionAsync()
    {
        var token = Credentials.GetToken();
        if (string.IsNullOrWhiteSpace(token)) return false;
        Http.Token = token;
        try
        {
            var info = await Auth.GetInfoAsync().ConfigureAwait(false);
            _currentUser = info.User;
            return true;
        }
        catch (DatumException ex) when (ex.SessionExpired)
        {
            Http.Token = null;
            Credentials.ClearToken();
            return false;
        }
        catch (Exception)
        {
            return false;
        }
    }

    public static async Task CompleteLoginAsync(LoginResult login)
    {
        PreviewMode = false;
        Http.Token = login.Token;
        Credentials.SetToken(login.Token);
        var info = await Auth.GetInfoAsync().ConfigureAwait(false);
        _currentUser = info.User;
        RaiseSessionChanged();
    }

    public static async Task RefreshUserAsync()
    {
        var info = await Auth.GetInfoAsync().ConfigureAwait(false);
        _currentUser = info.User;
        RaiseSessionChanged();
    }

    public static async Task SignOutAsync()
    {
        try
        {
            if (Http.IsAuthenticated) await Auth.LogoutAsync().ConfigureAwait(false);
        }
        catch (Exception)
        {
        }

        Http.Token = null;
        Credentials.ClearToken();
        _currentUser = null;
        RaiseSessionChanged();
    }

    public static void HandleSessionExpired(string message)
    {
        Http.Token = null;
        Credentials.ClearToken();
        _currentUser = null;
        RunOnUi(() =>
        {
            SessionExpired?.Invoke(message);
            SessionChanged?.Invoke();
        });
    }

    public static void RaiseSessionChanged()
    {
        RunOnUi(() => SessionChanged?.Invoke());
    }

    public static async Task SaveSettingsAsync()
    {
        await Files.WriteTextAsync("settings.json", Settings.ToJson()).ConfigureAwait(false);
    }

    public static void RunOnUi(Action action)
    {
        CoreDispatcher? dispatcher;
        try
        {
            dispatcher = CoreApplication.MainView?.CoreWindow?.Dispatcher
                ?? CoreApplication.GetCurrentView()?.CoreWindow?.Dispatcher;
        }
        catch
        {
            dispatcher = null;
        }

        if (dispatcher == null || dispatcher.HasThreadAccess)
        {
            action();
            return;
        }

        _ = dispatcher.RunAsync(CoreDispatcherPriority.Normal, () => action());
    }
}
