using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using PezMax.Core;
using PezMax.Core.Models;
using PezMax.Core.Net;
using PezMax.Core.Urls;
using PezMax.Login;
using PezMax.Transfer;
using Windows.Storage;
using Windows.Storage.Pickers;
using Windows.System;
using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Media.Imaging;
using Windows.UI.Xaml.Navigation;
using muxc = Microsoft.UI.Xaml.Controls;

namespace PezMax.Pages
{
    public sealed partial class ProfilePage : Page
    {
        private readonly ObservableCollection<NotificationItem> _notices = new();
        private readonly ObservableCollection<DownloadEntry> _downloads = new();
        private bool _loaded;

        public ProfilePage()
        {
            InitializeComponent();
            NoticeList.ItemsSource = _notices;
            DownloadList.ItemsSource = _downloads;
            var version = Windows.ApplicationModel.Package.Current.Id.Version;
            AboutText.Text = $"PezMax One {version.Major}.{version.Minor}.{version.Build} · Windows";
        }

        protected override async void OnNavigatedTo(NavigationEventArgs e)
        {
            base.OnNavigatedTo(e);
            if (e.Parameter is string target)
            {
                Tabs.SelectedIndex = target switch
                {
                    "notifications" => 1,
                    "downloads" => 2,
                    "settings" => 3,
                    _ => 0,
                };
            }

            if (_loaded) return;
            _loaded = true;
            PopulateSettings();
            await RefreshProfileAsync();
            await LoadNotificationsAsync();
            await LoadDownloadsAsync();
        }

        private async Task RefreshProfileAsync()
        {
            try
            {
                if (AppServices.PreviewMode)
                {
                    ProfileName.Text = "预览用户";
                    ProfileMeta.Text = "ID 7 · 上传 6 份 · 状态 正常";
                    StatUpload.Text = "6";
                    StatDownload.Text = "42";
                    StatFavorite.Text = "9";
                    AvatarImage.Source = null;
                    return;
                }

                var profile = await AppServices.Profile.GetProfileAsync();
                var stats = await AppServices.Profile.GetStatsAsync();
                ProfileName.Text = profile.DisplayName;
                ProfileMeta.Text = $"ID {profile.UserId} · 上传 {profile.Count} 份 · 状态 {(profile.Status == "0" ? "已停用" : "正常")}";
                StatUpload.Text = stats.UploadCount.ToString();
                StatDownload.Text = stats.DownloadCount.ToString();
                StatFavorite.Text = stats.FavoriteCount.ToString();
                if (!string.IsNullOrWhiteSpace(profile.Avatar))
                {
                    AvatarImage.Source = new BitmapImage(new Uri(UrlNormalizer.Normalize(profile.Avatar, AppServices.Settings.ServerUrl)));
                }
                else
                {
                    AvatarImage.Source = null;
                }
            }
            catch (Exception ex)
            {
                ShowProfileError(ex is DatumException datum ? datum.Message : "个人资料加载失败");
            }
        }

        private void ShowProfileError(string message)
        {
            ProfileBar.Severity = muxc.InfoBarSeverity.Error;
            ProfileBar.Message = message;
            ProfileBar.IsOpen = true;
        }

        private void ShowProfileSuccess(string message)
        {
            ProfileBar.Severity = muxc.InfoBarSeverity.Success;
            ProfileBar.Message = message;
            ProfileBar.IsOpen = true;
        }

        private async void ChangeAvatarClick(object sender, RoutedEventArgs e)
        {
            var picker = new FileOpenPicker();
            picker.FileTypeFilter.Add(".jpg");
            picker.FileTypeFilter.Add(".jpeg");
            picker.FileTypeFilter.Add(".png");
            picker.FileTypeFilter.Add(".gif");
            var file = await picker.PickSingleFileAsync();
            if (file == null) return;

            try
            {
                using var stream = (await file.OpenAsync(FileAccessMode.Read)).AsStreamForRead();
                await AppServices.Profile.UploadAvatarAsync(
                    file.Name, stream, string.IsNullOrWhiteSpace(file.ContentType) ? "image/jpeg" : file.ContentType);
                await AppServices.RefreshUserAsync();
                await RefreshProfileAsync();
                ShowProfileSuccess("头像已更新");
            }
            catch (Exception ex)
            {
                ShowProfileError(ex is DatumException datum ? datum.Message : "头像上传失败");
            }
        }

        private async void ChangeNameClick(object sender, RoutedEventArgs e)
        {
            var box = new TextBox { Header = "新用户名", Text = AppServices.CurrentUser?.UserName ?? "", CornerRadius = new CornerRadius(0) };
            var dialog = new PezDialog
            {
                Title = "修改用户名",
                Content = box,
                PrimaryButtonText = "保存",
                CloseButtonText = "取消",
                DefaultButton = ContentDialogButton.Primary,
            };
            if (await dialog.ShowAsync() != ContentDialogResult.Primary) return;

            try
            {
                await AppServices.Profile.UpdateUserNameAsync(box.Text);
                await AppServices.RefreshUserAsync();
                await RefreshProfileAsync();
                ShowProfileSuccess("用户名已更新");
            }
            catch (Exception ex)
            {
                ShowProfileError(ex is DatumException datum ? datum.Message : "用户名修改失败");
            }
        }

        private async void ChangePasswordClick(object sender, RoutedEventArgs e)
        {
            var oldBox = new PasswordBox { Header = "当前密码", CornerRadius = new CornerRadius(0) };
            var newBox = new PasswordBox { Header = "新密码", CornerRadius = new CornerRadius(0) };
            var confirmBox = new PasswordBox { Header = "确认新密码", CornerRadius = new CornerRadius(0) };
            var panel = new StackPanel { Spacing = 6, Width = 360 };
            panel.Children.Add(oldBox);
            panel.Children.Add(newBox);
            panel.Children.Add(confirmBox);
            var dialog = new PezDialog
            {
                Title = "修改密码",
                Content = panel,
                PrimaryButtonText = "保存",
                CloseButtonText = "取消",
                DefaultButton = ContentDialogButton.Primary,
            };
            if (await dialog.ShowAsync() != ContentDialogResult.Primary) return;
            if (newBox.Password != confirmBox.Password)
            {
                ShowProfileError("两次输入的新密码不一致");
                return;
            }

            try
            {
                await AppServices.Profile.UpdatePasswordAsync(oldBox.Password, newBox.Password);
                await ForceReloginAsync("密码已修改，请重新登录");
            }
            catch (Exception ex)
            {
                ShowProfileError(ex is DatumException datum ? datum.Message : "密码修改失败");
            }
        }

        private async void ChangeSecurityClick(object sender, RoutedEventArgs e)
        {
            IReadOnlyList<string> questions;
            try
            {
                questions = await AppServices.Profile.GetSecurityQuestionsAsync();
            }
            catch (Exception ex)
            {
                ShowProfileError(ex is DatumException datum ? datum.Message : "密保问题加载失败");
                return;
            }

            var passwordBox = new PasswordBox { Header = "当前密码", CornerRadius = new CornerRadius(0) };
            var questionBoxes = new List<TextBox>();
            var answerBoxes = new List<TextBox>();
            var panel = new StackPanel { Spacing = 6, Width = 440 };
            panel.Children.Add(passwordBox);
            for (var i = 0; i < 3; i++)
            {
                var question = new TextBox { Header = $"密保问题 {i + 1}", Text = i < questions.Count ? questions[i] : "", CornerRadius = new CornerRadius(0) };
                var answer = new TextBox { Header = $"答案 {i + 1}", CornerRadius = new CornerRadius(0) };
                questionBoxes.Add(question);
                answerBoxes.Add(answer);
                panel.Children.Add(question);
                panel.Children.Add(answer);
            }

            var dialog = new PezDialog
            {
                Title = "密保设置",
                Content = new ScrollViewer { Content = panel, MaxHeight = 560 },
                PrimaryButtonText = "保存",
                CloseButtonText = "取消",
                DefaultButton = ContentDialogButton.Primary,
            };
            if (await dialog.ShowAsync() != ContentDialogResult.Primary) return;

            try
            {
                await AppServices.Profile.UpdateSecurityAsync(
                    passwordBox.Password,
                    new PezMax.Core.Api.SecurityQuestionSet
                    {
                        One = questionBoxes[0].Text,
                        Two = questionBoxes[1].Text,
                        Three = questionBoxes[2].Text,
                    },
                    new PezMax.Core.Api.SecurityAnswerSet
                    {
                        One = answerBoxes[0].Text,
                        Two = answerBoxes[1].Text,
                        Three = answerBoxes[2].Text,
                    });
                ShowProfileSuccess("密保已更新");
            }
            catch (Exception ex)
            {
                ShowProfileError(ex is DatumException datum ? datum.Message : "密保更新失败");
            }
        }

        private async void SignOutClick(object sender, RoutedEventArgs e)
        {
            var dialog = new PezDialog
            {
                Title = "退出登录",
                Content = "确定退出当前账号吗？",
                PrimaryButtonText = "退出",
                CloseButtonText = "取消",
                DefaultButton = ContentDialogButton.Close,
            };
            if (await dialog.ShowAsync() != ContentDialogResult.Primary) return;
            await ForceReloginAsync(null);
        }

        private async Task ForceReloginAsync(string? message)
        {
            await AppServices.SignOutAsync();
            if (message != null)
            {
                await new PezDialog
                {
                    Title = "提示",
                    Content = message,
                    CloseButtonText = "知道了",
                    DefaultButton = ContentDialogButton.Close,
                }.ShowAsync();
            }

            Frame.Navigate(typeof(LoginPage));
            Frame.BackStack.Clear();
        }

        private async Task LoadNotificationsAsync()
        {
            if (AppServices.PreviewMode) return;
            NoticeProgress.IsActive = true;
            NoticeProgress.Visibility = Visibility.Visible;
            try
            {
                var page = await AppServices.Notifications.GetListAsync(1, 100);
                _notices.Clear();
                foreach (var item in page.Rows) _notices.Add(item);
            }
            catch (Exception ex)
            {
                ShowProfileError(ex is DatumException datum ? datum.Message : "通知加载失败");
            }
            finally
            {
                NoticeProgress.IsActive = false;
                NoticeProgress.Visibility = Visibility.Collapsed;
            }
        }

        private async void RefreshNotificationsClick(object sender, RoutedEventArgs e) => await LoadNotificationsAsync();

        private async void NoticeClick(object sender, ItemClickEventArgs e)
        {
            if (e.ClickedItem is not NotificationItem item) return;
            await new PezDialog
            {
                Title = item.Title,
                Content = new ScrollViewer
                {
                    MaxHeight = 480,
                    Content = new TextBlock
                    {
                        Text = item.Content,
                        TextWrapping = TextWrapping.Wrap,
                        IsTextSelectionEnabled = true,
                    },
                },
                CloseButtonText = "关闭",
                DefaultButton = ContentDialogButton.Close,
            }.ShowAsync();
        }

        private async Task LoadDownloadsAsync()
        {
            if (AppServices.PreviewMode) return;
            DownloadProgress.IsActive = true;
            DownloadProgress.Visibility = Visibility.Visible;
            try
            {
                var page = await AppServices.Downloads.GetMyDownloadsAsync(AppServices.UserId, 1, 100);
                _downloads.Clear();
                foreach (var item in page.Rows) _downloads.Add(item);
            }
            catch (Exception ex)
            {
                ShowProfileError(ex is DatumException datum ? datum.Message : "下载记录加载失败");
            }
            finally
            {
                DownloadProgress.IsActive = false;
                DownloadProgress.Visibility = Visibility.Collapsed;
            }
        }

        private async void RefreshDownloadsClick(object sender, RoutedEventArgs e) => await LoadDownloadsAsync();

        private async void RemoveDownloadClick(object sender, RoutedEventArgs e)
        {
            if (DownloadList.SelectedItem is not DownloadEntry entry) return;
            try
            {
                await AppServices.Downloads.RemoveAsync(AppServices.UserId, entry.File.FileId);
                _downloads.Remove(entry);
            }
            catch (Exception ex)
            {
                ShowProfileError(ex is DatumException datum ? datum.Message : "删除下载记录失败");
            }
        }

        private async void RedownloadClick(object sender, RoutedEventArgs e)
        {
            if (DownloadList.SelectedItem is not DownloadEntry entry) return;
            try
            {
                var file = await DownloadService.DownloadFileAsync(entry.File);
                await Launcher.LaunchFileAsync(file);
            }
            catch (Exception ex)
            {
                ShowProfileError(ex is DatumException datum ? datum.Message : "下载失败");
            }
        }

        private void PopulateSettings()
        {
            var settings = AppServices.Settings;
            ThemeBox.SelectedIndex = settings.ThemeMode switch
            {
                ThemeMode.Light => 1,
                ThemeMode.Dark => 2,
                _ => 0,
            };
            DownloadDirBox.Text = settings.DownloadDirectory ?? "";
            SilentDownloadSwitch.IsOn = settings.SilentDownload;
            AutoUpdateSwitch.IsOn = settings.AutoCheckUpdate;
            ServerBox.Text = settings.ServerUrl;
        }

        private async void SaveSettingsClick(object sender, RoutedEventArgs e)
        {
            var server = (ServerBox.Text ?? "").Trim();
            if (!Uri.TryCreate(server, UriKind.Absolute, out var uri) ||
                (uri.Scheme != Uri.UriSchemeHttp && uri.Scheme != Uri.UriSchemeHttps))
            {
                SettingsBar.Severity = muxc.InfoBarSeverity.Error;
                SettingsBar.Message = "服务器地址必须是 http:// 或 https:// 开头的完整地址";
                SettingsBar.IsOpen = true;
                return;
            }

            var previousServer = AppServices.Settings.ServerUrl;
            var settings = AppServices.Settings;
            settings.ThemeMode = ThemeBox.SelectedIndex switch
            {
                1 => ThemeMode.Light,
                2 => ThemeMode.Dark,
                _ => ThemeMode.System,
            };
            settings.DownloadDirectory = string.IsNullOrWhiteSpace(DownloadDirBox.Text) ? null : DownloadDirBox.Text.Trim();
            settings.SilentDownload = SilentDownloadSwitch.IsOn;
            settings.AutoCheckUpdate = AutoUpdateSwitch.IsOn;
            settings.ServerUrl = DatumHttp.NormalizeBaseUrl(server);
            await AppServices.SaveSettingsAsync();

            var root = Window.Current.Content as FrameworkElement;
            if (root != null) AppServices.ApplyTheme(root);

            if (!string.Equals(previousServer, settings.ServerUrl, StringComparison.OrdinalIgnoreCase))
            {
                AppServices.Http.BaseUrl = settings.ServerUrl;
                await ForceReloginAsync("服务器地址已修改，请重新登录");
                return;
            }

            SettingsBar.Severity = muxc.InfoBarSeverity.Success;
            SettingsBar.Message = "设置已保存";
            SettingsBar.IsOpen = true;
        }

        private async void ClearCacheClick(object sender, RoutedEventArgs e)
        {
            try
            {
                var cache = ApplicationData.Current.LocalCacheFolder;
                foreach (var item in await cache.GetItemsAsync())
                {
                    await item.DeleteAsync(StorageDeleteOption.PermanentDelete);
                }
                SettingsBar.Severity = muxc.InfoBarSeverity.Success;
                SettingsBar.Message = "缓存已清除（登录态与下载记录不受影响）";
                SettingsBar.IsOpen = true;
            }
            catch (Exception ex)
            {
                SettingsBar.Severity = muxc.InfoBarSeverity.Error;
                SettingsBar.Message = ex.Message;
                SettingsBar.IsOpen = true;
            }
        }

        private async void GitHubClick(object sender, RoutedEventArgs e) =>
            await Launcher.LaunchUriAsync(new Uri("https://github.com/PezMax/PezMax-One"));

        private async void OpenThemeSettingsClick(object sender, RoutedEventArgs e) =>
            await Launcher.LaunchUriAsync(new Uri("ms-settings:colors"));
    }
}
