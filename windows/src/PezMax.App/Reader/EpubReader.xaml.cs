using System;
using System.Collections.Generic;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Microsoft.Web.WebView2.Core;
using PezMax.Core.Reading;
using Windows.Storage;
using Windows.Storage.Streams;
using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;

namespace PezMax.Reader
{
    /// <summary>目录里的一行；不给模板，直接靠 <see cref="ToString"/> 显示。</summary>
    public sealed class EpubTocEntry
    {
        public EpubTocEntry(int index, string label)
        {
            Index = index;
            Label = label;
        }

        public int Index { get; }

        public string Label { get; }

        public override string ToString() => Label;
    }

    /// <summary>
    /// EPUB 阅读器：解压到 <c>LocalCache/epub/{ebookId}/</c>，EPUB 结构由 Core 的 <see cref="EpubPackage"/> 解析，
    /// 正文与资源经虚拟主机 <c>pezmax.local</c> 交给 WebView2。一次显示一章（chromium 自己竖向滚动）。
    /// </summary>
    public sealed partial class EpubReader : UserControl
    {
        private const string HostName = "pezmax.local";
        private const string CacheFolderName = "epub";

        /// <summary>阅读区样式：限宽居中、行高放宽、图片不溢出；深色跟随系统。</summary>
        private const string ReaderCss =
            "html{background:transparent;}" +
            "body{margin:0 auto;max-width:44em;padding:28px 32px 64px;line-height:1.8;font-size:17px;" +
            "font-family:'Segoe UI','Microsoft YaHei UI','Microsoft YaHei',serif;color:#1b1f24;background:transparent;}" +
            "img,svg,video{max-width:100%;height:auto;}" +
            "a{color:#1a5fb4;}" +
            "pre,code{white-space:pre-wrap;word-break:break-word;}" +
            "@media (prefers-color-scheme: dark){body{color:#e8ecf1;}a{color:#8ab4f8;}}";

        private EpubBook? _book;
        private StorageFolder? _root;
        private int _index;
        private bool _ready;

        public EpubReader()
        {
            InitializeComponent();
            Unloaded += (_, __) => Unload();
        }

        public event EventHandler? CloseRequested;

        public async Task OpenAsync(InMemoryRandomAccessStream stream, long ebookId, string title)
        {
            Unload();
            ChapterText.Text = title;
            SetBusy(true);

            try
            {
                var texts = new Dictionary<string, string>(StringComparer.Ordinal);
                _root = await ExtractAsync(stream, ebookId, texts);

                var book = EpubPackage.Parse(texts);
                if (!book.IsUsable)
                {
                    ShowError(book.Failure ?? "这本 EPUB 解析不出章节");
                    return;
                }

                _book = book;
                TocList.ItemsSource = BuildToc(book);

                if (!await EnsureWebViewAsync()) return;

                _ready = true;
                ShowChapter(0);
            }
            catch (Exception)
            {
                ShowError("EPUB 打开失败，文件可能损坏");
            }
            finally
            {
                SetBusy(false);
            }
        }

        /// <summary>释放解析结果；重复调用无副作用。</summary>
        public void Unload()
        {
            _book = null;
            _index = 0;
            _ready = false;
            TocList.ItemsSource = null;
            ChapterText.Text = string.Empty;
            ErrorBar.IsOpen = false;
        }

        private void ShowChapter(int index)
        {
            if (!_ready || _book == null || _book.Chapters.Count == 0) return;

            _index = Math.Clamp(index, 0, _book.Chapters.Count - 1);
            var chapter = _book.Chapters[_index];

            try
            {
                View.CoreWebView2?.Navigate("https://" + HostName + "/" + EncodePath(chapter.Path));
            }
            catch (Exception)
            {
                // 导航失败就在下一章兜底：不吞掉当前状态，用户点「下一章」还能继续。
            }

            PrevButton.IsEnabled = _index > 0;
            NextButton.IsEnabled = _index < _book.Chapters.Count - 1;

            var label = chapter.Title.Length > 0 ? chapter.Title : "第 " + (_index + 1) + " 章";
            ChapterText.Text = label + " · " + (_index + 1) + " / " + _book.Chapters.Count;
        }

        private static IReadOnlyList<EpubTocEntry> BuildToc(EpubBook book)
        {
            var entries = new List<EpubTocEntry>(book.Chapters.Count);
            foreach (var chapter in book.Chapters)
            {
                var label = chapter.Title.Length > 0 ? chapter.Title : "第 " + (chapter.Order + 1) + " 章";
                entries.Add(new EpubTocEntry(chapter.Order, label));
            }

            return entries;
        }

        private async Task<bool> EnsureWebViewAsync()
        {
            if (_root == null) return false;

            try
            {
                if (View.CoreWebView2 == null)
                {
                    // WinUI 2 的 WebView2 控件自己建环境（winmd 里没有带 environment 参数的重载），
                    // user data folder 由控件放在应用自己的目录下，UWP 容器里可写。
                    await View.EnsureCoreWebView2Async();
                }

                var core = View.CoreWebView2;
                if (core == null)
                {
                    ShowError("WebView2 初始化失败，暂时无法在线阅读");
                    return false;
                }

                core.SetVirtualHostNameToFolderMapping(HostName, _root.Path, CoreWebView2HostResourceAccessKind.Allow);
                core.Settings.AreDefaultContextMenusEnabled = false;
                core.Settings.IsStatusBarEnabled = false;

                var script = "(()=>{const s=document.createElement('style');s.textContent=" +
                             Quote(ReaderCss) +
                             ";document.head&&document.head.appendChild(s);})();";
                await core.AddScriptToExecuteOnDocumentCreatedAsync(script);
                return true;
            }
            catch (Exception)
            {
                // 常见原因：机器上没装 Evergreen WebView2 Runtime（Win10 早期版本）。
                ShowError("这台机器没有 WebView2 运行时，暂时不能在线阅读 EPUB：装一次 Microsoft Edge WebView2 Runtime 即可。");
                return false;
            }
        }

        private async Task<StorageFolder> ExtractAsync(
            InMemoryRandomAccessStream stream,
            long ebookId,
            IDictionary<string, string> texts)
        {
            var cache = ApplicationData.Current.LocalCacheFolder;
            var books = await cache.CreateFolderAsync(CacheFolderName, CreationCollisionOption.OpenIfExists);

            var name = ebookId.ToString();
            if (await books.TryGetItemAsync(name) is StorageFolder previous)
            {
                await previous.DeleteAsync(StorageDeleteOption.PermanentDelete);
            }

            var folder = await books.CreateFolderAsync(name);
            var rootPath = folder.Path;

            using var zipStream = stream.AsStreamForRead();
            using var archive = new ZipArchive(zipStream, ZipArchiveMode.Read, leaveOpen: true);

            foreach (var entry in archive.Entries)
            {
                var entryName = entry.FullName.Replace('\\', '/');
                if (entryName.Length == 0 || entryName.EndsWith("/", StringComparison.Ordinal)) continue;
                if (entryName.StartsWith("__MACOSX/", StringComparison.OrdinalIgnoreCase)) continue;

                var target = Path.GetFullPath(Path.Combine(rootPath, entryName));
                // zip slip：条目名里带 ../ 时不要写到解压目录之外。
                if (!target.StartsWith(rootPath, StringComparison.OrdinalIgnoreCase)) continue;

                var directory = Path.GetDirectoryName(target);
                if (!string.IsNullOrEmpty(directory)) Directory.CreateDirectory(directory);

                using (var input = entry.Open())
                using (var output = File.Create(target))
                {
                    await input.CopyToAsync(output);
                }

                if (!IsTextEntry(entryName)) continue;

                using var reader = new StreamReader(entry.Open(), Encoding.UTF8, true);
                texts[entryName] = await reader.ReadToEndAsync();
            }

            return folder;
        }

        private static bool IsTextEntry(string path) =>
            path.EndsWith(".opf", StringComparison.OrdinalIgnoreCase) ||
            path.EndsWith(".xml", StringComparison.OrdinalIgnoreCase) ||
            path.EndsWith(".xhtml", StringComparison.OrdinalIgnoreCase) ||
            path.EndsWith(".html", StringComparison.OrdinalIgnoreCase) ||
            path.EndsWith(".htm", StringComparison.OrdinalIgnoreCase) ||
            path.EndsWith(".ncx", StringComparison.OrdinalIgnoreCase);

        /// <summary>路径进 URL：逐段百分号编码，中文文件名与空格才不会 404。</summary>
        private static string EncodePath(string path) =>
            string.Join("/", path.Split('/').Select(Uri.EscapeDataString));

        private static string Quote(string text) =>
            "\"" + text.Replace("\\", "\\\\").Replace("\"", "\\\"").Replace("\r", "").Replace("\n", " ") + "\"";

        private void PrevClick(object sender, RoutedEventArgs e) => ShowChapter(_index - 1);

        private void NextClick(object sender, RoutedEventArgs e) => ShowChapter(_index + 1);

        private void TocItemClick(object sender, ItemClickEventArgs e)
        {
            if (e.ClickedItem is EpubTocEntry entry) ShowChapter(entry.Index);
        }

        private void CloseClick(object sender, RoutedEventArgs e) => CloseRequested?.Invoke(this, EventArgs.Empty);

        private void SetBusy(bool busy)
        {
            BusyRing.IsActive = busy;
            BusyRing.Visibility = busy ? Visibility.Visible : Visibility.Collapsed;
        }

        private void ShowError(string message)
        {
            ErrorBar.Message = message;
            ErrorBar.IsOpen = true;
        }
    }
}
