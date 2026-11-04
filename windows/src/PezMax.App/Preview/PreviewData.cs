using System;
using System.Collections.Generic;
using PezMax.Core.Models;

namespace PezMax;

internal static class PreviewData
{
    public static IReadOnlyList<FileItem> Files { get; } = new[]
    {
        MakeFile(1, "高等数学期末试卷.pdf", "数学", "实验中学", 2025, 1, "pdf", 2457600, 1),
        MakeFile(2, "大学物理期中试题.pdf", "物理", "第一中学", 2025, 2, "pdf", 1835008, 1),
        MakeFile(3, "英语四级真题合集.docx", "英语", "外国语学校", 2024, 4, "docx", 524288, 1),
        MakeFile(4, "数据结构复习提纲.md", "计算机", "实验中学", 2025, 4, "md", 20480, 1),
        MakeFile(5, "化学实验报告模板.txt", "化学", "第二中学", 2024, 6, "txt", 8192, 0),
        MakeFile(6, "线性代数模拟卷.pdf", "数学", "实验中学", 2024, 2, "pdf", 3145728, 1),
    };

    public static IReadOnlyList<FileTreeNode> Tree { get; } = new[]
    {
        Folder("数学", new[]
        {
            Folder("实验中学", new[]
            {
                Folder("期末", new[]
                {
                    Folder("2025", new[]
                    {
                        Leaf(Files[0]),
                    }),
                }),
                Folder("期中", new[]
                {
                    Folder("2024", new[] { Leaf(Files[5]) }),
                }),
            }),
        }),
        Folder("物理", new[]
        {
            Folder("第一中学", new[]
            {
                Folder("期中", new[] { Folder("2025", new[] { Leaf(Files[1]) }) }),
            }),
        }),
        Folder("英语", new[]
        {
            Folder("外国语学校", new[]
            {
                Folder("资料", new[] { Folder("2024", new[] { Leaf(Files[2]) }) }),
            }),
        }),
    };

    public static IReadOnlyList<RankEntry> Rank { get; } = new[]
    {
        new RankEntry { UserId = 1, UserName = "林同学", Count = 128, Uploads = 128 },
        new RankEntry { UserId = 2, UserName = "陈同学", Count = 96, Uploads = 96 },
        new RankEntry { UserId = 3, UserName = "周同学", Count = 74, Uploads = 74 },
        new RankEntry { UserId = 4, UserName = "王同学", Count = 51, Uploads = 51 },
        new RankEntry { UserId = 5, UserName = "李同学", Count = 33, Uploads = 33 },
    };

    private static FileItem MakeFile(
        long id, string name, string subject, string school, long year, long type, string format, long size, long status) => new()
    {
        Id = id,
        FileId = id,
        Label = name,
        FileName = name,
        FileUrl = "https://example.com/" + Uri.EscapeDataString(name),
        FileSize = size,
        FileFormat = format,
        FileYear = year,
        FileType = type,
        FileSchool = school,
        FileSubject = subject,
        FileStatus = status,
        UserId = 7,
        CreateTime = "2026-11-01 10:00:00",
    };

    private static FileTreeNode Folder(string label, IReadOnlyList<FileTreeNode> children) => new()
    {
        Id = "folder-" + label,
        Label = label,
        Type = "folder",
        Children = children,
    };

    private static FileTreeNode Leaf(FileItem file) => new()
    {
        Id = "file-" + file.FileId,
        Label = file.DisplayName,
        Type = "file",
        FileId = file.FileId,
        FileUrl = file.FileUrl,
        FileFormat = file.FileFormat,
        Children = Array.Empty<FileTreeNode>(),
        File = file,
    };
}
