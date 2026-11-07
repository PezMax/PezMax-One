using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using PezMax.Core.Api;
using PezMax.Core.Models;
using PezMax.Core.Net;
using PezMax.Core.Validation;
using PezMax.Reader;
using PezMax.Transfer;
using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Navigation;
using muxc = Microsoft.UI.Xaml.Controls;

namespace PezMax.Pages
{
    /// <summary>
    /// 上传资料：挑文件 / 挑文件夹 → 逐个本地校验 → 串行上传 → 逐行结果；
    /// 另一个页签是「我的上传」，可以编辑元信息、删除、预览自己提交的东西。
    /// 校验规则在 Core 的 <see cref="UploadValidation"/> / <see cref="Utf8Check"/>，这里只做表单与呈现。
    /// </summary>
    public sealed partial class UploadPage : Page
    {
        private readonly ObservableCollection<UploadItem> _items = new();
        private readonly ObservableCollection<FileItem> _mine = new();
        private CancellationTokenSource? _cancel;
        private bool _running;
        private bool _mineLoaded;

        public UploadPage()
        {
            InitializeComponent();

            FileList.ItemsSource = _items;
            MineList.ItemsSource = _mine;
            TypeBox.ItemsSource = FileTypes.Names;
            TypeBox.SelectedIndex = 0;
            YearBox.Text = DateTime.Now.Year.ToString();
            PdfHost.CloseRequested += (_, __) => CloseReader();
        }

        protected override void OnNavigatedTo(NavigationEventArgs e)
        {
            base.OnNavigatedTo(e);
            _ = LoadSubjectsAsync();
        }

        private async Task LoadSubjectsAsync()
        {
            try
            {
                var subjects = await AppServices.FilesApi.GetSubjectsAsync();
                SubjectBox.ItemsSource = subjects.Select(item => item.Subject).Where(name => name.Length > 0).ToList();
            }
            catch (Exception)
            {
                // 学科建议拿不到不影响手填。
            }
        }

        private async void SchoolTextChanged(AutoSuggestBox sender, AutoSuggestBoxTextChangedEventArgs args)
        {
            if (args.Reason != AutoSuggestionBoxTextChangeReason.UserInput) return;

            var keyword = (sender.Text ?? "").Trim();
            if (keyword.Length == 0)
            {
                sender.ItemsSource = Array.Empty<string>();
                return;
            }

            try
            {
                sender.ItemsSource = await AppServices.FilesApi.GetSchoolsAsync(keyword);
            }
            catch (Exception)
            {
                sender.ItemsSource = Array.Empty<string>();
            }
        }

        private async void PickFilesClick(object sender, RoutedEventArgs e)
        {
            if (!EnsureLoggedIn()) return;

            try
            {
                Add(await UploadService.PickFilesAsync());
            }
            catch (Exception)
            {
                ShowBar("打开文件选择器失败", muxc.InfoBarSeverity.Error);
            }
        }

        private async void PickFolderClick(object sender, RoutedEventArgs e)
        {
            if (!EnsureLoggedIn()) return;

            try
            {
                Add(await UploadService.PickFolderAsync());
            }
            catch (Exception)
            {
                ShowBar("打开文件夹选择器失败", muxc.InfoBarSeverity.Error);
            }
        }

        private void Add(IEnumerable<UploadItem> items)
        {
            var added = 0;
            foreach (var item in items)
            {
                if (_items.Any(existing => existing.File.Path == item.File.Path)) continue;
                _items.Add(item);
                added++;
            }

            var rejected = _items.Count(entry => entry.State == UploadState.Rejected);
            SummaryText.Text = "共 " + _items.Count + " 个文件";
            if (rejected > 0) SummaryText.Text += "，其中 " + rejected + " 个本地校验未通过";

            if (added > 0) ShowBar("已加入 " + added + " 个文件", muxc.InfoBarSeverity.Informational);
            else if (items.Any()) ShowBar("这些文件已经在列表里了", muxc.InfoBarSeverity.Informational);
        }

        private async void StartClick(object sender, RoutedEventArgs e)
        {
            if (_running || !EnsureLoggedIn()) return;
            if (_items.Count == 0)
            {
                ShowBar("还没有选择文件", muxc.InfoBarSeverity.Warning);
                return;
            }

            var meta = BuildMeta();
            if (meta == null) return;

            _running = true;
            _cancel = new CancellationTokenSource();
            SetBusy(true);

            try
            {
                await UploadService.RunAsync(_items.ToList(), meta, _cancel.Token);
            }
            catch (OperationCanceledException)
            {
                ShowBar("已取消。已经传完的文件照常提交审核。", muxc.InfoBarSeverity.Informational);
            }
            finally
            {
                SetBusy(false);
                _cancel?.Dispose();
                _cancel = null;
                _running = false;
            }

            ReportSummary();
        }

        private void CancelClick(object sender, RoutedEventArgs e) => _cancel?.Cancel();

        private void TabSelectionChanged(object sender, SelectionChangedEventArgs e)
        {
            // 只在这一页第一次打开时拉；之后靠「刷新」，避免每次切页签都发请求。
            if (Tabs.SelectedIndex == 1 && !_mineLoaded) _ = LoadMineAsync();
        }

        private void RefreshMineClick(object sender, RoutedEventArgs e) => _ = LoadMineAsync();

        private async Task LoadMineAsync()
        {
            if (!AppServices.IsLoggedIn)
            {
                ShowMine("登录后才能看到自己的上传", muxc.InfoBarSeverity.Warning);
                return;
            }

            MineRing.IsActive = true;
            MineRing.Visibility = Visibility.Visible;
            try
            {
                // 带 userId 才是「我的上传」：待审 / 未通过也在里面；不带只有已通过的。
                var page = await AppServices.FilesApi.ListAsync(new FileQuery
                {
                    PageNum = 1,
                    PageSize = 100,
                    UserId = AppServices.UserId,
                });

                _mine.Clear();
                foreach (var item in page.Rows) _mine.Add(item);
                _mineLoaded = true;
                MineCaption.Text = "我提交的资料 · " + page.Total + " 份（含待审与未通过）";
            }
            catch (DatumException ex)
            {
                ShowMine(ex.Message, muxc.InfoBarSeverity.Error);
            }
            catch (Exception)
            {
                ShowMine("我的上传加载失败", muxc.InfoBarSeverity.Error);
            }
            finally
            {
                MineRing.IsActive = false;
                MineRing.Visibility = Visibility.Collapsed;
            }
        }

        private async void EditMineClick(object sender, RoutedEventArgs e)
        {
            if (!(sender is FrameworkElement element && element.Tag is FileItem item)) return;
            await EditMineAsync(item);
        }

        private async Task EditMineAsync(FileItem item)
        {
            var nameBox = new TextBox { Header = "文件名", Text = item.DisplayName, CornerRadius = new CornerRadius(0) };
            var subjectBox = new TextBox { Header = "科目", Text = item.FileSubject, CornerRadius = new CornerRadius(0) };
            var schoolBox = new TextBox { Header = "学校", Text = item.FileSchool, CornerRadius = new CornerRadius(0) };
            var yearBox = new TextBox
            {
                Header = "年份",
                Text = item.FileYear > 0 ? item.FileYear.ToString() : "",
                CornerRadius = new CornerRadius(0),
            };
            var typeBox = new ComboBox
            {
                Header = "类型",
                ItemsSource = FileTypes.Names,
                SelectedIndex = Math.Clamp((int)item.FileType - 1, 0, FileTypes.Names.Count - 1),
                CornerRadius = new CornerRadius(0),
            };

            var panel = new StackPanel { Spacing = 6, Width = 420 };
            panel.Children.Add(nameBox);
            panel.Children.Add(subjectBox);
            panel.Children.Add(schoolBox);
            panel.Children.Add(yearBox);
            panel.Children.Add(typeBox);
            panel.Children.Add(new TextBlock
            {
                Style = (Style)Application.Current.Resources["PezCaptionStyle"],
                Text = string.IsNullOrEmpty(item.Remark) ? "目录：（无）" : "目录：" + item.Remark,
                TextWrapping = TextWrapping.Wrap,
            });

            var dialog = new PezDialog
            {
                Title = "编辑资料",
                Content = panel,
                PrimaryButtonText = "保存",
                CloseButtonText = "取消",
                DefaultButton = ContentDialogButton.Primary,
            };
            if (await dialog.ShowAsync() != ContentDialogResult.Primary) return;

            if (!long.TryParse((yearBox.Text ?? "").Trim(), out var year)) year = 0;
            var type = typeBox.SelectedIndex + 1;

            // 服务端这段校验比上传松（不判科目字节数、不判年份），客户端仍沿用上传那套：
            // 更松会造出「客户端放行、服务端 400」。
            var verdict = UploadValidation.ValidateFileMeta(nameBox.Text, subjectBox.Text, schoolBox.Text, year, type);
            if (!verdict.Accepted)
            {
                ShowMine(verdict.Reason ?? "表单填写有误", muxc.InfoBarSeverity.Error);
                return;
            }

            // 整组回填：remark 原样带走，否则树里那条自定义目录会被清空。
            var request = FileUpdateRequest.From(item);
            request.FileName = (nameBox.Text ?? "").Trim();
            request.FileSubject = (subjectBox.Text ?? "").Trim();
            request.FileSchool = (schoolBox.Text ?? "").Trim();
            request.FileYear = year;
            request.FileType = type;

            try
            {
                await AppServices.FilesApi.UpdateAsync(request);
                ShowMine("已保存。编辑不会重新送审，状态保持原样。", muxc.InfoBarSeverity.Success);
                await LoadMineAsync();
            }
            catch (DatumException ex)
            {
                ShowMine(ex.Message, muxc.InfoBarSeverity.Error);
            }
            catch (Exception)
            {
                ShowMine("保存失败，请稍后重试", muxc.InfoBarSeverity.Error);
            }
        }

        private async void DeleteMineClick(object sender, RoutedEventArgs e)
        {
            if (!(sender is FrameworkElement element && element.Tag is FileItem item)) return;

            var dialog = new PezDialog
            {
                Title = "删除这份资料？",
                Content = item.DisplayName + "\n\n删除后不再出现在资料库与「我的上传」里；已经产生的下载记录不会消失。",
                PrimaryButtonText = "删除",
                CloseButtonText = "取消",
                DefaultButton = ContentDialogButton.Close,
            };
            if (await dialog.ShowAsync() != ContentDialogResult.Primary) return;

            try
            {
                await AppServices.FilesApi.DeleteAsync(item.FileId);
                ShowMine("已删除", muxc.InfoBarSeverity.Success);
                await LoadMineAsync();
            }
            catch (DatumException ex)
            {
                ShowMine(ex.Message, muxc.InfoBarSeverity.Error);
            }
            catch (Exception)
            {
                ShowMine("删除失败，请稍后重试", muxc.InfoBarSeverity.Error);
            }
        }

        private async void PreviewMineClick(object sender, RoutedEventArgs e)
        {
            if (!(sender is FrameworkElement element && element.Tag is FileItem item)) return;

            if (!string.Equals(item.FileFormat, "pdf", StringComparison.OrdinalIgnoreCase))
            {
                ShowMine("这份格式暂不支持内置预览：下载后用系统程序打开。", muxc.InfoBarSeverity.Informational);
                return;
            }

            try
            {
                // 待审件只有属主能取流（`datum_activity.go:530`：未通过且非属主 → 404）。
                var stream = await DownloadService.ReadStreamAsync(
                    ct => AppServices.Downloads.OpenFileStreamAsync(item.FileId, ct));

                ReaderOverlay.Visibility = Visibility.Visible;
                await PdfHost.OpenAsync(
                    stream,
                    item.FileId,
                    item.DisplayName,
                    AppServices.Settings.PdfView,
                    AppServices.Settings.PdfScale > 0 ? AppServices.Settings.PdfScale : null);
            }
            catch (Exception ex)
            {
                CloseReader();
                ShowMine(ex is DatumException datum ? datum.Message : "预览失败", muxc.InfoBarSeverity.Error);
            }
        }

        private void CloseReader()
        {
            PdfHost.Unload();
            ReaderOverlay.Visibility = Visibility.Collapsed;
        }

        private void ClearClick(object sender, RoutedEventArgs e)
        {
            if (_running) return;

            // 已经传完的行也一起清掉，免得重试时再传一遍。
            _items.Clear();
            SummaryText.Text = string.Empty;
            UploadBar.IsOpen = false;
        }

        private UploadMeta? BuildMeta()
        {
            var subject = (SubjectBox.Text ?? "").Trim();
            var school = (SchoolBox.Text ?? "").Trim();
            if (!long.TryParse((YearBox.Text ?? "").Trim(), out var year)) year = 0;
            var type = TypeBox.SelectedIndex + 1;

            // 文件名占位即可：这里要判的是科目 / 学校 / 年份 / 类型。
            var verdict = UploadValidation.ValidateFileMeta("占位.pdf", subject, school, year, type);
            if (!verdict.Accepted)
            {
                ShowBar(verdict.Reason ?? "表单填写有误", muxc.InfoBarSeverity.Error);
                return null;
            }

            if (verdict.WarnsNotEbook)
            {
                ShowBar("类型选「电子书」的资料不会出现在电子书列表里，建议走「电子书 → 上传」。",
                    muxc.InfoBarSeverity.Warning);
            }

            return new UploadMeta { Subject = subject, School = school, Year = year, Type = type };
        }

        private void ReportSummary()
        {
            var done = _items.Count(item => item.State == UploadState.Done);
            var failed = _items.Count(item => item.State == UploadState.Failed);
            var rejected = _items.Count(item => item.State == UploadState.Rejected);

            SummaryText.Text = "共 " + _items.Count + " 个：成功 " + done + "，失败 " + failed + "，未通过校验 " + rejected;

            if (failed > 0 || rejected > 0)
            {
                ShowBar("有文件没传成功：每个文件的结果写在它自己那一行，坏文件不影响其它文件。",
                    muxc.InfoBarSeverity.Warning);
            }
            else if (done > 0)
            {
                ShowBar("已提交 " + done + " 个文件，审核通过后公开。", muxc.InfoBarSeverity.Success);
            }
        }

        private void SetBusy(bool busy)
        {
            BusyRing.IsActive = busy;
            BusyRing.Visibility = busy ? Visibility.Visible : Visibility.Collapsed;
            StartButton.IsEnabled = !busy;
            ClearButton.IsEnabled = !busy;
            CancelButton.Visibility = busy ? Visibility.Visible : Visibility.Collapsed;
        }

        private bool EnsureLoggedIn()
        {
            if (AppServices.IsLoggedIn) return true;
            ShowBar("上传需要登录：先登录，再回来挑文件。", muxc.InfoBarSeverity.Warning);
            return false;
        }

        private void ShowBar(string message, muxc.InfoBarSeverity severity)
        {
            UploadBar.Severity = severity;
            UploadBar.Message = message;
            UploadBar.IsOpen = true;
        }

        private void ShowMine(string message, muxc.InfoBarSeverity severity)
        {
            MineBar.Severity = severity;
            MineBar.Message = message;
            MineBar.IsOpen = true;
        }
    }
}
