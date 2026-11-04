using System;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using PezMax.Core.Models;
using PezMax.Core.Net;
using Windows.Storage;
using Windows.Storage.Streams;

namespace PezMax.Transfer
{
    internal static class DownloadService
    {
        public static async Task<StorageFile> DownloadFileAsync(
            FileItem file,
            IProgress<double>? progress = null,
            CancellationToken cancellationToken = default)
        {
            var name = SanitizeFileName(file.DisplayName);
            var target = await DownloadsFolder.CreateFileAsync(name, CreationCollisionOption.GenerateUniqueName);
            using var response = await AppServices.Downloads.OpenFileStreamAsync(file.FileId, cancellationToken);
            var total = response.Content.Headers.ContentLength ?? 0;
            var input = await response.Content.ReadAsStreamAsync();
            var output = await target.OpenAsync(FileAccessMode.ReadWrite);
            using var outputStream = output.AsStreamForWrite();

            var buffer = new byte[81920];
            long copied = 0;
            int read;
            while ((read = await input.ReadAsync(buffer, 0, buffer.Length, cancellationToken)) > 0)
            {
                await outputStream.WriteAsync(buffer, 0, read, cancellationToken);
                copied += read;
                if (total > 0) progress?.Report((double)copied / total);
            }

            await outputStream.FlushAsync(cancellationToken);
            progress?.Report(1);
            return target;
        }

        public static async Task<StorageFile> DownloadEbookAsync(
            EbookItem ebook,
            IProgress<double>? progress = null,
            CancellationToken cancellationToken = default)
        {
            var name = SanitizeFileName(ebook.DisplayName);
            if (!string.IsNullOrWhiteSpace(ebook.EbookFormat) && !name.EndsWith("." + ebook.EbookFormat, StringComparison.OrdinalIgnoreCase))
            {
                name += "." + ebook.EbookFormat;
            }

            var target = await DownloadsFolder.CreateFileAsync(name, CreationCollisionOption.GenerateUniqueName);
            using var response = await AppServices.Downloads.OpenEbookStreamAsync(ebook.EbookId, cancellationToken);
            var total = response.Content.Headers.ContentLength ?? 0;
            var input = await response.Content.ReadAsStreamAsync();
            var output = await target.OpenAsync(FileAccessMode.ReadWrite);
            using var outputStream = output.AsStreamForWrite();

            var buffer = new byte[81920];
            long copied = 0;
            int read;
            while ((read = await input.ReadAsync(buffer, 0, buffer.Length, cancellationToken)) > 0)
            {
                await outputStream.WriteAsync(buffer, 0, read, cancellationToken);
                copied += read;
                if (total > 0) progress?.Report((double)copied / total);
            }

            await outputStream.FlushAsync(cancellationToken);
            progress?.Report(1);
            return target;
        }

        public static async Task<InMemoryRandomAccessStream> ReadStreamAsync(
            Func<CancellationToken, Task<System.Net.Http.HttpResponseMessage>> open,
            CancellationToken cancellationToken = default)
        {
            using var response = await open(cancellationToken);
            var source = await response.Content.ReadAsStreamAsync();
            var memory = new InMemoryRandomAccessStream();
            using var output = memory.AsStreamForWrite();
            await source.CopyToAsync(output, 81920, cancellationToken);
            await output.FlushAsync(cancellationToken);
            memory.Seek(0);
            return memory;
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
