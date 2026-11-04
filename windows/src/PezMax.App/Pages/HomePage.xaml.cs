using System;
using System.Collections.ObjectModel;
using System.Threading.Tasks;
using PezMax.Core.Api;
using PezMax.Core.Models;
using PezMax.Core.Net;
using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Navigation;

namespace PezMax.Pages
{
    public sealed partial class HomePage : Page
    {
        private readonly ObservableCollection<FileItem> _recent = new();

        public HomePage()
        {
            InitializeComponent();
            RecentList.ItemsSource = _recent;
        }

        protected override async void OnNavigatedTo(NavigationEventArgs e)
        {
            base.OnNavigatedTo(e);
            var user = AppServices.CurrentUser;
            WelcomeTitle.Text = AppServices.PreviewMode ? "欢迎，预览模式" : user == null ? "欢迎" : $"欢迎，{user.DisplayName}";
            WelcomeCaption.Text = user == null ? "拼图满绩 · 学习资料" : $"已上传 {user.Count} 份资料";
            await LoadAsync();
        }

        private async Task LoadAsync()
        {
            ErrorBar.IsOpen = false;
            RecentProgress.IsActive = true;
            RecentProgress.Visibility = Visibility.Visible;
            try
            {
                if (AppServices.PreviewMode)
                {
                    UploadValue.Text = "6";
                    DownloadValue.Text = "42";
                    FavoriteValue.Text = "9";
                    _recent.Clear();
                    foreach (var file in PreviewData.Files)
                    {
                        _recent.Add(file);
                        if (_recent.Count >= 5) break;
                    }
                    RecentEmpty.Visibility = Visibility.Collapsed;
                    return;
                }

                if (AppServices.IsLoggedIn)
                {
                    var stats = await AppServices.Profile.GetStatsAsync();
                    UploadValue.Text = stats.UploadCount.ToString();
                    DownloadValue.Text = stats.DownloadCount.ToString();
                    FavoriteValue.Text = stats.FavoriteCount.ToString();
                }

                var page = await AppServices.FilesApi.ListAsync(new FileQuery { PageNum = 1, PageSize = 12 });
                _recent.Clear();
                foreach (var file in page.Rows) _recent.Add(file);
                RecentEmpty.Visibility = _recent.Count == 0 ? Visibility.Visible : Visibility.Collapsed;
            }
            catch (DatumException ex)
            {
                ShowError(ex.Message);
            }
            catch (Exception)
            {
                ShowError("加载失败，请检查网络或后端地址");
            }
            finally
            {
                RecentProgress.IsActive = false;
                RecentProgress.Visibility = Visibility.Collapsed;
            }
        }

        private void ShowError(string message)
        {
            ErrorBar.Message = message;
            ErrorBar.IsOpen = true;
        }

        private void BrowseClick(object sender, RoutedEventArgs e) => Frame.Navigate(typeof(BrowsePage));

        private void FavoritesClick(object sender, RoutedEventArgs e) => Frame.Navigate(typeof(BrowsePage), "favorites");

        private void DownloadsClick(object sender, RoutedEventArgs e) => Frame.Navigate(typeof(ProfilePage), "downloads");

        private void SettingsClick(object sender, RoutedEventArgs e) => Frame.Navigate(typeof(ProfilePage), "settings");
    }
}
