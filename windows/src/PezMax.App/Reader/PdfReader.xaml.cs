using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Runtime.CompilerServices;
using System.Threading.Tasks;
using PezMax.Core.Reading;
using Windows.Data.Pdf;
using Windows.Graphics.Display;
using Windows.Storage.Streams;
using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Media.Imaging;

namespace PezMax.Reader
{
    /// <summary>PDF 的两种视图：缩略图网格（挑页）与纵向连续滚动（读）。</summary>
    public enum PdfViewMode
    {
        Grid,
        Continuous,
    }

    /// <summary>网格、左侧概览与连续视图共用的一页。</summary>
    public sealed class PdfPageItem : INotifyPropertyChanged
    {
        private BitmapImage? _image;
        private double _top;
        private double _hostWidth;
        private double _hostHeight;
        private double _tileWidth;
        private double _tileHeight;

        public PdfPageItem(int index)
        {
            Index = index;
            Label = (index + 1).ToString();
        }

        public int Index { get; }

        public string Label { get; }

        /// <summary>有没有请求过缩略图（防止虚化的容器反复入队）。</summary>
        internal bool ThumbnailRequested { get; set; }

        public BitmapImage? Image
        {
            get => _image;
            set
            {
                if (ReferenceEquals(_image, value)) return;
                _image = value;
                Raise();
                Raise(nameof(PlaceholderVisibility));
            }
        }

        /// <summary>还没有位图时显示页号占位。</summary>
        public Visibility PlaceholderVisibility => _image == null ? Visibility.Visible : Visibility.Collapsed;

        /// <summary>连续视图里的顶部偏移（DIP）。</summary>
        public double Top
        {
            get => _top;
            set => Set(ref _top, value);
        }

        public double HostWidth
        {
            get => _hostWidth;
            set => Set(ref _hostWidth, value);
        }

        public double HostHeight
        {
            get => _hostHeight;
            set => Set(ref _hostHeight, value);
        }

        public double TileWidth
        {
            get => _tileWidth;
            set => Set(ref _tileWidth, value);
        }

        public double TileHeight
        {
            get => _tileHeight;
            set => Set(ref _tileHeight, value);
        }

        public event PropertyChangedEventHandler? PropertyChanged;

        private void Set(ref double field, double value, [CallerMemberName] string? name = null)
        {
            if (Math.Abs(field - value) < 0.001) return;
            field = value;
            Raise(name);
        }

        private void Raise([CallerMemberName] string? name = null) =>
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(name));
    }

    /// <summary>
    /// PDF 阅读器：网格（缩略图挑页）与连续（纵向滚动 + 左侧概览）两种视图。
    /// 渲染窗口、页缓存键、缩放步进都由 <see cref="PdfLayout"/> 决定（`spec/fixtures/reading/`），
    /// 这里只负责把算出来的结果贴到界面上。
    /// </summary>
    public sealed partial class PdfReader : UserControl
    {
        private const double PageGap = 12;
        private const double PagePadding = 24;
        private const double TileLabelHeight = 22;

        private readonly ObservableCollection<PdfPageItem> _items = new();
        private readonly HashSet<int> _pending = new();

        private InMemoryRandomAccessStream? _stream;
        private PdfDocument? _document;
        private PdfPageCache? _cache;
        private PdfPageSize[] _sizes = Array.Empty<PdfPageSize>();
        private PdfContinuousLayout _layout = PdfContinuousLayout.Empty;
        private PageSpan _renderSpan = PageSpan.Empty;
        private PdfViewMode _mode = PdfViewMode.Continuous;
        private double _zoom;
        private double _pageWidth = PdfPageSize.DefaultWidth;
        private double _maxAspect = PdfPageSize.DefaultHeight / PdfPageSize.DefaultWidth;
        private double _rasterScale = 1.0;
        private double _lastFitWidth = -1;
        private bool _autoFit = true;
        private bool _opened;

        public PdfReader()
        {
            InitializeComponent();

            PageItemsControl.ItemsSource = _items;
            GridHost.ItemsSource = _items;
            OverviewList.ItemsSource = _items;

            try
            {
                _rasterScale = Math.Max(1.0, DisplayInformation.GetForCurrentView().LogicalDpi / 96.0);
            }
            catch (Exception)
            {
                _rasterScale = 1.0;
            }

            SizeChanged += OnSizeChanged;
            Unloaded += (_, __) => Unload();
        }

        /// <summary>用户点了「关闭」。</summary>
        public event EventHandler? CloseRequested;

        public PdfViewMode View => _mode;

        /// <summary>
        /// 打开一份 PDF。预览一律先用整段字节流（下载流或 <c>/datum/ebook/content</c>），阅读器不直接拿 URL。
        /// </summary>
        /// <param name="defaultView">设置里的默认视图（<c>grid</c> / <c>continuous</c>）；不传按连续。</param>
        /// <param name="zoom">默认缩放；不传或 0 表示适应宽度。</param>
        public async Task OpenAsync(
            InMemoryRandomAccessStream stream,
            long fileId,
            string title,
            string? defaultView = null,
            double? zoom = null)
        {
            Unload();
            TitleText.Text = title;
            SetBusy(true);

            try
            {
                _stream = stream;
                _document = await PdfDocument.LoadFromStreamAsync(stream);
                var count = (int)_document.PageCount;
                if (count == 0)
                {
                    ShowError("这份 PDF 没有可显示的页面");
                    return;
                }

                var sizes = new PdfPageSize[count];
                for (var i = 0; i < count; i++)
                {
                    using var page = _document.GetPage((uint)i);
                    sizes[i] = new PdfPageSize(page.Size.Width, page.Size.Height);
                }

                _sizes = sizes;
                _pageWidth = Max(sizes, size => size.Width);
                _maxAspect = Max(sizes, size => size.Aspect);
                _cache = new PdfPageCache(_document, fileId);

                for (var i = 0; i < count; i++) _items.Add(new PdfPageItem(i));

                _mode = PezMax.Core.AppSettings.NormalizeView(defaultView) == "grid"
                    ? PdfViewMode.Grid
                    : PdfViewMode.Continuous;
                if (zoom is > 0)
                {
                    _zoom = PdfLayout.ClampZoom(zoom.Value);
                    _autoFit = false;
                }
                else
                {
                    _autoFit = true;
                }

                _opened = true;
                ApplyMode();
            }
            catch (Exception)
            {
                ShowError("PDF 打开失败，文件可能损坏或格式不被支持");
            }
            finally
            {
                SetBusy(false);
            }
        }

        /// <summary>释放文档、取消在飞的渲染；重复调用无副作用。</summary>
        public void Unload()
        {
            _opened = false;
            _cache?.Invalidate();
            _cache = null;
            _document = null;
            _stream?.Dispose();
            _stream = null;
            _sizes = Array.Empty<PdfPageSize>();
            _layout = PdfContinuousLayout.Empty;
            _renderSpan = PageSpan.Empty;
            _pending.Clear();
            _items.Clear();
            PageItemsControl.Width = double.NaN;
            PageItemsControl.Height = double.NaN;
            PageText.Text = string.Empty;
            TitleText.Text = string.Empty;
            ErrorBar.IsOpen = false;
        }

        private void ApplyMode()
        {
            var continuous = _mode == PdfViewMode.Continuous;
            GridHost.Visibility = continuous ? Visibility.Collapsed : Visibility.Visible;
            ContinuousHost.Visibility = continuous ? Visibility.Visible : Visibility.Collapsed;
            GridModeButton.IsChecked = !continuous;
            LineModeButton.IsChecked = continuous;
            OverviewButton.IsEnabled = continuous;
            ZoomText.Visibility = continuous ? Visibility.Visible : Visibility.Collapsed;

            if (!_opened) return;
            if (continuous) ApplyContinuousLayout();
            else ApplyGridLayout();
        }

        private void ApplyContinuousLayout()
        {
            if (!_opened) return;

            UpdateLayout();
            var viewport = Scroll.ViewportWidth > 0 ? Scroll.ViewportWidth : ActualWidth;
            var fitWidth = Math.Max(120, viewport - (PagePadding * 2));
            _lastFitWidth = fitWidth;
            if (_autoFit) _zoom = PdfLayout.FitZoom(_pageWidth, fitWidth);
            if (_zoom <= 0) _zoom = 1.0;

            var displayWidth = Math.Max(80, _pageWidth * _zoom);
            _layout = PdfContinuousLayout.Create(_sizes, displayWidth, PageGap);

            PageItemsControl.Width = displayWidth;
            PageItemsControl.Height = Math.Max(1, _layout.ContentHeight);

            for (var i = 0; i < _items.Count; i++)
            {
                var item = _items[i];
                item.HostWidth = displayWidth;
                item.HostHeight = _layout.Heights[i];
                item.Top = _layout.Tops[i];
            }

            // 缩放变了：在飞的渲染按旧尺度，结果不再贴到界面上。
            _cache?.Invalidate();
            foreach (var item in _items) item.Image = null;
            _pending.Clear();
            _renderSpan = PageSpan.Empty;

            UpdateZoomText();
            RefreshWindow();
        }

        private void ApplyGridLayout()
        {
            var viewport = ActualWidth > 0 ? ActualWidth : 800;
            var tile = PdfLayout.GridTileWidth(Math.Max(160, viewport - 8), PdfLayout.GridColumns(Math.Max(160, viewport - 8)));
            var tileHeight = (tile * _maxAspect) + TileLabelHeight;

            foreach (var item in _items)
            {
                item.TileWidth = tile;
                item.TileHeight = tileHeight;
            }
        }

        private void RefreshWindow()
        {
            if (!_opened || _mode != PdfViewMode.Continuous) return;
            if (_layout.Count == 0)
            {
                PageText.Text = string.Empty;
                return;
            }

            var offset = Scroll.VerticalOffset;
            var viewport = Scroll.ViewportHeight;
            var visible = _layout.Visible(offset, viewport);

            var current = _layout.CurrentPage(offset, viewport);
            if (current >= 0) PageText.Text = (current + 1) + " / " + _layout.Count;

            var window = PdfLayout.WithOverscan(visible, _layout.Count);
            if (window.Equals(_renderSpan)) return;
            _renderSpan = window;

            foreach (var index in PdfLayout.PagesToRelease(LoadedPages(), window)) DropPage(index);
            foreach (var index in PdfLayout.RenderOrder(visible, window, LoadedPages())) RequestPage(index);
        }

        private IEnumerable<int> LoadedPages()
        {
            foreach (var item in _items)
            {
                if (item.Image != null) yield return item.Index;
            }
        }

        private void RequestPage(int index)
        {
            if (_cache == null) return;
            if (index < 0 || index >= _items.Count) return;
            if (!_pending.Add(index)) return;

            _ = RenderPageAsync(_items[index], _cache.Generation);
        }

        private async Task RenderPageAsync(PdfPageItem item, int generation)
        {
            try
            {
                var bitmap = await _cache!.RenderAsync(item.Index, _zoom * _rasterScale);
                if (generation != _cache.Generation) return;
                if (_mode != PdfViewMode.Continuous) return;
                item.Image = bitmap;
            }
            catch (Exception)
            {
                // 单页渲染失败：留页号占位，不影响其余页。
            }
            finally
            {
                _pending.Remove(item.Index);
            }
        }

        private void DropPage(int index)
        {
            if (index < 0 || index >= _items.Count) return;
            var item = _items[index];
            item.Image = null;

            // 下次进网格视图要能重新拉缩略图。
            item.ThumbnailRequested = false;
        }

        private void RequestThumbnail(PdfPageItem item)
        {
            if (_cache == null || item.ThumbnailRequested || item.Image != null) return;
            item.ThumbnailRequested = true;
            _ = LoadThumbnailAsync(item, _cache.Generation);
        }

        private async Task LoadThumbnailAsync(PdfPageItem item, int generation)
        {
            try
            {
                var scale = (PdfLayout.ThumbnailWidth / Math.Max(1, _pageWidth)) * _rasterScale;
                var bitmap = await _cache!.RenderAsync(item.Index, scale);
                if (generation != _cache.Generation)
                {
                    item.ThumbnailRequested = false;
                    return;
                }

                if (item.Image == null) item.Image = bitmap;
            }
            catch (Exception)
            {
                item.ThumbnailRequested = false;
            }
        }

        private void GoToPage(int index)
        {
            if (!_opened) return;

            _mode = PdfViewMode.Continuous;
            ApplyMode();
            UpdateLayout();
            Scroll.ChangeView(null, Math.Max(0, _layout.OffsetOf(index)), null, true);
            RefreshWindow();
        }

        private void ScrollViewChanged(object sender, ScrollViewerViewChangedEventArgs e) => RefreshWindow();

        private void OnSizeChanged(object sender, SizeChangedEventArgs e)
        {
            if (!_opened) return;
            if (_mode == PdfViewMode.Grid)
            {
                ApplyGridLayout();
                return;
            }

            if (!_autoFit)
            {
                RefreshWindow();
                return;
            }

            // 适应宽度时窗口一改就要重排；纯高度变化只需补渲。
            var fitWidth = Math.Max(120, Scroll.ViewportWidth - (PagePadding * 2));
            if (Math.Abs(fitWidth - _lastFitWidth) > 1) ApplyContinuousLayout();
            else RefreshWindow();
        }

        private void PageContainerContentChanging(ListViewBase sender, ContainerContentChangingEventArgs args)
        {
            if (args.InRecycleQueue) return;
            if (args.Item is PdfPageItem item) RequestThumbnail(item);
        }

        private void GridItemClick(object sender, ItemClickEventArgs e)
        {
            if (e.ClickedItem is PdfPageItem item) GoToPage(item.Index);
        }

        private void OverviewItemClick(object sender, ItemClickEventArgs e)
        {
            if (e.ClickedItem is PdfPageItem item) GoToPage(item.Index);
        }

        private void GridModeClick(object sender, RoutedEventArgs e)
        {
            _mode = PdfViewMode.Grid;
            ApplyMode();
        }

        private void LineModeClick(object sender, RoutedEventArgs e)
        {
            _mode = PdfViewMode.Continuous;
            ApplyMode();
        }

        private void OverviewClick(object sender, RoutedEventArgs e)
        {
            var open = OverviewButton.IsChecked == true;
            OverviewList.Visibility = open ? Visibility.Visible : Visibility.Collapsed;
            OverviewColumn.Width = open ? new GridLength(PdfLayout.OverviewPanelWidth) : new GridLength(0);
        }

        private void ZoomInClick(object sender, RoutedEventArgs e) => ZoomBy(1);

        private void ZoomOutClick(object sender, RoutedEventArgs e) => ZoomBy(-1);

        private void ZoomBy(int direction)
        {
            if (!_opened) return;
            if (_mode != PdfViewMode.Continuous)
            {
                _mode = PdfViewMode.Continuous;
                ApplyMode();
            }

            _autoFit = false;
            _zoom = PdfLayout.StepZoom(_zoom, direction);
            ApplyContinuousLayout();
        }

        private void FitWidthClick(object sender, RoutedEventArgs e)
        {
            if (!_opened) return;
            _autoFit = true;
            if (_mode != PdfViewMode.Continuous) _mode = PdfViewMode.Continuous;
            ApplyMode();
        }

        private void ResetZoomClick(object sender, RoutedEventArgs e)
        {
            if (!_opened) return;
            _autoFit = false;
            _zoom = 1.0;
            if (_mode != PdfViewMode.Continuous) _mode = PdfViewMode.Continuous;
            ApplyMode();
        }

        private void CloseClick(object sender, RoutedEventArgs e) => CloseRequested?.Invoke(this, EventArgs.Empty);

        private void UpdateZoomText() => ZoomText.Text = Math.Round(_zoom * 100) + "%";

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

        private static double Max(PdfPageSize[] sizes, Func<PdfPageSize, double> select)
        {
            var result = 0d;
            foreach (var size in sizes)
            {
                var value = select(size);
                if (value > result) result = value;
            }

            return result > 0 ? result : 1;
        }
    }
}
