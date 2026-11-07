using System.Net;
using System.Net.Http;
using System.Text;
using PezMax.Core.Net;
using Xunit;

namespace PezMax.Core.Tests;

/// <summary>
/// 上传的管道：进度包装流、以及「超时按请求设」这条改动。
/// 判定表类的行为在夹具里，这里测的是 I/O 管子的性质，用假 handler 跑，不碰网络。
/// </summary>
public class UploadPipelineTests
{
    [Fact]
    public void ProgressStream_ReportsCumulativeBytesAndPassesDataThrough()
    {
        var payload = new byte[4096];
        for (var i = 0; i < payload.Length; i++) payload[i] = (byte)(i % 251);

        var progress = new ListProgress();
        using var inner = new MemoryStream(payload);
        using var stream = new ProgressStream(inner, progress);

        var copy = new byte[payload.Length];
        var total = 0;
        while (total < copy.Length)
        {
            var read = stream.Read(copy, total, Math.Min(512, copy.Length - total));
            if (read <= 0) break;
            total += read;
        }

        Assert.Equal(payload.Length, total);
        Assert.Equal(payload, copy);
        Assert.Equal(payload.Length, stream.BytesSent);

        // 累计、单调、最后一条正好是全量。
        Assert.Equal(8, progress.Values.Count);
        Assert.True(progress.Values.SequenceEqual(progress.Values.OrderBy(v => v)));
        Assert.Equal(payload.Length, progress.Values[^1]);
    }

    [Fact]
    public void ProgressStream_DoesNotOwnInnerStream()
    {
        var inner = new MemoryStream();
        inner.Write(new byte[16], 0, 16);

        using (var stream = new ProgressStream(inner))
        {
            Assert.Equal(16, stream.Length);
        }

        // 内层还活着：文件句柄由上传服务自己关。
        inner.WriteByte(1);
        Assert.Equal(17, inner.Length);
        inner.Dispose();
    }

    [Fact]
    public async Task UploadAsync_ReportsProgressWhileSending()
    {
        var handler = new EchoHandler();
        using var http = new DatumHttp("http://example.invalid", handler);
        var payload = new byte[8192];
        var progress = new ListProgress();
        using var source = new MemoryStream(payload);

        var response = await http.UploadAsync(
            "/datum/file",
            new[] { new KeyValuePair<string, string?>("fileSubject", "数学") },
            "试卷.pdf",
            source,
            "application/pdf",
            default,
            progress);

        Assert.True(response.IsSuccess, response.Message);
        Assert.Equal(payload.Length, progress.Values[^1]);
        Assert.True(handler.BytesReceived > payload.Length, "multipart 的分隔符与表单项也算字节");
    }

    [Fact]
    public async Task Request_TimesOutWithTimeoutException()
    {
        using var http = new DatumHttp("http://example.invalid", new EchoHandler(TimeSpan.FromSeconds(5)))
        {
            RequestTimeout = TimeSpan.FromMilliseconds(50),
        };

        await Assert.ThrowsAsync<TimeoutException>(() => http.GetAsync("/datum/health"));
    }

    [Fact]
    public async Task Request_KeepsCallerCancellationDistinctFromTimeout()
    {
        using var http = new DatumHttp("http://example.invalid", new EchoHandler(TimeSpan.FromSeconds(5)));
        using var cancel = new CancellationTokenSource(TimeSpan.FromMilliseconds(50));

        // 用户点「取消」不能被说成「超时」——提示差一个字，处理方式差一个数量级。
        var error = await Assert.ThrowsAnyAsync<OperationCanceledException>(
            () => http.GetAsync("/datum/health", null, cancel.Token));
        Assert.IsNotType<TimeoutException>(error);
    }

    private sealed class ListProgress : IProgress<long>
    {
        public List<long> Values { get; } = new();

        public void Report(long value) => Values.Add(value);
    }

    private sealed class EchoHandler : HttpMessageHandler
    {
        private readonly TimeSpan _delay;

        public EchoHandler(TimeSpan? delay = null) => _delay = delay ?? TimeSpan.Zero;

        public long BytesReceived { get; private set; }

        protected override async Task<HttpResponseMessage> SendAsync(
            HttpRequestMessage request,
            CancellationToken cancellationToken)
        {
            if (_delay > TimeSpan.Zero) await Task.Delay(_delay, cancellationToken);

            if (request.Content != null)
            {
                var body = await request.Content.ReadAsByteArrayAsync();
                BytesReceived = body.Length;
            }

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
