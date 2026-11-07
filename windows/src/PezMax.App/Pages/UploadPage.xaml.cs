using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using PezMax.Core.Models;
using PezMax.Core.Net;
using PezMax.Core.Validation;
using PezMax.Transfer;
using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Navigation;
using muxc = Microsoft.UI.Xaml.Controls;

namespace PezMax.Pages
{
    /// <summary>
    /// 上传资料：挑文件 / 挑文件夹 → 逐个本地校验 → 串行上传 → 逐行结果。
    /// 校验规则在 Core 的 <see cref="UploadValidation"/> / <see cref="Utf8Check"/>，这里只做表单与呈现。
    /// </summary>
    public sealed partial class UploadPage : Page
    {
        private readonly ObservableCollection<UploadItem> _items = new();
        private CancellationTokenSource? _cancel;
        private bool _running;

        public UploadPage()
        {
            InitializeComponent();

            FileList.ItemsSource = _items;
            TypeBox.ItemsSource = FileTypes.Names;
            TypeBox.SelectedIndex = 0;
            YearBox.Text = DateTime.Now.Year.ToString();
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
    }
}
