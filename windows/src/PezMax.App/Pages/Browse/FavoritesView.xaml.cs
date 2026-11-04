using System;
using System.Collections.ObjectModel;
using System.Threading.Tasks;
using PezMax.Core.Models;
using PezMax.Core.Net;
using PezMax.Transfer;
using Windows.System;
using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;
using muxc = Microsoft.UI.Xaml.Controls;

namespace PezMax.Pages.Browse
{
    public sealed partial class FavoritesView : UserControl
    {
        private readonly ObservableCollection<FileItem> _files = new();
        private readonly ObservableCollection<EbookItem> _ebooks = new();
        private readonly ObservableCollection<Bookmark> _bookmarks = new();

        public FavoritesView()
        {
            InitializeComponent();
            FileList.ItemsSource = _files;
            EbookList.ItemsSource = _ebooks;
            BookmarkList.ItemsSource = _bookmarks;
            Loaded += async (_, __) => await LoadAllAsync();
        }

        private async Task LoadAllAsync()
        {
            if (!AppServices.IsLoggedIn || AppServices.UserId <= 0)
            {
                Caption.Text = "登录后可查看收藏";
                return;
            }

            SetBusy(true);
            try
            {
                var files = await AppServices.Favorites.GetMyFilesAsync(AppServices.UserId, 1, 100);
                var ebooks = await AppServices.Favorites.GetMyEbooksAsync(AppServices.UserId, 1, 100);
                var bookmarks = await AppServices.Favorites.GetMyBookmarksAsync(AppServices.UserId, 1, 100);

                _files.Clear();
                foreach (var item in files.Rows) _files.Add(item);
                _ebooks.Clear();
                foreach (var item in ebooks.Rows) _ebooks.Add(item);
                _bookmarks.Clear();
                foreach (var item in bookmarks.Rows) _bookmarks.Add(item);
                Caption.Text = $"我的收藏 · 资料 {files.Total} · 电子书 {ebooks.Total} · 书签 {bookmarks.Total}";
            }
            catch (Exception ex)
            {
                ShowError(ex is DatumException datum ? datum.Message : "收藏加载失败");
            }
            finally
            {
                SetBusy(false);
            }
        }

        private async void TabChanged(object sender, SelectionChangedEventArgs e)
        {
            if (!IsLoaded) return;
            await LoadAllAsync();
        }

        private async void OpenFileClick(object sender, RoutedEventArgs e)
        {
            if (FileList.SelectedItem is not FileItem file) return;
            try
            {
                var saved = await DownloadService.DownloadFileAsync(file);
                await Launcher.LaunchFileAsync(saved);
            }
            catch (Exception ex)
            {
                ShowError(ex is DatumException datum ? datum.Message : "下载失败");
            }
        }

        private async void RemoveFileClick(object sender, RoutedEventArgs e)
        {
            if (FileList.SelectedItem is not FileItem file) return;
            try
            {
                await AppServices.Favorites.RemoveFileAsync(AppServices.UserId, file.FileId);
                _files.Remove(file);
            }
            catch (Exception ex)
            {
                ShowError(ex is DatumException datum ? datum.Message : "取消收藏失败");
            }
        }

        private async void DownloadEbookClick(object sender, RoutedEventArgs e)
        {
            if (EbookList.SelectedItem is not EbookItem ebook) return;
            try
            {
                var saved = await DownloadService.DownloadEbookAsync(ebook);
                await Launcher.LaunchFileAsync(saved);
            }
            catch (Exception ex)
            {
                ShowError(ex is DatumException datum ? datum.Message : "下载失败");
            }
        }

        private async void RemoveEbookClick(object sender, RoutedEventArgs e)
        {
            if (EbookList.SelectedItem is not EbookItem ebook) return;
            try
            {
                await AppServices.Favorites.RemoveEbookAsync(AppServices.UserId, ebook.EbookId);
                _ebooks.Remove(ebook);
            }
            catch (Exception ex)
            {
                ShowError(ex is DatumException datum ? datum.Message : "取消收藏失败");
            }
        }

        private async void OpenBookmarkClick(object sender, RoutedEventArgs e)
        {
            if (BookmarkList.SelectedItem is not Bookmark bookmark) return;
            await Launcher.LaunchUriAsync(new Uri(bookmark.Url));
        }

        private async void RemoveBookmarkClick(object sender, RoutedEventArgs e)
        {
            if (BookmarkList.SelectedItem is not Bookmark bookmark) return;
            try
            {
                var id = bookmark.Id > 0 ? bookmark.Id : bookmark.BookmarkId;
                await AppServices.Favorites.RemoveBookmarkAsync(AppServices.UserId, id);
                _bookmarks.Remove(bookmark);
            }
            catch (Exception ex)
            {
                ShowError(ex is DatumException datum ? datum.Message : "取消收藏失败");
            }
        }

        private void SetBusy(bool busy)
        {
            Progress.IsActive = busy;
            Progress.Visibility = busy ? Visibility.Visible : Visibility.Collapsed;
        }

        private void ShowError(string message)
        {
            ErrorBar.Severity = muxc.InfoBarSeverity.Error;
            ErrorBar.Message = message;
            ErrorBar.IsOpen = true;
        }
    }
}
