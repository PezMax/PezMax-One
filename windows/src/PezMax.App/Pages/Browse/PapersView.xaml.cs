using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using PezMax.Core.Api;
using PezMax.Core.Models;
using PezMax.Core.Net;
using PezMax.Core.Urls;
using PezMax.Reader;
using PezMax.Transfer;
using Windows.ApplicationModel.DataTransfer;
using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Input;
using Windows.UI.Xaml.Media.Imaging;
using muxc = Microsoft.UI.Xaml.Controls;

namespace PezMax.Pages.Browse
{
    /// <summary>
    /// 资料页：按旧 egui 版的流程走——先选条件（科目 / 学校 / 年份 / 类型）再出列表。
    /// 这里没有文件树：用户是来找某一份卷子的，不是来逛目录。
    /// </summary>
    public sealed partial class PapersView : UserControl
    {
        private readonly ObservableCollection<FileItem> _files = new();
        private FileItem? _selected;
        private string _listEmptyText = "";

        /// <summary>填下拉项时不要把 SelectionChanged 当成用户在筛选。</summary>
        private bool _fillingFilters;

        /// <summary>首次 Loaded 之前来的搜索（外壳搜索框回车 → 导航到本页带参数），先记下来。</summary>
        private string _pendingSearch = "";
        private bool _loaded;

        public PapersView()
        {
            InitializeComponent();
            FileList.ItemsSource = _files;
            PdfHost.CloseRequested += (_, __) => CloseReader();
            Loaded += async (_, __) =>
            {
                _loaded = true;
                // 从外壳搜索框回车进来时，OnNavigatedTo 已经把关键词递过来了；
                // 这时候再去 LoadAllAsync 会把结果冲掉，看起来就是「回车没用」。
                if (!string.IsNullOrWhiteSpace(_pendingSearch))
                {
                    var pending = _pendingSearch;
                    _pendingSearch = "";
                    await SearchAsync(pending);
                    return;
                }

                await LoadAllAsync();
            };
        }

        public void ApplySearch(string keyword)
        {
            SearchBox.Text = keyword;
            if (!_loaded)
            {
                _pendingSearch = keyword ?? "";
                return;
            }

            _ = SearchAsync(keyword);
        }

        // ---------------------------------------------------------------- 取数

        private async Task LoadAllAsync()
        {
            await LoadFilterOptionsAsync();
            await ApplyFiltersAsync();
        }

        /// <summary>下拉项来自一次不带条件的拉取：科目 / 学校 / 年份都从结果里认，不另外猜字典。</summary>
        private async Task LoadFilterOptionsAsync()
        {
            IReadOnlyList<FileItem> sample;
            if (AppServices.PreviewMode)
            {
                sample = PreviewData.Files;
            }
            else
            {
                try
                {
                    var page = await AppServices.FilesApi.ListAsync(new FileQuery { PageNum = 1, PageSize = 100 });
                    sample = page.Rows;
                }
                catch (Exception ex)
                {
                    ShowListError(ex is DatumException datum ? datum.Message : "筛选条件加载失败");
                    return;
                }
            }

            _fillingFilters = true;
            try
            {
                Fill(SubjectFilter, sample.Select(item => item.FileSubject)
                    .Where(value => !string.IsNullOrWhiteSpace(value)).Distinct().OrderBy(value => value));

                Fill(SchoolFilter, sample.Select(item => item.FileSchool)
                    .Where(value => !string.IsNullOrWhiteSpace(value)).Distinct().OrderBy(value => value));

                Fill(YearFilter, sample.Select(item => item.FileYear)
                    .Where(year => year > 0).Distinct().OrderByDescending(year => year)
                    .Select(year => year.ToString()));

                Fill(TypeFilter, FileTypes.Names);
            }
            finally
            {
                _fillingFilters = false;
            }
        }

        private static void Fill(ComboBox box, IEnumerable<string> values)
        {
            var keep = box.SelectedItem as string;
            box.ItemsSource = values.ToList();
            if (keep != null && box.Items.Contains(keep)) box.SelectedItem = keep;
        }

        private FileQuery BuildQuery()
        {
            var query = new FileQuery { PageNum = 1, PageSize = 100 };
            if (SubjectFilter.SelectedItem is string subject) query.FileSubject = subject;
            if (SchoolFilter.SelectedItem is string school) query.FileSchool = school;
            if (YearFilter.SelectedItem is string year && long.TryParse(year, out var parsedYear)) query.FileYear = parsedYear;
            if (TypeFilter.SelectedItem is string type)
            {
                var typeId = FileTypes.IdFromName(type);
                if (typeId > 0) query.FileType = typeId;
            }
            return query;
        }

        /// <summary>「化学 · 第二中学 · 2024 · 期中」这种，让列表标题说清现在筛的是什么。</summary>
        private string BuildCaption()
        {
            var parts = new List<string>();
            if (SubjectFilter.SelectedItem is string subject) parts.Add(subject);
            if (SchoolFilter.SelectedItem is string school) parts.Add(school);
            if (YearFilter.SelectedItem is string year) parts.Add(year);
            if (TypeFilter.SelectedItem is string type) parts.Add(type);
            return parts.Count == 0 ? "全部资料" : string.Join(" · ", parts);
        }

        private async Task ApplyFiltersAsync()
        {
            _listEmptyText = "";
            await LoadListAsync(BuildQuery(), BuildCaption());
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
                    Replace(rows);
                    ListCaption.Text = $"{caption} · {rows.Count} 条（演示）";
                    return;
                }

                var page = await AppServices.FilesApi.ListAsync(query);
                Replace(page.Rows);
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
            var text = (keyword ?? "").Trim();
            if (text.Length == 0)
            {
                await ApplyFiltersAsync();
                return;
            }

            _listEmptyText = $"没有匹配「{text}」的资料";
            SetListBusy(true);
            try
            {
                if (AppServices.PreviewMode)
                {
                    var matched = PreviewData.Files.Where(item =>
                        item.DisplayName.Contains(text, StringComparison.OrdinalIgnoreCase) ||
                        item.FileSubject.Contains(text, StringComparison.OrdinalIgnoreCase) ||
                        item.FileSchool.Contains(text, StringComparison.OrdinalIgnoreCase)).ToList();
                    Replace(matched);
                    ListCaption.Text = $"搜索「{text}」· {matched.Count} 条（演示）";
                    return;
                }

                var rows = await AppServices.FilesApi.SearchAsync(text);
                Replace(rows);
                ListCaption.Text = $"搜索「{text}」· {rows.Count} 条";
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

        private void Replace(IEnumerable<FileItem> rows)
        {
            _files.Clear();
            foreach (var item in rows) _files.Add(item);
        }

        // ---------------------------------------------------------------- 交互

        private async void FilterChanged(object sender, SelectionChangedEventArgs e)
        {
            if (_fillingFilters) return;
            await ApplyFiltersAsync();
        }

        private async void ClearFiltersClick(object sender, RoutedEventArgs e)
        {
            _fillingFilters = true;
            try
            {
                SubjectFilter.SelectedItem = null;
                SchoolFilter.SelectedItem = null;
                YearFilter.SelectedItem = null;
                TypeFilter.SelectedItem = null;
                SearchBox.Text = "";
            }
            finally
            {
                _fillingFilters = false;
            }

            await ApplyFiltersAsync();
        }

        private async void SearchClick(object sender, RoutedEventArgs e) => await SearchAsync(SearchBox.Text);

        private async void SearchKeyDown(object sender, KeyRoutedEventArgs e)
        {
            if (e.Key == Windows.System.VirtualKey.Enter) await SearchAsync(SearchBox.Text);
        }

        private async void RefreshClick(object sender, RoutedEventArgs e) => await LoadAllAsync();

        private void FileSelectionChanged(object sender, SelectionChangedEventArgs e)
        {
            if (FileList.SelectedItem is FileItem file) ShowDetail(file);
        }

        private void ShowDetail(FileItem file)
        {
            _selected = file;
            DetailPanel.Visibility = Visibility.Visible;
            DetailTitle.Text = file.DisplayName;
            // 一行铺不下：详情栏只有 300，硬塞会把「格式」整段切在边缘外。
            DetailMeta.Text = $"{file.TypeName} · {file.SubjectSchoolText}\n" +
                              $"{file.FileYear} · {file.SizeText} · {file.FileFormat.ToUpperInvariant()}";
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
            PreviewError.IsOpen = false;

            // 换了文件就关阅读器，免得覆盖层上还停着上一份文档。
            if (ReaderOverlay.Visibility == Visibility.Visible) CloseReader();
        }

        private async Task OpenReaderAsync(FileItem file)
        {
            try
            {
                var stream = await DownloadService.ReadStreamAsync(
                    ct => AppServices.Downloads.OpenFileStreamAsync(file.FileId, ct));

                // 下载完再显示覆盖层：先显示的话读者对着空窗口等，还以为卡住了。
                ReaderOverlay.Visibility = Visibility.Visible;
                await PdfHost.OpenAsync(
                    stream,
                    file.FileId,
                    file.DisplayName,
                    AppServices.Settings.PdfView,
                    AppServices.Settings.PdfScale > 0 ? AppServices.Settings.PdfScale : null);
            }
            catch (Exception ex)
            {
                CloseReader();
                ShowListError(ex is DatumException datum ? datum.Message : "PDF 打开失败");
            }
        }

        private void CloseReader()
        {
            PdfHost.Unload();
            ReaderOverlay.Visibility = Visibility.Collapsed;
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
                    // PDF 走独立阅读器（网格 / 连续两种视图），不再塞进详情面板的小图里。
                    await OpenReaderAsync(file);
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
                PreviewError.Severity = muxc.InfoBarSeverity.Success;
                PreviewError.Message = "举报已提交，等待管理员处理";
                PreviewError.IsOpen = true;
            }
            catch (Exception ex)
            {
                PreviewError.Severity = muxc.InfoBarSeverity.Error;
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
            // 忙的时候别在同一个位置讲空态：两个都说「这里没东西」很容易被读成加载失败。
            if (busy) ListEmpty.Visibility = Visibility.Collapsed;
            else UpdateListEmpty();
        }

        /// <summary>列表空着时给一句为什么空：还没筛 / 这组条件下没有 / 搜不到。</summary>
        private void UpdateListEmpty()
        {
            if (_files.Count > 0)
            {
                ListEmpty.Visibility = Visibility.Collapsed;
                return;
            }

            ListEmpty.Visibility = Visibility.Visible;
            ListEmpty.Text = string.IsNullOrEmpty(_listEmptyText)
                ? "这组条件下没有资料，换一个条件试试"
                : _listEmptyText;
        }

        private void SetPreviewBusy(bool busy)
        {
            PreviewProgress.IsActive = busy;
            PreviewProgress.Visibility = busy ? Visibility.Visible : Visibility.Collapsed;
        }

        private void ShowListError(string message)
        {
            PreviewError.Severity = muxc.InfoBarSeverity.Error;
            PreviewError.Message = message;
            PreviewError.IsOpen = true;
            DetailPanel.Visibility = Visibility.Visible;
        }
    }
}
