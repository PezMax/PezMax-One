using System;
using System.Collections.Generic;
using System.Linq;
using PezMax.Core.Json;

namespace PezMax.Core.Models;

public sealed class FileItem
{
    public long Id { get; init; }
    public long FileId { get; init; }
    public string Label { get; init; } = "";
    public string FileName { get; init; } = "";
    public string FileUrl { get; init; } = "";
    public long FileSize { get; init; }
    public string FileFormat { get; init; } = "";
    public long FileYear { get; init; }
    public long FileType { get; init; }
    public string FileSchool { get; init; } = "";
    public string FileSubject { get; init; } = "";
    public long FileStatus { get; init; }
    public string Reviewer { get; init; } = "";
    public long DelFlag { get; init; }
    public long UserId { get; init; }
    public string CreateBy { get; init; } = "";
    public string CreateTime { get; init; } = "";
    public string UpdateBy { get; init; } = "";
    public string UpdateTime { get; init; } = "";
    public string Remark { get; init; } = "";

    public string DisplayName => string.IsNullOrWhiteSpace(FileName) ? Label : FileName;
    public bool IsPreviewable => FileFormat is "pdf" or "jpg" or "jpeg" or "png" or "webp" or "txt" or "md";
    public string SizeText => FormatSize(FileSize);
    public string YearText => FileYear > 0 ? FileYear.ToString() : "-";
    public string TypeName => FileTypes.Name(FileType);
    public string SubjectSchoolText => string.Join(" · ", new[] { FileSubject, FileSchool }.Where(s => !string.IsNullOrWhiteSpace(s)));
    public string StatusText => FileStatus switch
    {
        0 => "待审核",
        1 => "已通过",
        2 => "未通过",
        3 => "已下架",
        _ => "",
    };

    public static string FormatSize(long bytes)
    {
        if (bytes <= 0) return "-";
        string[] units = { "B", "KB", "MB", "GB" };
        double value = bytes;
        var unit = 0;
        while (value >= 1024 && unit < units.Length - 1)
        {
            value /= 1024;
            unit++;
        }
        return $"{value:0.#} {units[unit]}";
    }

    public static FileItem From(JsonValue? value) => new()
    {
        Id = JsonModel.Long(value, "id"),
        FileId = JsonModel.Long(value, "fileId"),
        Label = JsonModel.Str(value, "label"),
        FileName = JsonModel.Str(value, "fileName"),
        FileUrl = JsonModel.Str(value, "fileUrl"),
        FileSize = JsonModel.Long(value, "fileSize"),
        FileFormat = JsonModel.Str(value, "fileFormat"),
        FileYear = JsonModel.Long(value, "fileYear"),
        FileType = JsonModel.Long(value, "fileType"),
        FileSchool = JsonModel.Str(value, "fileSchool"),
        FileSubject = JsonModel.Str(value, "fileSubject"),
        FileStatus = JsonModel.Long(value, "fileStatus"),
        Reviewer = JsonModel.Str(value, "reviewer"),
        DelFlag = JsonModel.Long(value, "delFlag"),
        UserId = JsonModel.Long(value, "userId"),
        CreateBy = JsonModel.Str(value, "createBy"),
        CreateTime = JsonModel.Str(value, "createTime"),
        UpdateBy = JsonModel.Str(value, "updateBy"),
        UpdateTime = JsonModel.Str(value, "updateTime"),
        Remark = JsonModel.Str(value, "remark"),
    };
}

public sealed class FileTreeNode
{
    public string Id { get; init; } = "";
    public string Label { get; init; } = "";
    public string Type { get; init; } = "";
    public long FileId { get; init; }
    public string FileUrl { get; init; } = "";
    public string FileFormat { get; init; } = "";
    public IReadOnlyList<FileTreeNode> Children { get; init; } = Array.Empty<FileTreeNode>();

    public bool IsFolder => string.Equals(Type, "folder", StringComparison.OrdinalIgnoreCase);
    public FileItem? File { get; init; }

    public static FileTreeNode From(JsonValue value)
    {
        var children = new List<FileTreeNode>();
        if (value.GetArray("children") is { } nodes)
        {
            foreach (var node in nodes)
            {
                if (node.IsObject) children.Add(From(node));
            }
        }

        var type = JsonModel.Str(value, "type");
        var isFile = string.Equals(type, "file", StringComparison.OrdinalIgnoreCase);
        return new FileTreeNode
        {
            Id = JsonModel.Str(value, "id"),
            Label = JsonModel.Str(value, "label"),
            Type = type,
            FileId = JsonModel.Long(value, "fileId"),
            FileUrl = JsonModel.Str(value, "fileUrl"),
            FileFormat = JsonModel.Str(value, "fileFormat"),
            Children = children,
            File = isFile ? FileItem.From(value) : null,
        };
    }

    public static IReadOnlyList<FileTreeNode> ListFrom(IReadOnlyList<JsonValue>? nodes)
    {
        var result = new List<FileTreeNode>();
        if (nodes == null) return result;
        foreach (var node in nodes)
        {
            if (node.IsObject) result.Add(From(node));
        }
        return result;
    }
}

public static class FileTypes
{
    public static string Name(long type) => type switch
    {
        1 => "期末",
        2 => "期中",
        3 => "补考",
        4 => "资料",
        5 => "其他学校",
        6 => "神秘文件",
        7 => "电子书",
        _ => "其他",
    };

    public static long IdFromName(string? name) => name switch
    {
        "期末" => 1,
        "期中" => 2,
        "补考" => 3,
        "资料" => 4,
        "其他学校" => 5,
        "神秘文件" => 6,
        "电子书" => 7,
        _ => 0,
    };

    public static IReadOnlyList<string> Names { get; } = new[] { "期末", "期中", "补考", "资料", "其他学校", "神秘文件", "电子书" };
}

public sealed class DownloadEntry
{
    public FileItem File { get; init; } = new();
    public string FirstDownloadTime { get; init; } = "";
    public string DisplayName => File.DisplayName;
    public string MetaText => $"{File.TypeName} · {File.SubjectSchoolText} · {FirstDownloadTime}";

    public static DownloadEntry From(JsonValue? value) => new()
    {
        File = FileItem.From(value),
        FirstDownloadTime = JsonModel.Str(value, "firstDownloadTime"),
    };
}
