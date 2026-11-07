using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using PezMax.Core.Models;
using PezMax.Core.Notify;
using PezMax.Login;
using PezMax.Pages;
using PezMax.Update;
using Windows.ApplicationModel.Core;
using Windows.UI.ViewManagement;
using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Navigation;
using muxc = Microsoft.UI.Xaml.Controls;

namespace PezMax.Shell
{
    public sealed partial class ShellPage : Page
    {
        private string _lastSearch = "";
        private readonly NotificationCenter _notifications;
        private IReadOnlyList<NotificationItem> _scrollQueue = Array.Empty<NotificationItem>();
        private int _scrollIndex;
        private bool _noticeBusy;

        public ShellPage()
        {
            InitializeComponent();
            _notifications = new NotificationCenter(
                relativePath => AppServices.Files.ReadTextAsync(relativePath),
                content => AppServices.Files.WriteTextAsync(NoticeStorage.FileName, content));
            SetUpTitleBar();
            AppServices.SessionChanged += UpdateAccount;
            AppServices.SessionExpired += OnSessionExpired;
            HookDownloadCenter();
            Loaded += OnLoaded;
        }

        private async void OnLoaded(object sender, RoutedEventArgs e)
        {
            UpdateAccount();
            if (AppServices.PreviewMode)
            {
                SessionBar.Severity = muxc.InfoBarSeverity.Informational;
                SessionBar.Message = "预览模式：未连接后端，页面数据为演示内容";
                SessionBar.IsOpen = true;
            }

            if (ContentFrame.Content == null)
            {
                ContentFrame.Navigate(typeof(HomePage));
            }

            await RefreshNotificationsAsync();
            await CheckUpdateAsync();
        }

        private PezMax.Core.Update.UpdateChoice? _pendingUpdate;

        /// <summary>下载浮层：订阅 DownloadCenter 的事件，串行队列里同时只会有一个在跑。</summary>
        private void HookDownloadCenter()
        {
            Transfer.DownloadCenter.Started += fileName => RunOnUi(() =>
            {
                DownloadFileText.Text = "正在下载 " + fileName;
                DownloadBar.Value = 0;
                DownloadHost.Visibility = Visibility.Visible;
            });

            Transfer.DownloadCenter.Progress += report => RunOnUi(() =>
            {
                DownloadFileText.Text = "正在下载 " + report.FileName + "（" +
                                        FileItem.FormatSize(report.BytesReceived) + " / " +
                                        FileItem.FormatSize(report.Total) + "）";
                DownloadBar.Value = report.Fraction;
            });

            Transfer.DownloadCenter.Completed += _ => RunOnUi(() =>
            {
                DownloadBar.Value = 1;
                DownloadHost.Visibility = Visibility.Collapsed;
            });

            Transfer.DownloadCenter.Failed += (_, message) => RunOnUi(() =>
            {
                DownloadHost.Visibility = Visibility.Collapsed;
                SessionBar.Severity = muxc.InfoBarSeverity.Warning;
                SessionBar.Message = message;
                SessionBar.IsOpen = true;
            });
        }

        private static void RunOnUi(Action action) => AppServices.RunOnUi(action);

        private void DownloadCancelClick(object sender, RoutedEventArgs e)
        {
            Transfer.DownloadCenter.Cancel();
            DownloadHost.Visibility = Visibility.Collapsed;
        }

        /// <summary>启动后查一次更新（24 小时节流），有新版只在顶部挂一条提示，不弹模态框。</summary>
        private async Task CheckUpdateAsync()
        {
            try
            {
                _pendingUpdate = await UpdateService.CheckIfDueAsync(AppServices.Settings);
                if (_pendingUpdate == null) return;

                UpdateBar.Message = "有新版本 " + _pendingUpdate.Version + "（" + _pendingUpdate.ReleaseTag + "），可以在「设置 → 关于」里安装。";
                UpdateBar.IsOpen = true;
            }
            catch (Exception ex)
            {
                App.WriteCrashLog(ex);
            }
        }

        private async void UpdateInstallClick(object sender, RoutedEventArgs e)
        {
            if (_pendingUpdate == null) return;

            UpdateBar.IsOpen = false;
            try
            {
                await UpdateService.InstallAsync(_pendingUpdate);
            }
            catch (Exception ex)
            {
                App.WriteCrashLog(ex);
                SessionBar.Message = "打开安装程序失败，请到 Release 页面手动下载";
                SessionBar.IsOpen = true;
            }
        }

        /// <summary>
        /// 取两条 feed、刷滚动条、按队列弹窗。通知取不到不报错——外壳不该因为通知挂掉。
        /// 首屏渲染完再查（`docs/RELEASES.md` 对更新检查的要求同款）。
        /// </summary>
        private async Task RefreshNotificationsAsync()
        {
            if (!AppServices.IsLoggedIn || AppServices.UserId <= 0) return;

            try
            {
                await _notifications.LoadAsync(AppServices.UserId);
                ShowScrollNotice();
                UpdateBell();
                await RunPopupQueueAsync();
            }
            catch (Exception ex)
            {
                App.WriteCrashLog(ex);
            }
        }

        private void ShowScrollNotice()
        {
            _scrollQueue = _notifications.PlanScroll();
            _scrollIndex = 0;
            if (_scrollQueue.Count == 0)
            {
                ScrollNoticeHost.Visibility = Visibility.Collapsed;
                return;
            }

            var item = _scrollQueue[_scrollIndex];
            ScrollNoticeText.Text = string.IsNullOrWhiteSpace(item.Content)
                ? item.Title
                : item.Title + "：" + item.Content;
            ScrollNoticeHost.Visibility = Visibility.Visible;
        }

        private async void ScrollNoticeNextClick(object sender, RoutedEventArgs e)
        {
            if (_scrollQueue.Count == 0) return;
            _scrollIndex = (_scrollIndex + 1) % _scrollQueue.Count;
            var item = _scrollQueue[_scrollIndex];
            ScrollNoticeText.Text = string.IsNullOrWhiteSpace(item.Content) ? item.Title : item.Title + "：" + item.Content;
            await _notifications.MarkReadAsync(item);
            UpdateBell();
        }

        private async void ScrollNoticeCloseClick(object sender, RoutedEventArgs e)
        {
            ScrollNoticeHost.Visibility = Visibility.Collapsed;
            if (_scrollQueue.Count > 0)
            {
                await _notifications.MarkDismissedAsync(_scrollQueue[_scrollIndex]);
                UpdateBell();
            }
        }

        /// <summary>弹窗队列：一次一条，确认后记进「已弹过」（type 1/4）与已读。</summary>
        private async Task RunPopupQueueAsync()
        {
            if (_noticeBusy) return;

            var queue = _notifications.PlanPopup();
            if (queue.Count == 0) return;

            _noticeBusy = true;
            try
            {
                foreach (var item in queue)
                {
                    var dismissible = NotificationPolicy.IsDismissible(item, DateTimeOffset.Now);
                    var dialog = new PezDialog
                    {
                        Title = item.Title,
                        Content = string.IsNullOrWhiteSpace(item.Content) ? item.TypeName : item.Content,
                        CloseButtonText = dismissible ? "知道了" : null,
                        PrimaryButtonText = dismissible ? null : "我知道了",
                        DefaultButton = ContentDialogButton.Close,
                    };

                    await dialog.ShowAsync();
                    await _notifications.MarkDismissedAsync(item);
                }
            }
            finally
            {
                _noticeBusy = false;
                UpdateBell();
            }
        }

        private void UpdateBell()
        {
            var unread = _notifications.UnreadCount;
            NoticeBadge.Value = unread;
            NoticeBadge.Visibility = unread > 0 ? Visibility.Visible : Visibility.Collapsed;
        }

        private async void NoticeBellClick(object sender, RoutedEventArgs e)
        {
            if (!AppServices.IsLoggedIn || AppServices.UserId <= 0)
            {
                SessionBar.Message = "登录后才能查看通知";
                SessionBar.IsOpen = true;
                return;
            }

            var items = _notifications.PopupItems
                .Concat(_notifications.PlanScroll())
                .GroupBy(item => item.NotifyId)
                .Select(group => group.First())
                .ToList();

            if (items.Count == 0)
            {
                await new PezDialog
                {
                    Title = "通知",
                    Content = "目前没有通知。",
                    CloseButtonText = "好",
                }.ShowAsync();
                return;
            }

            var panel = new StackPanel { Spacing = 10 };
            foreach (var item in items)
            {
                var unread = !_notifications.IsRead(item.NotifyId);
                panel.Children.Add(new TextBlock
                {
                    Text = (unread ? "● " : "　") + item.Title,
                    Style = (Style)Application.Current.Resources["PezBodyStyle"],
                });
                panel.Children.Add(new TextBlock
                {
                    Text = item.Content.Length > 120 ? item.Content.Substring(0, 120) + "…" : item.Content,
                    Style = (Style)Application.Current.Resources["PezCaptionStyle"],
                });
            }

            var dialog = new PezDialog
            {
                Title = "通知",
                Content = new ScrollViewer { Content = panel, MaxHeight = 420 },
                CloseButtonText = "关闭",
            };
            await dialog.ShowAsync();

            // 打开过就全标已读：铃铛的未读计数是「有没有看过」，不是「逐条点过」。
            foreach (var item in items) await _notifications.MarkReadAsync(item);
            UpdateBell();
        }

        private void SetUpTitleBar()
        {
            try
            {
                CoreApplication.GetCurrentView().TitleBar.ExtendViewIntoTitleBar = true;
                var titleBar = ApplicationView.GetForCurrentView().TitleBar;
                titleBar.ButtonBackgroundColor = Windows.UI.Colors.Transparent;
                titleBar.ButtonInactiveBackgroundColor = Windows.UI.Colors.Transparent;
                Window.Current.SetTitleBar(AppTitleBar);
            }
            catch (Exception ex)
            {
                App.WriteCrashLog(ex);
            }
        }

        private void NavItemInvoked(muxc.NavigationView sender, muxc.NavigationViewItemInvokedEventArgs args)
        {
            if (args.IsSettingsInvoked)
            {
                Navigate(typeof(ProfilePage), "settings");
                return;
            }

            switch (args.InvokedItemContainer?.Tag as string)
            {
                case "home":
                    Navigate(typeof(HomePage), null);
                    break;
                case "browse":
                    Navigate(typeof(BrowsePage), _lastSearch);
                    break;
                case "community":
                    Navigate(typeof(CommunityPage), null);
                    break;
                case "upload":
                    Navigate(typeof(UploadPage), null);
                    break;
                case "account":
                    Navigate(typeof(ProfilePage), "profile");
                    break;
            }
        }

        private void Navigate(Type page, object? parameter)
        {
            if (ContentFrame.CurrentSourcePageType == page && parameter == null) return;
            ContentFrame.Navigate(page, parameter);
        }

        private void SearchClick(object sender, RoutedEventArgs e) => SubmitSearch(SearchBox.Text);

        private void SearchKeyDown(object sender, Windows.UI.Xaml.Input.KeyRoutedEventArgs e)
        {
            if (e.Key == Windows.System.VirtualKey.Enter) SubmitSearch(SearchBox.Text);
        }

        private void SubmitSearch(string? query)
        {
            _lastSearch = query ?? "";
            if (ContentFrame.CurrentSourcePageType != typeof(BrowsePage))
            {
                ContentFrame.Navigate(typeof(BrowsePage), _lastSearch);
            }
            else if (ContentFrame.Content is BrowsePage browse)
            {
                browse.ApplySearch(_lastSearch);
            }
        }

        private void UpdateAccount()
        {
            var user = AppServices.CurrentUser;
            AccountItem.Content = user == null ? "登录" : user.DisplayName;
        }

        private void OnSessionExpired(string message)
        {
            SessionBar.Message = message;
            SessionBar.IsOpen = true;
        }

        private async void SessionLoginClick(object sender, RoutedEventArgs e)
        {
            SessionBar.IsOpen = false;
            await AppServices.SignOutAsync();
            Frame.Navigate(typeof(LoginPage));
            Frame.BackStack.Clear();
        }
    }
}
