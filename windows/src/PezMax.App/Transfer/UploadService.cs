using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.IO;
using System.Runtime.CompilerServices;
using System.Threading;
using System.Threading.Tasks;
using PezMax.Core.Models;
using PezMax.Core.Net;
using PezMax.Core.Validation;
using Windows.Storage;
using Windows.Storage.Pickers;

namespace PezMax.Transfer
{
    public enum UploadState
    {
        /// <summary>校验通过、等着上传。</summary>
        Pending,

        /// <summary>正在传（或服务端正在转档）。</summary>
        Uploading,

        /// <summary>已提交，等审核。</summary>
        Done,

        /// <summary>传失败，可以重试这一个。</summary>
        Failed,

        /// <summary>本地校验就没过，不要发请求。</summary>
        Rejected,
    }

    /// <summary>上传页列表里的一行。</summary>
    public sealed class UploadItem : INotifyPropertyChanged
    {
        private UploadState _state = UploadState.Pending;
        private string _status = "待上传";
        private double _progress;

        internal UploadItem(StorageFile file, string folderPath, long size, bool needsUtf8Check)
        {
            File = file;
            FolderPath = folderPath;
            Size = size;
            NeedsUtf8Check = needsUtf8Check;
        }

        public StorageFile File { get; }

        public string FileName => File.Name;

        /// <summary>文件夹上传时的相对目录；会作为 <c>remark</c> 变成树里的自定义目录。</summary>
        public string FolderPath { get; }

        public long Size { get; }

        public string SizeText => FileItem.FormatSize(Size);

        /// <summary>这一行要额外提醒的事（转档、编码检查、电子书走错门）。</summary>
        public string Hint { get; internal set; } = "";

        internal bool NeedsUtf8Check { get; }

        public UploadState State
        {
            get => _state;
            private set
            {
                if (_state == value) return;
                _state = value;
                Raise();
                Raise(nameof(CanRetry));
            }
        }

        public string Status
        {
            get => _status;
            private set
            {
                if (_status == value) return;
                _status = value;
                Raise();
            }
        }

        /// <summary>0–1，给 <c>ProgressBar.Value</c> 用。</summary>
        public double Progress
        {
            get => _progress;
            private set
            {
                if (Math.Abs(_progress - value) < 0.001) return;
                _progress = value;
                Raise();
            }
        }

        public bool CanRetry => _state is UploadState.Pending or UploadState.Failed;

        public event PropertyChangedEventHandler? PropertyChanged;

        internal void Begin()
        {
            Progress = 0;
            Status = "上传中…";
            State = UploadState.Uploading;
        }

        internal void Report(long sent)
        {
            Progress = Size > 0 ? Math.Min(1.0, sent / (double)Size) : 0;
        }

        internal void Complete(long fileId)
        {
            Progress = 1;
            Status = "已提交（fileId " + fileId + "），审核通过后公开";
            State = UploadState.Done;
        }

        internal void Fail(string reason)
        {
            Status = reason;
            State = UploadState.Failed;
        }

        internal void Reject(string reason)
        {
            Status = reason;
            State = UploadState.Rejected;
        }

        internal void Cancel()
        {
            Status = "已取消";
            State = UploadState.Pending;
        }

        private void Raise([CallerMemberName] string? name = null) =>
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(name));
    }

    /// <summary>上传资料时的元信息；一次批量共用一份。</summary>
    internal sealed class UploadMeta
    {
        public string Subject { get; init; } = "";
        public string School { get; init; } = "";
        public long Year { get; init; }
        public long Type { get; init; }
    }

    /// <summary>
    /// 上传资料（<c>POST /datum/file</c>）：挑文件 / 挑文件夹 → 逐个本地校验 → 串行上传。
    /// 服务端是单文件端点，所以「一个坏文件不拖垮整批」靠这里逐个判、逐个出结果。
    /// </summary>
    internal static class UploadService
    {
        /// <summary>多选文件。选择器按白名单开，但**不照抄**服务端那句只列 10 种的文案。</summary>
        public static async Task<IReadOnlyList<UploadItem>> PickFilesAsync()
        {
            var picker = new FileOpenPicker();
            foreach (var extension in UploadValidation.FileExtensions) picker.FileTypeFilter.Add("." + extension);

            var files = await picker.PickMultipleFilesAsync();
            if (files == null || files.Count == 0) return Array.Empty<UploadItem>();
            return await ToItemsAsync(files, "");
        }

        /// <summary>整个文件夹递归上传，相对目录作为 <c>remark</c>。</summary>
        public static async Task<IReadOnlyList<UploadItem>> PickFolderAsync()
        {
            var picker = new FolderPicker();
            picker.FileTypeFilter.Add("*");

            var folder = await picker.PickSingleFolderAsync();
            if (folder == null) return Array.Empty<UploadItem>();

            var found = new List<StorageFile>();
            var paths = new List<string>();
            await CollectAsync(folder, "", found, paths);

            var items = new List<UploadItem>();
            for (var i = 0; i < found.Count; i++)
            {
                items.Add(await CreateAsync(found[i], paths[i]));
            }

            return items;
        }

        /// <summary>串行上传：经 edge 时并发只会互抢，单个失败不影响其余。</summary>
        public static async Task RunAsync(
            IEnumerable<UploadItem> items,
            UploadMeta meta,
            CancellationToken cancellationToken)
        {
            foreach (var item in items)
            {
                if (!item.CanRetry) continue;
                cancellationToken.ThrowIfCancellationRequested();
                await UploadOneAsync(item, meta, cancellationToken);
            }
        }

        private static async Task UploadOneAsync(UploadItem item, UploadMeta meta, CancellationToken cancellationToken)
        {
            item.Begin();

            try
            {
                using var native = await item.File.OpenAsync(FileAccessMode.Read);
                using var stream = native.AsStreamForRead();

                // txt / md 服务端只收 UTF-8（否则 400）：本地先判，省一次往返也省一次「不知道为什么失败」。
                if (item.NeedsUtf8Check && !Utf8Check.IsUtf8(stream))
                {
                    item.Reject(Utf8Check.RejectMessage);
                    return;
                }

                stream.Seek(0, SeekOrigin.Begin);

                var contentType = string.IsNullOrWhiteSpace(item.File.ContentType)
                    ? "application/octet-stream"
                    : item.File.ContentType;

                var progress = new Progress<long>(sent => item.Report(sent));
                var result = await AppServices.FilesApi.UploadAsync(
                    item.FileName,
                    stream,
                    contentType,
                    meta.Subject,
                    meta.School,
                    meta.Year,
                    meta.Type,
                    string.IsNullOrEmpty(item.FolderPath) ? null : item.FolderPath,
                    cancellationToken,
                    progress);

                item.Complete(result.FileId);
            }
            catch (OperationCanceledException) when (cancellationToken.IsCancellationRequested)
            {
                item.Cancel();
                throw;
            }
            catch (TimeoutException)
            {
                // 转档走 LibreOffice，排队时会超过普通请求的 30 秒；超时不等于文件有问题。
                item.Fail("服务端转档排队超时，可以单独重试这一个");
            }
            catch (DatumException ex)
            {
                item.Fail(ex.Message);
            }
            catch (Exception)
            {
                item.Fail("上传失败，请稍后重试");
            }
        }

        private static async Task CollectAsync(
            StorageFolder folder,
            string relative,
            List<StorageFile> files,
            List<string> paths)
        {
            foreach (var file in await folder.GetFilesAsync())
            {
                files.Add(file);
                paths.Add(relative);
            }

            foreach (var child in await folder.GetFoldersAsync())
            {
                var path = string.IsNullOrEmpty(relative) ? child.Name : relative + "/" + child.Name;
                await CollectAsync(child, path, files, paths);
            }
        }

        private static async Task<List<UploadItem>> ToItemsAsync(IEnumerable<StorageFile> files, string folderPath)
        {
            var items = new List<UploadItem>();
            foreach (var file in files) items.Add(await CreateAsync(file, folderPath));
            return items;
        }

        private static async Task<UploadItem> CreateAsync(StorageFile file, string folderPath)
        {
            long size;
            try
            {
                size = (long)(await file.GetBasicPropertiesAsync()).Size;
            }
            catch (Exception)
            {
                size = 0;
            }

            var verdict = UploadValidation.ValidateFile(file.Name, size);
            var item = new UploadItem(file, folderPath, size, verdict.NeedsUtf8Check);

            if (!verdict.Accepted)
            {
                item.Reject(verdict.Reason ?? "文件不合格");
                return item;
            }

            if (verdict.ConvertsToPdf)
                item.Hint = "将转换为 PDF 存储，可能需要 1–2 分钟";
            else if (verdict.NeedsUtf8Check)
                item.Hint = "上传前会检查 UTF-8 编码";
            else if (verdict.WarnsNotEbook)
                item.Hint = "电子书请走「电子书」上传，这里只会进资料库";

            return item;
        }
    }
}
