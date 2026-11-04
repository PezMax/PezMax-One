using System;
using PezMax.Core.Models;
using PezMax.Login;
using PezMax.Pages;
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

        public ShellPage()
        {
            InitializeComponent();
            SetUpTitleBar();
            AppServices.SessionChanged += UpdateAccount;
            AppServices.SessionExpired += OnSessionExpired;
            Loaded += OnLoaded;
        }

        private void OnLoaded(object sender, RoutedEventArgs e)
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
