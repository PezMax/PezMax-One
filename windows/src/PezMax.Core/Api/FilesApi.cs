using System;
using System.Collections.Generic;
using System.IO;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using PezMax.Core.Json;
using PezMax.Core.Models;
using PezMax.Core.Net;

namespace PezMax.Core.Api;

public sealed class FileTreeResult
{
    public FileTreeResult(IReadOnlyList<FileTreeNode> nodes, string? hash, bool unchanged)
    {
        Nodes = nodes;
        Hash = hash;
        Unchanged = unchanged;
    }

    public IReadOnlyList<FileTreeNode> Nodes { get; }
    public string? Hash { get; }
    public bool Unchanged { get; }
}

public sealed class FileUploadResult
{
    public long FileId { get; init; }
    public string FileUrl { get; init; } = "";
    public string FileName { get; init; } = "";

    public static FileUploadResult From(JsonValue? value) => new()
    {
        FileId = JsonModel.Long(value, "fileId"),
        FileUrl = JsonModel.Str(value, "fileUrl"),
        FileName = JsonModel.Str(value, "fileName"),
    };
}

public sealed class FileUpdateRequest
{
    public long FileId { get; set; }
    public string FileName { get; set; } = "";
    public long FileYear { get; set; }
    public long FileType { get; set; }
    public string FileSchool { get; set; } = "";
    public string FileSubject { get; set; } = "";
    public string Remark { get; set; } = "";

    /// <summary>
    /// 从当前行回填。<c>PUT /datum/file</c> 是**整组覆盖**：少发一个字段就等于把它清空，
    /// 尤其 <c>remark</c>——那是树里的自定义目录，不带就把目录抹了。
    /// </summary>
    public static FileUpdateRequest From(FileItem item) => new()
    {
        FileId = item.FileId,
        FileName = item.DisplayName,
        FileYear = item.FileYear,
        FileType = item.FileType,
        FileSchool = item.FileSchool,
        FileSubject = item.FileSubject,
        Remark = item.Remark,
    };
}

public sealed class FileQuery
{
    public int PageNum { get; set; } = 1;
    public int PageSize { get; set; } = 20;
    public long? FileType { get; set; }
    public string? FileSubject { get; set; }
    public string? FileSchool { get; set; }
    public long? FileYear { get; set; }
    public long? YearFrom { get; set; }
    public long? YearTo { get; set; }
    public string? Keyword { get; set; }
    public long? UserId { get; set; }
}

public sealed class FilesApi
{
    private readonly DatumHttp _http;

    public FilesApi(DatumHttp http) => _http = http;

    public async Task<FileTreeResult> GetTreeAsync(
        string? hash = null,
        CancellationToken cancellationToken = default)
    {
        var query = hash == null
            ? null
            : new[] { new KeyValuePair<string, string?>("hash", hash) };
        var response = await _http.GetAsync("/datum/file/tree", query, cancellationToken).ConfigureAwait(false);
        ApiHelpers.EnsureSuccess(response);
        return new FileTreeResult(
            FileTreeNode.ListFrom(response.Data?.IsArray == true ? response.Data.Items : null),
            response.Hash,
            response.Unchanged);
    }

    public async Task<PageResult<FileItem>> ListAsync(FileQuery query, CancellationToken cancellationToken = default)
    {
        var response = await _http.GetAsync("/datum/file/list", BuildQuery(query), cancellationToken).ConfigureAwait(false);
        return ApiHelpers.EnsurePage(response, FileItem.From);
    }

    public async Task<IReadOnlyList<FileItem>> SearchAsync(string keyword, CancellationToken cancellationToken = default)
    {
        var query = new[] { new KeyValuePair<string, string?>("keyword", keyword) };
        var response = ApiHelpers.EnsureSuccess(
            await _http.GetAsync("/datum/file/search", query, cancellationToken).ConfigureAwait(false));
        return ApiHelpers.MapArray(response.Data, FileItem.From);
    }

    public async Task<IReadOnlyList<SubjectCount>> GetSubjectsAsync(CancellationToken cancellationToken = default)
    {
        var response = ApiHelpers.EnsureSuccess(
            await _http.GetAsync("/datum/file/subjects", cancellationToken: cancellationToken).ConfigureAwait(false));
        return ApiHelpers.MapArray(response.Data, SubjectCount.From);
    }

    public async Task<IReadOnlyList<string>> GetSchoolsAsync(
        string? keyword = null,
        CancellationToken cancellationToken = default)
    {
        var query = keyword == null
            ? null
            : new[] { new KeyValuePair<string, string?>("keyword", keyword) };
        var response = ApiHelpers.EnsureSuccess(
            await _http.GetAsync("/datum/file/schools", query, cancellationToken).ConfigureAwait(false));
        var result = new List<string>();
        if (response.Data?.IsArray == true)
        {
            foreach (var item in response.Data.Items)
            {
                if (item.StringValue is { } text && !string.IsNullOrWhiteSpace(text)) result.Add(text);
            }
        }
        return result;
    }

    public async Task<bool> CheckSchoolAsync(string schoolName, CancellationToken cancellationToken = default)
    {
        var query = new[] { new KeyValuePair<string, string?>("schoolName", schoolName) };
        var response = ApiHelpers.EnsureSuccess(
            await _http.GetAsync("/datum/file/schools/check", query, cancellationToken).ConfigureAwait(false));
        return SchoolCheck.From(response.Data).Exists;
    }

    public async Task<FileItem> GetDetailAsync(long fileId, CancellationToken cancellationToken = default) =>
        ApiHelpers.EnsureData(
            await _http.GetAsync($"/datum/file/{fileId}", cancellationToken: cancellationToken).ConfigureAwait(false),
            FileItem.From);

    public async Task<string> GetTextContentAsync(long fileId, CancellationToken cancellationToken = default)
    {
        var query = new[] { new KeyValuePair<string, string?>("fileId", fileId.ToString()) };
        using var response = await _http.GetStreamAsync("/datum/file/content", query, cancellationToken).ConfigureAwait(false);
        using var reader = new StreamReader(await response.Content.ReadAsStreamAsync().ConfigureAwait(false), Encoding.UTF8);
        return await reader.ReadToEndAsync().ConfigureAwait(false);
    }

    public Task<FileUploadResult> UploadAsync(
        string fileName,
        Stream stream,
        string contentType,
        string fileSubject,
        string fileSchool,
        long fileYear,
        long fileType,
        string? folderPath = null,
        CancellationToken cancellationToken = default,
        IProgress<long>? bytesSent = null)
    {
        var fields = new List<KeyValuePair<string, string?>>
        {
            new("fileName", fileName),
            new("fileSubject", fileSubject),
            new("fileSchool", fileSchool),
            new("fileYear", fileYear.ToString()),
            new("fileType", fileType.ToString()),
            new("remark", folderPath ?? ""),
        };
        return UploadCoreAsync(fileName, stream, contentType, fields, cancellationToken, bytesSent);
    }

    private async Task<FileUploadResult> UploadCoreAsync(
        string fileName,
        Stream stream,
        string contentType,
        IEnumerable<KeyValuePair<string, string?>> fields,
        CancellationToken cancellationToken,
        IProgress<long>? bytesSent = null)
    {
        var response = await _http.UploadAsync(
            "/datum/file", fields, fileName, stream, contentType, cancellationToken, bytesSent).ConfigureAwait(false);
        return ApiHelpers.EnsureData(response, FileUploadResult.From);
    }

    public async Task UpdateAsync(FileUpdateRequest request, CancellationToken cancellationToken = default)
    {
        var body = new JsonObjectBuilder()
            .Number("fileId", request.FileId)
            .String("fileName", request.FileName)
            .Number("fileYear", request.FileYear)
            .Number("fileType", request.FileType)
            .String("fileSchool", request.FileSchool)
            .String("fileSubject", request.FileSubject)
            .String("remark", request.Remark)
            .ToString();
        ApiHelpers.EnsureTrue(
            await _http.PutJsonAsync("/datum/file", body, cancellationToken).ConfigureAwait(false));
    }

    public async Task DeleteAsync(long fileId, CancellationToken cancellationToken = default) =>
        ApiHelpers.EnsureTrue(
            await _http.DeleteAsync($"/datum/file/{fileId}", cancellationToken).ConfigureAwait(false));

    private static IEnumerable<KeyValuePair<string, string?>> BuildQuery(FileQuery query)
    {
        yield return new KeyValuePair<string, string?>("pageNum", query.PageNum.ToString());
        yield return new KeyValuePair<string, string?>("pageSize", query.PageSize.ToString());
        yield return new KeyValuePair<string, string?>("fileType", query.FileType?.ToString());
        yield return new KeyValuePair<string, string?>("fileSubject", query.FileSubject);
        yield return new KeyValuePair<string, string?>("fileSchool", query.FileSchool);
        yield return new KeyValuePair<string, string?>("fileYear", query.FileYear?.ToString());
        yield return new KeyValuePair<string, string?>("yearFrom", query.YearFrom?.ToString());
        yield return new KeyValuePair<string, string?>("yearTo", query.YearTo?.ToString());
        yield return new KeyValuePair<string, string?>("keyword", query.Keyword);
        yield return new KeyValuePair<string, string?>("userId", query.UserId?.ToString());
    }
}
