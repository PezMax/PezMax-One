using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using PezMax.Core.Models;
using PezMax.Core.Net;
using Windows.Storage;
using Windows.Storage.Pickers;
using Windows.System;
using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Navigation;
using muxc = Microsoft.UI.Xaml.Controls;

namespace PezMax.Pages
{
    public sealed partial class CommunityPage : Page
    {
        private readonly ObservableCollection<RankEntry> _rank = new();
        private readonly ObservableCollection<ReportItem> _reports = new();
        private readonly List<StorageFile> _picked = new();
        private bool _loaded;

        public CommunityPage()
        {
            InitializeComponent();
            RankList.ItemsSource = _rank;
            ReportList.ItemsSource = _reports;
            UploadTypeBox.ItemsSource = FileTypes.Names;
        }

        protected override async void OnNavigatedTo(NavigationEventArgs e)
        {
            base.OnNavigatedTo(e);
            if (_loaded) return;
            _loaded = true;
            await LoadRankAsync();
            await LoadReportsAsync();
        }

        private async Task LoadRankAsync()
        {
            RankProgress.IsActive = true;
            RankProgress.Visibility = Visibility.Visible;
            try
            {
                if (AppServices.PreviewMode)
                {
                    _rank.Clear();
                    foreach (var row in PreviewData.Rank) _rank.Add(row);
                    return;
                }

                var rows = await AppServices.Community.GetRankAsync();
                _rank.Clear();
                foreach (var row in rows) _rank.Add(row);
            }
            catch (Exception ex)
            {
                ShowBar(UploadBar, muxc.InfoBarSeverity.Error, ex is DatumException datum ? datum.Message : "排行榜加载失败");
            }
            finally
            {
                RankProgress.IsActive = false;
                RankProgress.Visibility = Visibility.Collapsed;
            }
        }

        private async Task LoadReportsAsync()
        {
            if (AppServices.PreviewMode || AppServices.UserId <= 0) return;
            ReportProgress.IsActive = true;
            ReportProgress.Visibility = Visibility.Visible;
            try
            {
                var files = await AppServices.Reports.GetMyFileReportsAsync(AppServices.UserId, 1, 100);
                var bookmarks = await AppServices.Reports.GetMyBookmarkReportsAsync(AppServices.UserId, 1, 100);
                var ebooks = await AppServices.Ebooks.GetMyReportsAsync(AppServices.UserId, 1, 100);

                _reports.Clear();
                foreach (var item in files.Rows) _reports.Add(item);
                foreach (var item in bookmarks.Rows) _reports.Add(item);
                foreach (var item in ebooks.Rows) _reports.Add(item);
            }
            catch (Exception ex)
            {
                ShowBar(UploadBar, muxc.InfoBarSeverity.Error, ex is DatumException datum ? datum.Message : "举报记录加载失败");
            }
            finally
            {
                ReportProgress.IsActive = false;
                ReportProgress.Visibility = Visibility.Collapsed;
            }
        }

        private async void RefreshRankClick(object sender, RoutedEventArgs e) => await LoadRankAsync();

        private async void RefreshReportsClick(object sender, RoutedEventArgs e) => await LoadReportsAsync();

        private async void PickFilesClick(object sender, RoutedEventArgs e)
        {
            var picker = new FileOpenPicker();
            foreach (var ext in new[] { ".doc", ".docx", ".ppt", ".pptx", ".pdf", ".jpg", ".jpeg", ".png", ".webp", ".txt", ".md", ".epub", ".mobi", ".azw3" })
            {
                picker.FileTypeFilter.Add(ext);
            }

            var files = await picker.PickMultipleFilesAsync();
            _picked.Clear();
            if (files != null) _picked.AddRange(files);
            PickedText.Text = _picked.Count == 0 ? "尚未选择文件" : $"已选择 {_picked.Count} 个文件";
        }

        private async void UploadClick(object sender, RoutedEventArgs e)
        {
            if (_picked.Count == 0)
            {
                ShowBar(UploadBar, muxc.InfoBarSeverity.Warning, "请先选择文件");
                return;
            }
            if (string.IsNullOrWhiteSpace(UploadSubjectBox.Text))
            {
                ShowBar(UploadBar, muxc.InfoBarSeverity.Warning, "科目不能为空");
                return;
            }
            if (!int.TryParse(UploadYearBox.Text, out var year) || year < 1900 || year > 2100)
            {
                ShowBar(UploadBar, muxc.InfoBarSeverity.Warning, "年份需在 1900 到 2100 之间");
                return;
            }

            var fileType = UploadTypeBox.SelectedIndex + 1;
            UploadProgress.IsActive = true;
            UploadProgress.Visibility = Visibility.Visible;
            var success = 0;
            var failed = 0;
            try
            {
                foreach (var file in _picked)
                {
                    try
                    {
                        using var stream = (await file.OpenAsync(FileAccessMode.Read)).AsStreamForRead();
                        await AppServices.FilesApi.UploadAsync(
                            file.Name,
                            stream,
                            string.IsNullOrWhiteSpace(file.ContentType) ? "application/octet-stream" : file.ContentType,
                            UploadSubjectBox.Text.Trim(),
                            UploadSchoolBox.Text.Trim(),
                            year,
                            fileType);
                        success++;
                    }
                    catch (Exception)
                    {
                        failed++;
                    }
                }

                ShowBar(UploadBar,
                    failed == 0 ? muxc.InfoBarSeverity.Success : muxc.InfoBarSeverity.Warning,
                    $"上传完成：成功 {success} 个，失败 {failed} 个；通过审核后公开");
            }
            finally
            {
                UploadProgress.IsActive = false;
                UploadProgress.Visibility = Visibility.Collapsed;
            }
        }

        private async void TimelineClick(object sender, RoutedEventArgs e)
        {
            if (ReportList.SelectedItem is not ReportItem report) return;
            try
            {
                IReadOnlyList<TimelineEvent> events;
                if (report.BookmarkId > 0)
                {
                    events = await AppServices.Reports.GetBookmarkTimelineAsync(report.ReportId);
                }
                else if (report.FileId > 0)
                {
                    events = await AppServices.Reports.GetFileTimelineAsync(report.ReportId);
                }
                else
                {
                    ShowBar(UploadBar, muxc.InfoBarSeverity.Informational, "电子书举报暂不提供时间线");
                    return;
                }

                var text = string.Join("\n\n", events.Select(item =>
                    $"{item.Step} · {item.Time}\n结果：{item.Result}{(string.IsNullOrWhiteSpace(item.Reason) ? "" : "\n原因：" + item.Reason)}{(string.IsNullOrWhiteSpace(item.Remark) ? "" : "\n备注：" + item.Remark)}{(string.IsNullOrWhiteSpace(item.Conclusion) ? "" : "\n结论：" + item.Conclusion)}"));
                await new PezDialog
                {
                    Title = $"举报 #{report.ReportId} 时间线",
                    Content = new ScrollViewer
                    {
                        MaxHeight = 480,
                        Content = new TextBlock { Text = text, TextWrapping = TextWrapping.Wrap, IsTextSelectionEnabled = true },
                    },
                    CloseButtonText = "关闭",
                    DefaultButton = ContentDialogButton.Close,
                }.ShowAsync();
            }
            catch (Exception ex)
            {
                ShowBar(UploadBar, muxc.InfoBarSeverity.Error, ex is DatumException datum ? datum.Message : "时间线加载失败");
            }
        }

        private void ShowBar(muxc.InfoBar bar, muxc.InfoBarSeverity severity, string message)
        {
            bar.Severity = severity;
            bar.Message = message;
            bar.IsOpen = true;
        }
    }
}
