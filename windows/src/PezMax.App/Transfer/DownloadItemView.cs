using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using PezMax.Core.Models;
using PezMax.Core.Transfer;

namespace PezMax.Transfer
{
    /// <summary>
    /// 「我的下载」一行：把 Core 的 <see cref="DownloadRow"/>（服务端元信息 + 本地状态）翻成界面要显示的字符串。
    /// 合并规则本身在 Core（`spec/fixtures/download/`），这里只做展示映射。
    /// </summary>
    internal sealed class DownloadItemView
    {
        public long FileId { get; init; }
        public string Title { get; init; } = "";
        public string Meta { get; init; } = "";
        public string Status { get; init; } = "";
        public string LocalPath { get; init; } = "";
        public bool IsDone { get; init; }
        public bool IsPartial { get; init; }

        public string ActionText => IsDone ? "打开" : IsPartial ? "继续下载" : "下载";
        public bool CanOpenFile => IsDone && !string.IsNullOrEmpty(LocalPath) && File.Exists(LocalPath);
        public bool CanShowInFolder => !string.IsNullOrEmpty(LocalPath);

        public static DownloadItemView From(DownloadRow row)
        {
            var file = row.File;
            var meta = string.Join(" · ", new[]
            {
                file.TypeName,
                file.SubjectSchoolText,
                file.YearText,
                FileItem.FormatSize(file.FileSize),
                row.FirstDownloadTime,
            }.Where(part => !string.IsNullOrWhiteSpace(part) && part != "-"));

            return new DownloadItemView
            {
                FileId = row.FileId,
                Title = string.IsNullOrWhiteSpace(file.DisplayName) ? "未命名" : file.DisplayName,
                Meta = meta,
                Status = row.StatusText,
                LocalPath = row.Local?.LocalPath ?? "",
                IsDone = row.Local?.IsDone == true,
                IsPartial = row.IsPartial,
            };
        }

        public static IReadOnlyList<DownloadItemView> FromRows(IEnumerable<DownloadRow>? rows) =>
            rows == null
                ? Array.Empty<DownloadItemView>()
                : rows.Select(From).ToList();
    }
}
