using System;
using System.Threading.Tasks;
using Windows.Storage;
using Windows.Storage.AccessCache;
using Windows.Storage.Pickers;

namespace PezMax.Transfer
{
    /// <summary>
    /// 下载目录的解析与授权。
    /// <para>
    /// 默认目录是「下载\PezMax」：`DownloadsFolder` 不需要用户授权就能建子目录。
    /// 用户改目录时用 `FolderPicker` 选一次，把 `FutureAccessList` 的 token 存进设置——
    /// UWP 的 AppContainer 不能凭路径访问任意目录，**只有 token 能跨启动授权**。
    /// </para>
    /// <para>
    /// 按 `windows/AGENTS.md` 的约定：不申请 `broadFileSystemAccess`（受限能力，且系统设置里默认关闭）。
    /// </para>
    /// </summary>
    internal static class DownloadStorage
    {
        public const string DefaultFolderName = "PezMax";

        /// <summary>最近一次解析出来的真实路径（含回落），给设置页显示用。</summary>
        public static string? LastResolvedPath { get; private set; }

        /// <summary>当前可写的目录；授权失效时自动回落到「下载\PezMax」。</summary>
        public static async Task<StorageFolder> ResolveAsync()
        {
            var folder = await ResolveCoreAsync().ConfigureAwait(false);
            LastResolvedPath = folder.Path;
            return folder;
        }

        private static async Task<StorageFolder> ResolveCoreAsync()
        {
            var settings = AppServices.Settings;

            // 1. 用户选过的目录：先试 token（唯一有权限的方式），再试路径（同进程内可能仍有效）。
            if (!string.IsNullOrWhiteSpace(settings.DownloadFolderToken))
            {
                try
                {
                    return await StorageApplicationPermissions.FutureAccessList
                        .GetFolderAsync(settings.DownloadFolderToken!);
                }
                catch (Exception)
                {
                    // token 失效（目录被删 / 权限被撤）：清掉它，别每下载一次都抛。
                    settings.DownloadFolderToken = null;
                }
            }

            if (!string.IsNullOrWhiteSpace(settings.DownloadDirectory))
            {
                try
                {
                    return await StorageFolder.GetFolderFromPathAsync(settings.DownloadDirectory!);
                }
                catch (Exception)
                {
                    settings.DownloadDirectory = null;
                }
            }

            // 2. 默认：下载\PezMax（不需要用户授权，只要应用有 downloadsFolder 能力）。
            //    个别环境里 DownloadsFolder 会抛 UnauthorizedAccessException（能力或组策略），
            //    这时退到应用缓存目录——能下总比下不了好，界面上的路径会如实显示。
            try
            {
                return await Windows.Storage.DownloadsFolder
                    .CreateFolderAsync(DefaultFolderName, CreationCollisionOption.OpenIfExists);
            }
            catch (UnauthorizedAccessException)
            {
                return await ApplicationData.Current.LocalCacheFolder
                    .CreateFolderAsync(DefaultFolderName, CreationCollisionOption.OpenIfExists);
            }
        }

        /// <summary>让用户选一个目录并记住授权；返回 null 表示用户取消。</summary>
        public static async Task<StorageFolder?> PickAsync()
        {
            var picker = new FolderPicker
            {
                SuggestedStartLocation = PickerLocationId.Downloads,
            };
            picker.FileTypeFilter.Add("*");

            var folder = await picker.PickSingleFolderAsync();
            if (folder == null) return null;

            var token = StorageApplicationPermissions.FutureAccessList.Add(folder);
            AppServices.Settings.DownloadFolderToken = token;
            AppServices.Settings.DownloadDirectory = folder.Path;
            await AppServices.SaveSettingsAsync();
            return folder;
        }

        /// <summary>恢复默认（「下载\PezMax」），并释放之前的授权。</summary>
        public static async Task ResetAsync()
        {
            var settings = AppServices.Settings;
            if (!string.IsNullOrWhiteSpace(settings.DownloadFolderToken))
            {
                try
                {
                    StorageApplicationPermissions.FutureAccessList.Remove(settings.DownloadFolderToken!);
                }
                catch (Exception)
                {
                }
            }

            settings.DownloadFolderToken = null;
            settings.DownloadDirectory = null;
            await AppServices.SaveSettingsAsync();
        }

        public static string Describe()
        {
            if (!string.IsNullOrWhiteSpace(LastResolvedPath)) return LastResolvedPath!;

            var settings = AppServices.Settings;
            return string.IsNullOrWhiteSpace(settings.DownloadDirectory)
                ? "下载\\" + DefaultFolderName
                : settings.DownloadDirectory!;
        }
    }
}
