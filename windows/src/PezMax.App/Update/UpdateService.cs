using System;
using System.IO;
using System.Net.Http;
using System.Threading.Tasks;
using PezMax.Core;
using PezMax.Core.Update;
using Windows.System;

namespace PezMax.Update
{
    /// <summary>
    /// 应用内更新：取 releases → 交给 Core 的 <see cref="UpdateChecker"/> 挑版本 → 交给系统装。
    /// 判定（资产名、版本比较、镜像前缀）全在 Core，跑 `spec/fixtures/update/`。
    /// </summary>
    internal static class UpdateService
    {
        private static readonly HttpClient Client = new() { Timeout = TimeSpan.FromSeconds(15) };

        public static string CurrentVersion
        {
            get
            {
                var version = Windows.ApplicationModel.Package.Current.Id.Version;
                return $"{version.Major}.{version.Minor}.{version.Build}";
            }
        }

        /// <summary>检查更新；有新版返回选择结果，没有返回 null。失败也返回 null（调用方按「已是最新」处理）。</summary>
        public static async Task<UpdateChoice?> CheckAsync(string? mirrorBase = null)
        {
            try
            {
                var releases = await ReleaseFeed.FetchAsync(Client).ConfigureAwait(false);
                if (releases.Count == 0) return null;

                return UpdateChecker.Find(
                    CurrentVersion,
                    "windows",
                    "x86_64",
                    releases,
                    mirrorBase);
            }
            catch (Exception ex)
            {
                App.WriteCrashLog(ex);
                return null;
            }
        }

        /// <summary>启动时的自动检查：24 小时内最多一次（`docs/RELEASES.md`）。</summary>
        public static async Task<UpdateChoice?> CheckIfDueAsync(AppSettings settings)
        {
            if (!settings.AutoCheckUpdate) return null;

            var now = DateTimeOffset.UtcNow.ToUnixTimeSeconds();
            if (now - settings.UpdateCheckedAtSeconds < 24 * 60 * 60) return null;

            settings.UpdateCheckedAtSeconds = now;
            await AppServices.SaveSettingsAsync().ConfigureAwait(false);

            return await CheckAsync(settings.UpdateMirrorBase).ConfigureAwait(false);
        }

        /// <summary>
        /// Windows 的安装动作：把 `.appinstaller` 交给系统 App Installer（UWP 不能静默自装）。
        /// 没有 `.appinstaller` 就退回 Release 页面，别让用户卡在「有个版本但你装不了」。
        /// </summary>
        public static async Task<bool> InstallAsync(UpdateChoice choice)
        {
            choice.Assets.TryGetValue("appinstaller", out var appInstaller);
            var target = string.IsNullOrEmpty(appInstaller) ? choice.ReleasePageUrl : appInstaller;
            if (string.IsNullOrEmpty(target)) return false;

            if (!Uri.TryCreate(target, UriKind.Absolute, out var uri)) return false;
            return await Launcher.LaunchUriAsync(uri);
        }

        public static string Describe(UpdateChoice choice) =>
            "新版本 " + choice.Version + "（" + choice.ReleaseTag + "）";
    }
}
