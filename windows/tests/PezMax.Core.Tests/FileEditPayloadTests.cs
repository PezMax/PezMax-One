using System.Net;
using System.Net.Http;
using System.Text;
using PezMax.Core.Api;
using PezMax.Core.Models;
using PezMax.Core.Net;
using Xunit;

namespace PezMax.Core.Tests;

/// <summary>
/// 编辑自己的资料：<c>PUT /datum/file</c> 是**整组覆盖**，请求少一个字段就等于把它清空
/// （`file_repository.go:59-76` 的 SET 列出的就是全部可写字段）。这两件事只在发请求那一刻才看得出来，
/// 所以用假 handler 抓 body 断言。
/// </summary>
public class FileEditPayloadTests
{
    private static readonly string[] WritableKeys =
    {
        "fileId", "fileName", "fileYear", "fileType", "fileSchool", "fileSubject", "remark",
    };

    [Fact]
    public void From_CopiesEveryWritableFieldIncludingRemark()
    {
        var item = new FileItem
        {
            FileId = 12,
            FileName = "期中试卷.pdf",
            FileYear = 2025,
            FileType = 2,
            FileSchool = "实验中学",
            FileSubject = "数学",
            Remark = "数学/实验中学/期末/2025",
        };

        var request = FileUpdateRequest.From(item);

        Assert.Equal(12, request.FileId);
        Assert.Equal("期中试卷.pdf", request.FileName);
        Assert.Equal(2025, request.FileYear);
        Assert.Equal(2, request.FileType);
        Assert.Equal("实验中学", request.FileSchool);
        Assert.Equal("数学", request.FileSubject);
        Assert.Equal("数学/实验中学/期末/2025", request.Remark);
    }

    [Fact]
    public async Task UpdatePayload_CarriesEveryField()
    {
        var handler = new CapturingHandler();
        using var http = new DatumHttp("http://example.invalid", handler);
        var api = new FilesApi(http);

        var item = new FileItem
        {
            FileId = 12,
            FileName = "期中试卷.pdf",
            FileYear = 2025,
            FileType = 2,
            FileSchool = "实验中学",
            FileSubject = "数学",
            Remark = "数学/实验中学",
        };

        await api.UpdateAsync(FileUpdateRequest.From(item));

        foreach (var key in WritableKeys) Assert.Contains("\"" + key + "\"", handler.LastBody);
        Assert.Contains("数学/实验中学", handler.LastBody);
    }

    [Fact]
    public void From_FallsBackToLabelWhenFileNameIsEmpty()
    {
        // 树叶子只有 label、没有 fileName：编辑时发空文件名，服务端回 400「文件名不能为空」。
        var item = new FileItem
        {
            FileId = 7,
            Label = "只有 label.pdf",
            FileSubject = "物理",
            FileType = 1,
            FileYear = 2024,
        };

        Assert.Equal("只有 label.pdf", FileUpdateRequest.From(item).FileName);
    }

    private sealed class CapturingHandler : HttpMessageHandler
    {
        public string LastBody { get; private set; } = string.Empty;

        protected override async Task<HttpResponseMessage> SendAsync(
            HttpRequestMessage request,
            CancellationToken cancellationToken)
        {
            LastBody = request.Content == null ? string.Empty : await request.Content.ReadAsStringAsync();
            return new HttpResponseMessage(HttpStatusCode.OK)
            {
                Content = new StringContent(
                    "{\"code\":200,\"msg\":\"ok\",\"data\":true}",
                    Encoding.UTF8,
                    "application/json"),
            };
        }
    }
}
