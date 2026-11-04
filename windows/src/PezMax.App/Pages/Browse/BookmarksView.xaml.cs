using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using PezMax.Core.Api;
using PezMax.Core.Models;
using PezMax.Core.Net;
using PezMax.Core.Urls;
using PezMax.Core.Validation;
using Windows.Storage;
using Windows.Storage.Pickers;
using Windows.System;
using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Input;
using Windows.UI.Xaml.Media.Imaging;
using Windows.UI.Xaml.Navigation;
using muxc = Microsoft.UI.Xaml.Controls;

namespace PezMax.Pages.Browse
{
    public sealed partial class BookmarksView : UserControl
    {
        private readonly ObservableCollection<Bookmark> _items = new();
        private Bookmark? _selected;
        private string _keyword = "";

        public BookmarksView()
        {
            InitializeComponent();
            BookmarkList.ItemsSource = _items;
            Loaded += async (_, __) => await LoadAsync();
        }

        public void ApplySearch(string keyword)
        {
            SearchBox.Text = keyword;
            _ = SearchAsync(keyword);
        }

        private async Task LoadAsync()
        {
            SetBusy(true);
            try
            {
                var page = await AppServices.Bookmarks.ListAsync(new BookmarkQuery
                {
                    PageNum = 1,
                    PageSize = 100,
                    Keyword = string.IsNullOrWhiteSpace(_keyword) ? null : _keyword,
                });
                _items.Clear();
                foreach (var item in page.Rows) _items.Add(item);
                ListCaption.Text = $"书签 · {page.Total} 条";
            }
            catch (Exception ex)
            {
                ShowError(ex is DatumException datum ? datum.Message : "书签加载失败");
            }
            finally
            {
                SetBusy(false);
            }
        }

        private async Task SearchAsync(string keyword)
        {
            _keyword = keyword ?? "";
            await LoadAsync();
        }

        private void BookmarkSelectionChanged(object sender, SelectionChangedEventArgs e)
        {
            if (BookmarkList.SelectedItem is Bookmark bookmark) ShowDetail(bookmark);
        }

        private void ShowDetail(Bookmark bookmark)
        {
            _selected = bookmark;
            DetailPanel.Visibility = Visibility.Visible;
            DetailTitle.Text = bookmark.Title;
            DetailUrl.Text = bookmark.Url;
            DetailDescription.Text = bookmark.Description;
            DetailMeta.Text = $"{bookmark.Subject} · {bookmark.ResourceType} · {bookmark.Collection} · {bookmark.CreateTime}";
            ErrorBar.IsOpen = false;
            if (!string.IsNullOrWhiteSpace(bookmark.CoverImage))
            {
                CoverImage.Source = new BitmapImage(new Uri(UrlNormalizer.Normalize(bookmark.CoverImage, AppServices.Settings.ServerUrl)));
                CoverImage.Visibility = Visibility.Visible;
            }
            else
            {
                CoverImage.Source = null;
                CoverImage.Visibility = Visibility.Collapsed;
            }
        }

        private async void SearchClick(object sender, RoutedEventArgs e) => await SearchAsync(SearchBox.Text);

        private async void SearchKeyDown(object sender, KeyRoutedEventArgs e)
        {
            if (e.Key == VirtualKey.Enter) await SearchAsync(SearchBox.Text);
        }

        private async void RefreshClick(object sender, RoutedEventArgs e) => await LoadAsync();

        private async void CreateClick(object sender, RoutedEventArgs e)
        {
            var dialog = BuildEditDialog(null);
            if (await dialog.ShowAsync() != ContentDialogResult.Primary) return;
            var write = ReadWriteDialog(dialog);
            try
            {
                await AppServices.Bookmarks.CreateAsync(write);
                await LoadAsync();
            }
            catch (Exception ex)
            {
                ShowError(ex is DatumException datum ? datum.Message : "书签保存失败");
            }
        }

        private async void EditClick(object sender, RoutedEventArgs e)
        {
            if (_selected == null) return;
            var dialog = BuildEditDialog(_selected);
            if (await dialog.ShowAsync() != ContentDialogResult.Primary) return;
            var write = ReadWriteDialog(dialog);
            write.Id = _selected.Id > 0 ? _selected.Id : _selected.BookmarkId;
            try
            {
                await AppServices.Bookmarks.UpdateAsync(write);
                await LoadAsync();
            }
            catch (Exception ex)
            {
                ShowError(ex is DatumException datum ? datum.Message : "书签更新失败");
            }
        }

        private static ContentDialog BuildEditDialog(Bookmark? bookmark)
        {
            var panel = new StackPanel { Spacing = 6, Width = 420 };
            AddField(panel, "网址", bookmark?.Url ?? "", "UrlBox");
            AddField(panel, "标题", bookmark?.Title ?? "", "TitleBox");
            AddField(panel, "描述", bookmark?.Description ?? "", "DescriptionBox");
            AddField(panel, "科目", bookmark?.Subject ?? "", "SubjectBox");
            AddField(panel, "资源类型", bookmark?.ResourceType ?? "other", "ResourceTypeBox");
            AddField(panel, "合集", bookmark?.Collection ?? "", "CollectionBox");
            AddField(panel, "备注", bookmark?.Remark ?? "", "RemarkBox");

            return new PezDialog
            {
                Title = bookmark == null ? "新建书签" : "编辑书签",
                Content = new ScrollViewer { Content = panel, MaxHeight = 520 },
                PrimaryButtonText = "保存",
                CloseButtonText = "取消",
                DefaultButton = ContentDialogButton.Primary,
            };
        }

        private static void AddField(Panel panel, string header, string value, string name)
        {
            panel.Children.Add(new TextBox { Header = header, Text = value, Name = name, CornerRadius = new CornerRadius(0) });
        }

        private static BookmarkWrite ReadWriteDialog(ContentDialog dialog)
        {
            var panel = (StackPanel)((ScrollViewer)dialog.Content).Content;
            string Text(string name) => panel.Children.OfType<TextBox>().First(box => box.Name == name).Text;
            return new BookmarkWrite
            {
                Url = Text("UrlBox"),
                Title = Text("TitleBox"),
                Description = Text("DescriptionBox"),
                Subject = Text("SubjectBox"),
                ResourceType = Text("ResourceTypeBox"),
                Collection = Text("CollectionBox"),
                Remark = Text("RemarkBox"),
            };
        }

        private async void OpenClick(object sender, RoutedEventArgs e)
        {
            if (_selected == null) return;
            await Launcher.LaunchUriAsync(new Uri(_selected.Url));
        }

        private async void CoverClick(object sender, RoutedEventArgs e)
        {
            if (_selected == null) return;
            var picker = new FileOpenPicker();
            picker.FileTypeFilter.Add(".jpg");
            picker.FileTypeFilter.Add(".jpeg");
            picker.FileTypeFilter.Add(".png");
            picker.FileTypeFilter.Add(".gif");
            picker.FileTypeFilter.Add(".webp");
            var file = await picker.PickSingleFileAsync();
            if (file == null) return;

            try
            {
                var id = _selected.Id > 0 ? _selected.Id : _selected.BookmarkId;
                using var stream = (await file.OpenAsync(FileAccessMode.Read)).AsStreamForRead();
                var result = await AppServices.Bookmarks.UploadCoverAsync(
                    id, file.Name, stream, string.IsNullOrWhiteSpace(file.ContentType) ? "image/jpeg" : file.ContentType);
                _selected = Bookmark.From(null);
                await LoadAsync();
                ErrorBar.Severity = muxc.InfoBarSeverity.Success;
                ErrorBar.Message = $"封面已更新：{result.Url}";
                ErrorBar.IsOpen = true;
            }
            catch (Exception ex)
            {
                ShowError(ex is DatumException datum ? datum.Message : "封面上传失败");
            }
        }

        private async void FavoriteClick(object sender, RoutedEventArgs e)
        {
            if (_selected == null) return;
            try
            {
                var id = _selected.Id > 0 ? _selected.Id : _selected.BookmarkId;
                await AppServices.Bookmarks.AddFavoriteAsync(id);
                ErrorBar.Severity = muxc.InfoBarSeverity.Success;
                ErrorBar.Message = "已收藏";
                ErrorBar.IsOpen = true;
            }
            catch (Exception ex)
            {
                ShowError(ex is DatumException datum ? datum.Message : "收藏失败");
            }
        }

        private async void ReportClick(object sender, RoutedEventArgs e)
        {
            if (_selected == null) return;
            var input = new TextBox
            {
                PlaceholderText = "请填写举报原因（不超过 500 字）",
                AcceptsReturn = true,
                Height = 96,
                TextWrapping = TextWrapping.Wrap,
                CornerRadius = new CornerRadius(0),
            };
            var dialog = new PezDialog
            {
                Title = "举报书签",
                Content = input,
                PrimaryButtonText = "提交",
                CloseButtonText = "取消",
                DefaultButton = ContentDialogButton.Primary,
            };
            if (await dialog.ShowAsync() != ContentDialogResult.Primary) return;

            try
            {
                var id = _selected.Id > 0 ? _selected.Id : _selected.BookmarkId;
                await AppServices.Reports.CreateBookmarkReportAsync(id, input.Text);
                ErrorBar.Severity = muxc.InfoBarSeverity.Success;
                ErrorBar.Message = "举报已提交";
                ErrorBar.IsOpen = true;
            }
            catch (Exception ex)
            {
                ShowError(ex is DatumException datum ? datum.Message : "举报提交失败");
            }
        }

        private async void DeleteClick(object sender, RoutedEventArgs e)
        {
            if (_selected == null) return;
            var dialog = new PezDialog
            {
                Title = "删除书签",
                Content = $"确定删除「{_selected.Title}」吗？",
                PrimaryButtonText = "删除",
                CloseButtonText = "取消",
                DefaultButton = ContentDialogButton.Close,
            };
            if (await dialog.ShowAsync() != ContentDialogResult.Primary) return;

            try
            {
                var id = _selected.Id > 0 ? _selected.Id : _selected.BookmarkId;
                await AppServices.Bookmarks.DeleteAsync(id);
                DetailPanel.Visibility = Visibility.Collapsed;
                await LoadAsync();
            }
            catch (Exception ex)
            {
                ShowError(ex is DatumException datum ? datum.Message : "删除失败");
            }
        }

        private void SetBusy(bool busy)
        {
            ListProgress.IsActive = busy;
            ListProgress.Visibility = busy ? Visibility.Visible : Visibility.Collapsed;
        }

        private void ShowError(string message)
        {
            ErrorBar.Severity = muxc.InfoBarSeverity.Error;
            ErrorBar.Message = message;
            ErrorBar.IsOpen = true;
            DetailPanel.Visibility = Visibility.Visible;
        }
    }
}
