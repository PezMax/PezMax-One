using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using PezMax.Core.Api;
using PezMax.Core.Models;
using PezMax.Core.Net;
using PezMax.Core.Urls;
using PezMax.Transfer;
using Windows.ApplicationModel.DataTransfer;
using Windows.Data.Pdf;
using Windows.Storage.Streams;
using Windows.System;
using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Input;
using Windows.UI.Xaml.Media.Imaging;
using muxc = Microsoft.UI.Xaml.Controls;

namespace PezMax.Pages.Browse
{
    public sealed partial class PapersView : UserControl
    {
        private readonly ObservableCollection<FileItem> _files = new();
        private readonly Dictionary<FileTreeNode, muxc.TreeViewNode> _nodeMap = new();
        private FileItem? _selected;
        private string? _treeHash;
        private PdfDocument? _pdf;
        private InMemoryRandomAccessStream? _pdfStream;
        private int _pdfPage;

        public PapersView()
        {
            InitializeComponent();
            FileList.ItemsSource = _files;
            Loaded += async (_, __) =>
            {
                if (AppServices.PreviewMode) await LoadPreviewAsync();
                else await LoadTreeAsync();
            };
        }

        public void ApplySearch(string keyword)
        {
            SearchBox.Text = keyword;
            _ = SearchAsync(keyword);
        }

        private sealed class TreeEntry
        {
            public TreeEntry(string label, FileTreeNode data, muxc.TreeViewNode view)
            {
                Label = label;
                Data = data;
                View = view;
            }

            public string Label { get; }
            public FileTreeNode Data { get; }
            public muxc.TreeViewNode View { get; }
            public override string ToString() => Label;
        }

        private async Task LoadPreviewAsync()
        {
            _treeHash = null;
            Tree.RootNodes.Clear();
            _nodeMap.Clear();
            foreach (var node in PreviewData.Tree) Tree.RootNodes.Add(BuildNode(node));
            await LoadListAsync(new FileQuery { PageNum = 1, PageSize = 100 }, "演示资料");
        }

        private async Task LoadTreeAsync()
        {
            try
            {
                var result = await AppServices.FilesApi.GetTreeAsync(_treeHash);
                if (result.Unchanged) return;
                _treeHash = result.Hash;
                Tree.RootNodes.Clear();
                _nodeMap.Clear();
                foreach (var node in result.Nodes) Tree.RootNodes.Add(BuildNode(node));
            }
            catch (Exception ex)
            {
                ShowListError(ex is DatumException datum ? datum.Message : "文件树加载失败");
            }
        }

        private muxc.TreeViewNode BuildNode(FileTreeNode node)
        {
            var viewNode = new muxc.TreeViewNode { IsExpanded = false };
            viewNode.Content = new TreeEntry(node.Label, node, viewNode);
            _nodeMap[node] = viewNode;
            foreach (var child in node.Children) viewNode.Children.Add(BuildNode(child));
            return viewNode;
        }

        private async void TreeItemInvoked(muxc.TreeView sender, muxc.TreeViewItemInvokedEventArgs args)
        {
            if (args.InvokedItem is not TreeEntry entry) return;
            var data = entry.Data;
            var node = entry.View;

            if (!data.IsFolder && data.File != null)
            {
                ShowDetail(data.File);
                await SelectInListAsync(data.File);
                return;
            }

            node.IsExpanded = !node.IsExpanded;
            var query = BuildQuery(node);
            await LoadListAsync(query, data.Label);
        }

        private static FileQuery BuildQuery(muxc.TreeViewNode node)
        {
            var labels = new List<string>();
            var current = node;
            while (current?.Content is FileTreeNode data)
            {
                labels.Add(data.Label);
                current = current.Parent;
            }
            labels.Reverse();

            var query = new FileQuery { PageNum = 1, PageSize = 100 };
            if (labels.Count > 0) query.FileSubject = labels[0];
            if (labels.Count > 1) query.FileSchool = labels[1];
            if (labels.Count > 2)
            {
                var type = FileTypes.IdFromName(labels[2]);
                if (type > 0) query.FileType = type;
            }
            if (labels.Count > 3 && int.TryParse(labels[3], out var year)) query.FileYear = year;
            return query;
        }

        private async Task LoadListAsync(FileQuery query, string caption)
        {
            SetListBusy(true);
            try
            {
                if (AppServices.PreviewMode)
                {
                    var rows = PreviewData.Files.Where(item =>
                        (query.FileSubject == null || item.FileSubject == query.FileSubject) &&
                        (query.FileSchool == null || item.FileSchool == query.FileSchool) &&
                        (query.FileYear == null || item.FileYear == query.FileYear) &&
                        (query.FileType == null || item.FileType == query.FileType)).ToList();
                    _files.Clear();
                    foreach (var item in rows) _files.Add(item);
                    ListCaption.Text = $"{caption} · {rows.Count} 条（演示）";
                    return;
                }

                var page = await AppServices.FilesApi.ListAsync(query);
                _files.Clear();
                foreach (var item in page.Rows) _files.Add(item);
                ListCaption.Text = $"{caption} · {page.Total} 条";
            }
            catch (Exception ex)
            {
                ShowListError(ex is DatumException datum ? datum.Message : "列表加载失败");
            }
            finally
            {
                SetListBusy(false);
            }
        }

        private async Task SearchAsync(string keyword)
        {
            if (string.IsNullOrWhiteSpace(keyword))
            {
                await LoadListAsync(new FileQuery { PageNum = 1, PageSize = 100 }, "全部资料");
                return;
            }

            SetListBusy(true);
            try
            {
                if (AppServices.PreviewMode)
                {
                    var text = keyword.Trim();
                    var previewRows = PreviewData.Files.Where(item =>
                        item.DisplayName.Contains(text, StringComparison.OrdinalIgnoreCase) ||
                        item.FileSubject.Contains(text, StringComparison.OrdinalIgnoreCase) ||
                        item.FileSchool.Contains(text, StringComparison.OrdinalIgnoreCase)).ToList();
                    _files.Clear();
                    foreach (var item in previewRows) _files.Add(item);
                    ListCaption.Text = $"搜索「{text}」· {previewRows.Count} 条（演示）";
                    return;
                }

                var rows = await AppServices.FilesApi.SearchAsync(keyword.Trim());
                _files.Clear();
                foreach (var item in rows) _files.Add(item);
                ListCaption.Text = $"搜索「{keyword.Trim()}」· {rows.Count} 条";
            }
            catch (Exception ex)
            {
                ShowListError(ex is DatumException datum ? datum.Message : "搜索失败");
            }
            finally
            {
                SetListBusy(false);
            }
        }

        private async Task SelectInListAsync(FileItem file)
        {
            var match = _files.FirstOrDefault(item => item.FileId == file.FileId);
            if (match != null)
            {
                FileList.SelectedItem = match;
                return;
            }

            _files.Insert(0, file);
            FileList.SelectedItem = file;
            await Task.CompletedTask;
        }

        private void FileSelectionChanged(object sender, SelectionChangedEventArgs e)
        {
            if (FileList.SelectedItem is FileItem file) ShowDetail(file);
        }

        private void ShowDetail(FileItem file)
        {
            _selected = file;
            DetailPanel.Visibility = Visibility.Visible;
            DetailTitle.Text = file.DisplayName;
            DetailMeta.Text = $"{file.TypeName} · {file.SubjectSchoolText} · {file.FileYear} · {file.SizeText} · {file.FileFormat.ToUpperInvariant()}";
            DetailStatus.Text = $"状态：{file.StatusText} · 上传者 ID {file.UserId} · {file.CreateTime}";
            ResetPreview();
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
                var favorited = await AppServices.Favorites.IsFileFavoritedAsync(_selected.FileId);
                FavoriteButton.Content = favorited ? "取消收藏" : "收藏";
            }
            catch (Exception)
            {
                FavoriteButton.Content = "收藏";
            }
        }

        private void ResetPreview()
        {
            PreviewHost.Visibility = Visibility.Collapsed;
            PreviewImage.Source = null;
            PreviewText.Text = "";
            PreviewTextHost.Visibility = Visibility.Collapsed;
            PdfPager.Visibility = Visibility.Collapsed;
            PreviewError.IsOpen = false;
            _pdf = null;
            _pdfStream?.Dispose();
            _pdfStream = null;
            _pdfPage = 0;
        }

        private async void SearchClick(object sender, RoutedEventArgs e) => await SearchAsync(SearchBox.Text);

        private async void SearchKeyDown(object sender, KeyRoutedEventArgs e)
        {
            if (e.Key == VirtualKey.Enter) await SearchAsync(SearchBox.Text);
        }

        private async void RefreshClick(object sender, RoutedEventArgs e)
        {
            _treeHash = null;
            if (AppServices.PreviewMode)
            {
                await LoadPreviewAsync();
                return;
            }

            await LoadTreeAsync();
            await LoadListAsync(new FileQuery { PageNum = 1, PageSize = 100 }, "全部资料");
        }

        private async void PreviewClick(object sender, RoutedEventArgs e)
        {
            if (_selected == null) return;
            await PreviewAsync(_selected);
        }

        private async Task PreviewAsync(FileItem file)
        {
            ResetPreview();
            SetPreviewBusy(true);
            try
            {
                var format = file.FileFormat.ToLowerInvariant();
                if (format is "jpg" or "jpeg" or "png" or "webp" or "gif")
                {
                    var url = UrlNormalizer.Normalize(file.FileUrl, AppServices.Settings.ServerUrl);
                    PreviewImage.Source = new BitmapImage(new Uri(url));
                    PreviewHost.Visibility = Visibility.Visible;
                }
                else if (format is "txt" or "md")
                {
                    var text = await AppServices.FilesApi.GetTextContentAsync(file.FileId);
                    PreviewText.Text = text;
                    PreviewTextHost.Visibility = Visibility.Visible;
                    PreviewHost.Visibility = Visibility.Visible;
                }
                else if (format == "pdf")
                {
                    _pdfStream = await DownloadService.ReadStreamAsync(
                        ct => AppServices.Downloads.OpenFileStreamAsync(file.FileId, ct));
                    _pdf = await PdfDocument.LoadFromStreamAsync(_pdfStream);
                    PreviewHost.Visibility = Visibility.Visible;
                    PdfPager.Visibility = Visibility.Visible;
                    await RenderPdfPageAsync();
                }
                else
                {
                    PreviewError.Message = "该格式暂不支持内置预览，请下载后查看";
                    PreviewError.IsOpen = true;
                }
            }
            catch (Exception ex)
            {
                PreviewError.Message = ex is DatumException datum ? datum.Message : "预览加载失败";
                PreviewError.IsOpen = true;
            }
            finally
            {
                SetPreviewBusy(false);
            }
        }

        private async Task RenderPdfPageAsync()
        {
            if (_pdf == null || _pdf.PageCount == 0) return;
            _pdfPage = Math.Clamp(_pdfPage, 0, (int)_pdf.PageCount - 1);
            using var page = _pdf.GetPage((uint)_pdfPage);
            using var stream = new InMemoryRandomAccessStream();
            await page.RenderToStreamAsync(stream);
            stream.Seek(0);
            var bitmap = new BitmapImage();
            await bitmap.SetSourceAsync(stream);
            PreviewImage.Source = bitmap;
            PreviewTextHost.Visibility = Visibility.Collapsed;
            PreviewHost.Visibility = Visibility.Visible;
            PdfPageText.Text = $"{_pdfPage + 1} / {_pdf.PageCount}";
        }

        private async void PdfPrevClick(object sender, RoutedEventArgs e)
        {
            if (_pdf == null || _pdfPage <= 0) return;
            _pdfPage--;
            await RenderPdfPageAsync();
        }

        private async void PdfNextClick(object sender, RoutedEventArgs e)
        {
            if (_pdf == null || _pdfPage >= _pdf.PageCount - 1) return;
            _pdfPage++;
            await RenderPdfPageAsync();
        }

        private async void DownloadClick(object sender, RoutedEventArgs e)
        {
            if (_selected == null) return;
            try
            {
                var file = await DownloadService.DownloadFileAsync(_selected);
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
                PreviewError.Message = ex is DatumException datum ? datum.Message : "下载失败";
                PreviewError.IsOpen = true;
            }
        }

        private async void FavoriteClick(object sender, RoutedEventArgs e)
        {
            if (_selected == null) return;
            try
            {
                var favorited = await AppServices.Favorites.IsFileFavoritedAsync(_selected.FileId);
                if (favorited)
                {
                    await AppServices.Favorites.RemoveFileAsync(AppServices.UserId, _selected.FileId);
                    FavoriteButton.Content = "收藏";
                }
                else
                {
                    await AppServices.Favorites.AddFileAsync(_selected.FileId);
                    FavoriteButton.Content = "取消收藏";
                }
            }
            catch (Exception ex)
            {
                PreviewError.Message = ex is DatumException datum ? datum.Message : "收藏操作失败";
                PreviewError.IsOpen = true;
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
                Title = "举报资料",
                Content = input,
                PrimaryButtonText = "提交",
                CloseButtonText = "取消",
                DefaultButton = ContentDialogButton.Primary,
            };
            var result = await dialog.ShowAsync();
            if (result != ContentDialogResult.Primary) return;

            try
            {
                await AppServices.Reports.CreateFileReportAsync(_selected.FileId, input.Text);
                PreviewError.Severity = Microsoft.UI.Xaml.Controls.InfoBarSeverity.Success;
                PreviewError.Message = "举报已提交，等待管理员处理";
                PreviewError.IsOpen = true;
            }
            catch (Exception ex)
            {
                PreviewError.Severity = Microsoft.UI.Xaml.Controls.InfoBarSeverity.Error;
                PreviewError.Message = ex is DatumException datum ? datum.Message : "举报提交失败";
                PreviewError.IsOpen = true;
            }
        }

        private void CopyLinkClick(object sender, RoutedEventArgs e)
        {
            if (_selected == null) return;
            var package = new DataPackage();
            package.SetText(UrlNormalizer.Normalize(_selected.FileUrl, AppServices.Settings.ServerUrl));
            Clipboard.SetContent(package);
        }

        private void SetListBusy(bool busy)
        {
            ListProgress.IsActive = busy;
            ListProgress.Visibility = busy ? Visibility.Visible : Visibility.Collapsed;
        }

        private void SetPreviewBusy(bool busy)
        {
            PreviewProgress.IsActive = busy;
            PreviewProgress.Visibility = busy ? Visibility.Visible : Visibility.Collapsed;
        }

        private void ShowListError(string message)
        {
            PreviewError.Severity = Microsoft.UI.Xaml.Controls.InfoBarSeverity.Error;
            PreviewError.Message = message;
            PreviewError.IsOpen = true;
            DetailPanel.Visibility = Visibility.Visible;
        }
    }
}
