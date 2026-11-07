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
using PezMax.Reader;
using PezMax.Transfer;
using Windows.Storage;
using Windows.Storage.Pickers;
using Windows.Storage.Streams;
using Windows.System;
using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Input;
using Windows.UI.Xaml.Media.Imaging;
using muxc = Microsoft.UI.Xaml.Controls;

namespace PezMax.Pages.Browse
{
    public sealed partial class EbooksView : UserControl
    {
        private readonly ObservableCollection<EbookItem> _items = new();
        private EbookItem? _selected;
        private string _keyword = "";

        public EbooksView()
        {
            InitializeComponent();
            BookList.ItemsSource = _items;
            PdfHost.CloseRequested += (_, __) => CloseReader();
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
                var page = await AppServices.Ebooks.ListAsync(new EbookQuery
                {
                    PageNum = 1,
                    PageSize = 100,
                    Keyword = string.IsNullOrWhiteSpace(_keyword) ? null : _keyword,
                });
                _items.Clear();
                foreach (var item in page.Rows) _items.Add(item);
                ListCaption.Text = $"电子书 · {page.Total} 本";
            }
            catch (Exception ex)
            {
                ShowError(ex is DatumException datum ? datum.Message : "电子书加载失败");
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

        private void BookSelectionChanged(object sender, SelectionChangedEventArgs e)
        {
            if (BookList.SelectedItem is EbookItem item) ShowDetail(item);
        }

        private void ShowDetail(EbookItem item)
        {
            _selected = item;
            DetailPanel.Visibility = Visibility.Visible;
            DetailTitle.Text = item.DisplayName;
            DetailMeta.Text = $"{item.EbookSubject} · {item.TypeName} · {item.SizeText} · {item.EbookFormat.ToUpperInvariant()}";
            DetailPublisher.Text = string.IsNullOrWhiteSpace(item.Publisher) ? item.Author : $"{item.Author} · {item.Publisher}";
            ErrorBar.IsOpen = false;
            PreviewHost.Visibility = Visibility.Collapsed;
            PreviewHint.Text = "";
            PreviewImage.Source = null;
            CloseReader();

            if (!string.IsNullOrWhiteSpace(item.CoverUrl))
            {
                CoverImage.Source = new BitmapImage(new Uri(UrlNormalizer.Normalize(item.CoverUrl, AppServices.Settings.ServerUrl)));
            }
            else
            {
                CoverImage.Source = null;
            }

            _ = RefreshFavoriteAsync();
        }

        private async Task RefreshFavoriteAsync()
        {
            if (_selected == null || !AppServices.IsLoggedIn)
            {
                FavoriteButton.Content = "收藏";
                return;
            }

            try
            {
                FavoriteButton.Content = await AppServices.Ebooks.IsFavoritedAsync(_selected.EbookId) ? "取消收藏" : "收藏";
            }
            catch (Exception)
            {
                FavoriteButton.Content = "收藏";
            }
        }

        private async void SearchClick(object sender, RoutedEventArgs e) => await SearchAsync(SearchBox.Text);

        private async void SearchKeyDown(object sender, KeyRoutedEventArgs e)
        {
            if (e.Key == VirtualKey.Enter) await SearchAsync(SearchBox.Text);
        }

        private async void RefreshClick(object sender, RoutedEventArgs e) => await LoadAsync();

        private async void PreviewClick(object sender, RoutedEventArgs e)
        {
            if (_selected == null) return;
            SetPreviewBusy(true);
            ErrorBar.IsOpen = false;
            try
            {
                if (string.Equals(_selected.EbookFormat, "pdf", StringComparison.OrdinalIgnoreCase))
                {
                    await OpenReaderAsync(_selected);
                    return;
                }

                PreviewHint.Text = "该格式暂不支持内置预览；请下载后使用系统阅读器打开。";
                PreviewHost.Visibility = Visibility.Collapsed;
            }
            catch (Exception ex)
            {
                ShowError(ex is DatumException datum ? datum.Message : "预览加载失败");
            }
            finally
            {
                SetPreviewBusy(false);
            }
        }

        private async Task OpenReaderAsync(EbookItem item)
        {
            try
            {
                var stream = await DownloadService.ReadStreamAsync(
                    ct => AppServices.Ebooks.OpenContentAsync(item.EbookId, ct));

                // 下载完再显示覆盖层：先显示的话读者对着空窗口等，还以为卡住了。
                ReaderOverlay.Visibility = Visibility.Visible;
                await PdfHost.OpenAsync(
                    stream,
                    item.EbookId,
                    item.DisplayName,
                    AppServices.Settings.PdfView,
                    AppServices.Settings.PdfScale > 0 ? AppServices.Settings.PdfScale : null);
            }
            catch (Exception ex)
            {
                CloseReader();
                ShowError(ex is DatumException datum ? datum.Message : "PDF 打开失败");
            }
        }

        private void CloseReader()
        {
            PdfHost.Unload();
            ReaderOverlay.Visibility = Visibility.Collapsed;
        }

        private async void DownloadClick(object sender, RoutedEventArgs e)
        {
            if (_selected == null) return;
            try
            {
                var file = await DownloadService.DownloadEbookAsync(_selected);
                await new PezDialog
                {
                    Title = "下载完成",
                    Content = file.Path,
                    CloseButtonText = "知道了",
                    DefaultButton = ContentDialogButton.Close,
                }.ShowAsync();
            }
            catch (Exception ex)
            {
                ShowError(ex is DatumException datum ? datum.Message : "下载失败");
            }
        }

        private async void FavoriteClick(object sender, RoutedEventArgs e)
        {
            if (_selected == null) return;
            try
            {
                var favorited = await AppServices.Ebooks.IsFavoritedAsync(_selected.EbookId);
                if (favorited)
                {
                    await AppServices.Ebooks.RemoveFavoriteAsync(_selected.EbookId);
                    FavoriteButton.Content = "收藏";
                }
                else
                {
                    await AppServices.Ebooks.AddFavoriteAsync(_selected.EbookId);
                    FavoriteButton.Content = "取消收藏";
                }
            }
            catch (Exception ex)
            {
                ShowError(ex is DatumException datum ? datum.Message : "收藏操作失败");
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
                Title = "举报电子书",
                Content = input,
                PrimaryButtonText = "提交",
                CloseButtonText = "取消",
                DefaultButton = ContentDialogButton.Primary,
            };
            if (await dialog.ShowAsync() != ContentDialogResult.Primary) return;

            try
            {
                await AppServices.Ebooks.ReportAsync(_selected.EbookId, input.Text);
                ErrorBar.Severity = muxc.InfoBarSeverity.Success;
                ErrorBar.Message = "举报已提交";
                ErrorBar.IsOpen = true;
            }
            catch (Exception ex)
            {
                ShowError(ex is DatumException datum ? datum.Message : "举报提交失败");
            }
        }

        private async void UploadClick(object sender, RoutedEventArgs e)
        {
            var picker = new FileOpenPicker();
            picker.FileTypeFilter.Add(".pdf");
            picker.FileTypeFilter.Add(".epub");
            picker.FileTypeFilter.Add(".mobi");
            picker.FileTypeFilter.Add(".azw3");
            var file = await picker.PickSingleFileAsync();
            if (file == null) return;

            var nameBox = new TextBox { Header = "书名", Text = Path.GetFileNameWithoutExtension(file.Name), CornerRadius = new CornerRadius(0) };
            var authorBox = new TextBox { Header = "作者", CornerRadius = new CornerRadius(0) };
            var publisherBox = new TextBox { Header = "出版社", CornerRadius = new CornerRadius(0) };
            var subjectBox = new TextBox { Header = "学科分类", CornerRadius = new CornerRadius(0) };
            var typeBox = new ComboBox { Header = "类型", SelectedIndex = 0, ItemsSource = new[] { "教材", "教辅/参考书", "课外读物", "其他" }, CornerRadius = new CornerRadius(0) };
            var panel = new StackPanel { Spacing = 6, Width = 420 };
            panel.Children.Add(nameBox);
            panel.Children.Add(authorBox);
            panel.Children.Add(publisherBox);
            panel.Children.Add(subjectBox);
            panel.Children.Add(typeBox);

            var dialog = new PezDialog
            {
                Title = "上传电子书",
                Content = panel,
                PrimaryButtonText = "上传",
                CloseButtonText = "取消",
                DefaultButton = ContentDialogButton.Primary,
            };
            if (await dialog.ShowAsync() != ContentDialogResult.Primary) return;
            if (string.IsNullOrWhiteSpace(subjectBox.Text))
            {
                ShowError("学科分类不能为空");
                return;
            }

            try
            {
                using var stream = (await file.OpenAsync(FileAccessMode.Read)).AsStreamForRead();
                await AppServices.Ebooks.UploadAsync(
                    file.Name,
                    stream,
                    string.IsNullOrWhiteSpace(file.ContentType) ? "application/octet-stream" : file.ContentType,
                    nameBox.Text,
                    authorBox.Text,
                    publisherBox.Text,
                    subjectBox.Text,
                    typeBox.SelectedIndex + 1);
                ErrorBar.Severity = muxc.InfoBarSeverity.Success;
                ErrorBar.Message = "已提交，审核通过后公开";
                ErrorBar.IsOpen = true;
                await LoadAsync();
            }
            catch (Exception ex)
            {
                ShowError(ex is DatumException datum ? datum.Message : "上传失败");
            }
        }

        private void SetBusy(bool busy)
        {
            ListProgress.IsActive = busy;
            ListProgress.Visibility = busy ? Visibility.Visible : Visibility.Collapsed;
        }

        private void SetPreviewBusy(bool busy)
        {
            PreviewProgress.IsActive = busy;
            PreviewProgress.Visibility = busy ? Visibility.Visible : Visibility.Collapsed;
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
