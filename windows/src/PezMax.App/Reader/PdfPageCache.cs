using System;
using System.Threading;
using System.Threading.Tasks;
using PezMax.Core.Reading;
using Windows.Data.Pdf;
using Windows.Storage;
using Windows.Storage.Streams;
using Windows.UI.Xaml.Media.Imaging;

namespace PezMax.Reader
{
    /// <summary>
    /// 页位图的渲染与磁盘缓存：同时最多渲 <see cref="PdfLayout.MaxConcurrentRenders"/> 页，
    /// 结果落在 <c>LocalCache/pdf/{fileId}/p0007_s0150.png</c>，下次打开直接读盘。
    /// 所有方法都在 UI 线程上跑（<see cref="BitmapImage"/> 不能跨线程创建），并发只靠信号量限制。
    /// </summary>
    internal sealed class PdfPageCache
    {
        private readonly PdfDocument _document;
        private readonly long _fileId;
        private readonly SemaphoreSlim _gate = new(PdfLayout.MaxConcurrentRenders, PdfLayout.MaxConcurrentRenders);

        private StorageFolder? _folder;
        private bool _folderResolved;
        private int _generation;

        public PdfPageCache(PdfDocument document, long fileId)
        {
            _document = document;
            _fileId = fileId;
        }

        /// <summary>换缩放 / 换文档时递增；在飞的渲染看到代数变了就不再贴到界面上（但照旧落盘）。</summary>
        public int Generation => _generation;

        public void Invalidate() => _generation++;

        /// <summary>取第 <paramref name="pageIndex"/> 页的位图，命中磁盘缓存时直接解码。</summary>
        public async Task<BitmapImage?> RenderAsync(int pageIndex, double scale, CancellationToken token = default)
        {
            var key = PdfLayout.PageCacheKey(pageIndex, scale);

            var cached = await TryReadAsync(key);
            if (cached != null) return cached;

            await _gate.WaitAsync(token);
            try
            {
                // 排队期间可能已经被别的请求渲好并落盘了，再查一次省一次渲染。
                cached = await TryReadAsync(key);
                if (cached != null) return cached;

                using var page = _document.GetPage((uint)pageIndex);
                var width = (uint)Math.Max(1, Math.Round(page.Size.Width * scale));
                using var stream = new InMemoryRandomAccessStream();
                await page.RenderToStreamAsync(stream, new PdfPageRenderOptions { DestinationWidth = width });
                await WriteAsync(key, stream);

                stream.Seek(0);
                var bitmap = new BitmapImage();
                await bitmap.SetSourceAsync(stream);
                return bitmap;
            }
            finally
            {
                _gate.Release();
            }
        }

        /// <summary>删掉某本书的整份页缓存（设置里的「清除缓存」用得上）。</summary>
        public static async Task ClearAsync(long fileId)
        {
            try
            {
                var pdf = await ApplicationData.Current.LocalCacheFolder
                    .CreateFolderAsync("pdf", CreationCollisionOption.OpenIfExists);
                var folder = await pdf.TryGetItemAsync(fileId.ToString()) as StorageFolder;
                if (folder != null) await folder.DeleteAsync(StorageDeleteOption.PermanentDelete);
            }
            catch (Exception)
            {
            }
        }

        private async Task<BitmapImage?> TryReadAsync(string key)
        {
            var folder = await EnsureFolderAsync();
            if (folder == null) return null;

            try
            {
                if (!(await folder.TryGetItemAsync(key + ".png") is StorageFile file)) return null;
                using var stream = await file.OpenAsync(FileAccessMode.Read);
                if (stream.Size == 0) return null;

                var bitmap = new BitmapImage();
                await bitmap.SetSourceAsync(stream);
                return bitmap;
            }
            catch (Exception)
            {
                // 半截文件 / 解码失败：当成没缓存，重新渲一遍。
                return null;
            }
        }

        private async Task WriteAsync(string key, IRandomAccessStream source)
        {
            var folder = await EnsureFolderAsync();
            if (folder == null || source.Size == 0) return;

            try
            {
                var size = (uint)source.Size;
                var buffer = new Windows.Storage.Streams.Buffer(size) { Length = size };
                source.Seek(0);
                await source.ReadAsync(buffer, size, InputStreamOptions.None);
                source.Seek(0);

                var file = await folder.CreateFileAsync(key + ".png", CreationCollisionOption.ReplaceExisting);
                await FileIO.WriteBufferAsync(file, buffer);
            }
            catch (Exception)
            {
                // 缓存写不进去不该影响阅读：本次照样显示刚渲好的位图。
            }
        }

        private async Task<StorageFolder?> EnsureFolderAsync()
        {
            if (_folderResolved) return _folder;

            try
            {
                var pdf = await ApplicationData.Current.LocalCacheFolder
                    .CreateFolderAsync("pdf", CreationCollisionOption.OpenIfExists);
                _folder = await pdf.CreateFolderAsync(_fileId.ToString(), CreationCollisionOption.OpenIfExists);
            }
            catch (Exception)
            {
                _folder = null;
            }

            _folderResolved = true;
            return _folder;
        }
    }
}
