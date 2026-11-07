using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Threading;
using System.Threading.Tasks;
using PezMax.Core.Models;
using PezMax.Core.Net;
using PezMax.Core.Transfer;
using Windows.Storage;
using Windows.Storage.Streams;

namespace PezMax.Transfer
{
    internal sealed class DownloadProgress
    {
        public string FileName { get; init; } = "";
        public long BytesReceived { get; init; }
        public long Total { get; init; }
        public double Fraction => Total > 0 ? Math.Min(1d, (double)BytesReceived / Total) : 0d;
    }

    /// <summary>
    /// 下载：`.part` 半成品 + `Range` / `If-Range` 续传 + 串行队列 + 本地记录。
    /// <para>
    /// 判定全部交给 Core 的 <see cref="DownloadPlanner"/>（跑 `spec/fixtures/download/`），这里只负责搬字节与落盘：
    /// 半成品叫 <c>&lt;最终文件名&gt;.part</c>，ETag 记在 `downloads.json` 里，不再单独存一个元数据文件。
    /// </para>
    /// <para>
    /// 队列默认**串行**：经 edge 部署时每 IP 只允许 1 个下载连接（`deploy/nginx.conf.template:86-93`），
    /// 并发只会被拒；而且下载与封面 / 头像共用同一个 bulk 令牌桶，抢也抢不出速度。
    /// </para>
    /// </summary>
    internal static class DownloadService
    {
        private static readonly SemaphoreSlim Queue = new(1, 1);
        private static readonly object Gate = new();
        private static List<DownloadRecord> _records = new();
        private static bool _loaded;

        public static async Task<StorageFile> DownloadFileAsync(
            FileItem file,
            IProgress<DownloadProgress>? progress = null,
            CancellationToken cancellationToken = default)
        {
            var name = SanitizeFileName(file.DisplayName);
            return await TransferAsync(
                DownloadKinds.File,
                file.FileId,
                name,
                (range, ifRange, token) => AppServices.Downloads.OpenFileStreamAsync(file.FileId, range, ifRange, token),
                progress,
                cancellationToken).ConfigureAwait(false);
        }

        public static async Task<StorageFile> DownloadEbookAsync(
            EbookItem ebook,
            IProgress<DownloadProgress>? progress = null,
            CancellationToken cancellationToken = default)
        {
            var name = SanitizeFileName(ebook.DisplayName);
            if (!string.IsNullOrWhiteSpace(ebook.EbookFormat) &&
                !name.EndsWith("." + ebook.EbookFormat, StringComparison.OrdinalIgnoreCase))
            {
                name += "." + ebook.EbookFormat;
            }

            return await TransferAsync(
                DownloadKinds.Ebook,
                ebook.EbookId,
                name,
                (range, ifRange, token) => AppServices.Downloads.OpenEbookStreamAsync(ebook.EbookId, range, ifRange, token),
                progress,
                cancellationToken).ConfigureAwait(false);
        }

        /// <summary>「我的下载」：服务端行 + 本地记录合并（规则见 `spec/fixtures/download/README.md`）。</summary>
        public static async Task<IReadOnlyList<DownloadRow>> GetMergedDownloadsAsync(
            long userId,
            int pageNum = 1,
            int pageSize = 20,
            CancellationToken cancellationToken = default)
        {
            var server = await AppServices.Downloads.GetMyDownloadsAsync(userId, pageNum, pageSize, cancellationToken)
                .ConfigureAwait(false);
            return DownloadRecords.Merge(server.Rows, await LoadRecordsAsync().ConfigureAwait(false));
        }

        public static async Task<IReadOnlyList<DownloadRecord>> LoadRecordsAsync()
        {
            lock (Gate)
            {
                if (_loaded) return _records.ToList();
            }

            var json = await AppServices.Files.ReadTextAsync(DownloadRecords.FileName).ConfigureAwait(false);
            var parsed = DownloadRecords.Parse(json).ToList();

            lock (Gate)
            {
                _records = parsed;
                _loaded = true;
                return _records.ToList();
            }
        }

        public static async Task RemoveRecordAsync(string kind, long id)
        {
            var records = await LoadRecordsAsync().ConfigureAwait(false);
            await SaveRecordsAsync(records.Where(r => !(r.Kind == kind && r.Id == id)).ToList()).ConfigureAwait(false);
        }

        /// <summary>预览用：把流整段读进内存（PDF / 图片阅读器不直接拿 URL）。</summary>
        public static async Task<InMemoryRandomAccessStream> ReadStreamAsync(
            Func<CancellationToken, Task<HttpResponseMessage>> open,
            CancellationToken cancellationToken = default)
        {
            using var response = await open(cancellationToken).ConfigureAwait(false);
            var source = await response.Content.ReadAsStreamAsync().ConfigureAwait(false);
            var memory = new InMemoryRandomAccessStream();
            using var output = memory.AsStreamForWrite();
            await source.CopyToAsync(output, 81920, cancellationToken).ConfigureAwait(false);
            await output.FlushAsync(cancellationToken).ConfigureAwait(false);
            memory.Seek(0);
            return memory;
        }

        private static async Task<StorageFile> TransferAsync(
            string kind,
            long id,
            string fileName,
            Func<string?, string?, CancellationToken, Task<HttpResponseMessage>> open,
            IProgress<DownloadProgress>? progress,
            CancellationToken cancellationToken)
        {
            await Queue.WaitAsync(cancellationToken).ConfigureAwait(false);
            try
            {
                return await TransferLockedAsync(kind, id, fileName, open, progress, cancellationToken)
                    .ConfigureAwait(false);
            }
            finally
            {
                Queue.Release();
            }
        }

        private static async Task<StorageFile> TransferLockedAsync(
            string kind,
            long id,
            string fileName,
            Func<string?, string?, CancellationToken, Task<HttpResponseMessage>> open,
            IProgress<DownloadProgress>? progress,
            CancellationToken cancellationToken)
        {
            var folder = ApplicationData.Current.LocalCacheFolder;
            var partFile = await folder.CreateFileAsync(fileName + ".part", CreationCollisionOption.OpenIfExists)
                .AsTask().ConfigureAwait(false);
            var record = FindRecord(kind, id);

            // 最多两轮：第一轮可能被判定要「丢弃半成品重下」（416 / ETag 变化），第二轮从头来。
            for (var attempt = 0; attempt < 2; attempt++)
            {
                var partSize = await LengthAsync(partFile).ConfigureAwait(false);
                var local = new DownloadPartState { Size = partSize, ETag = record?.ETag };

                var before = DownloadPlanner.BeforeRequest(local);
                if (before.DiscardPart || before.Action == DownloadAction.Restart)
                {
                    await partFile.DeleteAsync(StorageDeleteOption.PermanentDelete).AsTask().ConfigureAwait(false);
                    partFile = await folder.CreateFileAsync(fileName + ".part", CreationCollisionOption.OpenIfExists)
                        .AsTask().ConfigureAwait(false);
                    partSize = 0;
                    local = DownloadPartState.Empty;
                    before = DownloadPlanner.BeforeRequest(local);
                }

                HttpResponseMessage response;
                try
                {
                    response = await open(before.RangeHeader, before.IfRangeHeader, cancellationToken)
                        .ConfigureAwait(false);
                }
                catch (DatumException ex)
                {
                    // 416 也走异常分支：先让 Planner 判，再决定重下还是报错。
                    var failure = DownloadPlanner.Plan(local, new DownloadResponseInfo { Status = ex.HttpStatus });
                    if (failure.Action == DownloadAction.Restart && attempt == 0)
                    {
                        await partFile.DeleteAsync(StorageDeleteOption.PermanentDelete).AsTask().ConfigureAwait(false);
                        partFile = await folder.CreateFileAsync(fileName + ".part", CreationCollisionOption.OpenIfExists)
                            .AsTask().ConfigureAwait(false);
                        continue;
                    }

                    if (ex.HttpStatus != 416 && ex.HttpStatus != 404)
                    {
                        await TouchRecordAsync(kind, id, fileName, partFile.Path, partSize, record?.ETag,
                            DownloadStatuses.Failed).ConfigureAwait(false);
                    }

                    throw;
                }

                using (response)
                {
                    var info = new DownloadResponseInfo
                    {
                        Status = (int)response.StatusCode,
                        ETag = response.Headers.ETag?.ToString(),
                        ContentLength = response.Content.Headers.ContentLength ?? 0,
                        ContentRange = response.Content.Headers.ContentRange?.ToString(),
                    };

                    var decision = DownloadPlanner.Plan(local, info);
                    if (decision.Action == DownloadAction.Restart && attempt == 0)
                    {
                        await partFile.DeleteAsync(StorageDeleteOption.PermanentDelete).AsTask().ConfigureAwait(false);
                        partFile = await folder.CreateFileAsync(fileName + ".part", CreationCollisionOption.OpenIfExists)
                            .AsTask().ConfigureAwait(false);
                        continue;
                    }

                    if (decision.Action is DownloadAction.Fail or DownloadAction.SessionExpired)
                    {
                        await TouchRecordAsync(kind, id, fileName, partFile.Path, partSize, record?.ETag,
                            DownloadStatuses.Failed).ConfigureAwait(false);
                        throw new DatumException(
                            decision.Message ?? "下载失败",
                            info.Status,
                            0,
                            decision.Action == DownloadAction.SessionExpired,
                            decision.Action == DownloadAction.Fail);
                    }

                    var appended = decision.Action == DownloadAction.Resume;
                    await WriteAsync(response, partFile, appended, decision.Total, fileName, progress, cancellationToken)
                        .ConfigureAwait(false);

                    var written = await LengthAsync(partFile).ConfigureAwait(false);
                    var etag = info.ETag ?? record?.ETag;
                    if (decision.Total > 0 && written < decision.Total)
                    {
                        // 流提前结束但没到总大小：留半成品，下次续。
                        await TouchRecordAsync(kind, id, fileName, partFile.Path, written, etag, DownloadStatuses.Part)
                            .ConfigureAwait(false);
                        throw new IOException($"下载未完成（{written}/{decision.Total} 字节）");
                    }

                    var finalName = await CompleteAsync(partFile, fileName).ConfigureAwait(false);
                    var finalFile = await folder.GetFileAsync(finalName).AsTask().ConfigureAwait(false);
                    await TouchRecordAsync(kind, id, finalName, finalFile.Path, written, etag, DownloadStatuses.Done)
                        .ConfigureAwait(false);

                    progress?.Report(new DownloadProgress
                    {
                        FileName = fileName,
                        BytesReceived = written,
                        Total = written,
                    });

                    return finalFile;
                }
            }

            throw new IOException("下载失败：服务端两次都拒绝了分段请求");
        }

        private static async Task WriteAsync(
            HttpResponseMessage response,
            StorageFile partFile,
            bool append,
            long total,
            string fileName,
            IProgress<DownloadProgress>? progress,
            CancellationToken cancellationToken)
        {
            var input = await response.Content.ReadAsStreamAsync().ConfigureAwait(false);
            var target = await partFile.OpenAsync(FileAccessMode.ReadWrite).AsTask().ConfigureAwait(false);
            using var output = target.AsStreamForWrite();

            if (append)
            {
                output.Seek(0, SeekOrigin.End);
            }
            else
            {
                output.SetLength(0);
            }

            var received = output.Position;
            var buffer = new byte[81920];
            while (true)
            {
                var read = await input.ReadAsync(buffer, 0, buffer.Length, cancellationToken).ConfigureAwait(false);
                if (read <= 0) break;

                await output.WriteAsync(buffer, 0, read, cancellationToken).ConfigureAwait(false);
                received += read;

                progress?.Report(new DownloadProgress
                {
                    FileName = fileName,
                    BytesReceived = received,
                    Total = total,
                });

                if (DownloadPlanner.AfterWrite(received, total).Action == DownloadAction.Complete) break;
            }

            await output.FlushAsync(cancellationToken).ConfigureAwait(false);
        }

        private static async Task<string> CompleteAsync(StorageFile partFile, string fileName)
        {
            // 落盘改名：同目录内改名是原子的，用户不会看到半截文件。
            await partFile.RenameAsync(fileName, NameCollisionOption.GenerateUniqueName).AsTask().ConfigureAwait(false);
            return partFile.Name;
        }

        private static async Task<long> LengthAsync(StorageFile file)
        {
            try
            {
                var properties = await file.GetBasicPropertiesAsync().AsTask().ConfigureAwait(false);
                return (long)properties.Size;
            }
            catch (Exception)
            {
                return 0;
            }
        }

        private static DownloadRecord? FindRecord(string kind, long id)
        {
            lock (Gate)
            {
                return _records.FirstOrDefault(r => r.Kind == kind && r.Id == id);
            }
        }

        private static async Task TouchRecordAsync(
            string kind,
            long id,
            string fileName,
            string localPath,
            long bytesReceived,
            string? etag,
            string status)
        {
            var records = await LoadRecordsAsync().ConfigureAwait(false);
            var existing = records.FirstOrDefault(r => r.Kind == kind && r.Id == id);
            var baseRecord = existing ?? new DownloadRecord { Kind = kind, Id = id, FileName = fileName };

            var touched = baseRecord.WithProgress(bytesReceived, etag, status);
            var updated = new DownloadRecord
            {
                Kind = touched.Kind,
                Id = touched.Id,
                FileName = string.IsNullOrEmpty(fileName) ? touched.FileName : fileName,
                LocalPath = localPath,
                Size = touched.Size,
                BytesReceived = touched.BytesReceived,
                ETag = touched.ETag,
                Status = touched.Status,
                UpdatedAt = touched.UpdatedAt,
            };

            var list = records.Where(r => !(r.Kind == kind && r.Id == id)).ToList();
            list.Add(updated);
            await SaveRecordsAsync(list).ConfigureAwait(false);
        }

        private static async Task SaveRecordsAsync(List<DownloadRecord> records)
        {
            lock (Gate)
            {
                _records = records;
                _loaded = true;
            }

            await AppServices.Files.WriteTextAsync(DownloadRecords.FileName, DownloadRecords.ToJson(records))
                .ConfigureAwait(false);
        }

        private static string SanitizeFileName(string name)
        {
            var text = string.IsNullOrWhiteSpace(name) ? "download" : name.Trim();
            foreach (var c in Path.GetInvalidFileNameChars())
            {
                text = text.Replace(c, '_');
            }

            return text.Length switch
            {
                0 => "download",
                > 180 => text.Substring(0, 180),
                _ => text,
            };
        }
    }
}
