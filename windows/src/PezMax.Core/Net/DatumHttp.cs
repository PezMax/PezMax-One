using System;
using System.Collections.Generic;
using System.IO;
using System.Net;
using System.Net.Http;
using System.Net.Http.Headers;
using System.Text;
using System.Threading;
using System.Threading.Tasks;

namespace PezMax.Core.Net;

/// <summary>
/// PezMax-Go 的 <c>/datum/*</c> 请求网关：Base URL、Bearer、信封解析、真实 HTTP 状态码错误、401。
/// </summary>
public sealed class DatumHttp : IDisposable
{
    public const string DefaultBaseUrl = "http://127.0.0.1:8080";

    private readonly HttpClient _client;
    private readonly bool _ownsClient;
    private volatile string? _token;

    public DatumHttp(string? baseUrl = null, HttpMessageHandler? handler = null)
    {
        BaseUrl = NormalizeBaseUrl(baseUrl ?? DefaultBaseUrl);
        _ownsClient = handler == null;
        _client = new HttpClient(handler ?? CreateHandler(), _ownsClient)
        {
            // 按请求设超时（下面的 RequestTimeout / UploadTimeout）：上传要跑几分钟，
            // 而 HttpClient.Timeout 是全局的，套在转档上会把慢文件直接判死。
            Timeout = System.Threading.Timeout.InfiniteTimeSpan,
        };
    }

    public string BaseUrl { get; set; }

    /// <summary>普通请求的超时。</summary>
    public TimeSpan RequestTimeout { get; set; } = TimeSpan.FromSeconds(30);

    /// <summary>
    /// 上传的超时。转档走 LibreOffice，默认 2 分钟超时 + 2 并发（`convert.go:34-47`），
    /// 大文件排队时会超过普通请求的 30 秒，所以单独放宽；**不要**在 30 秒后判失败。
    /// </summary>
    public TimeSpan UploadTimeout { get; set; } = TimeSpan.FromMinutes(15);

    public string? Token
    {
        get => _token;
        set => _token = string.IsNullOrWhiteSpace(value) ? null : value;
    }

    public bool IsAuthenticated => !string.IsNullOrEmpty(_token);

    /// <summary>收到 401 时触发；token 已在此之前清空。</summary>
    public event Action? SessionExpired;

    public static string NormalizeBaseUrl(string value)
    {
        var text = (value ?? "").Trim();
        if (text.Length == 0) return DefaultBaseUrl;
        return text.TrimEnd('/');
    }

    private static HttpClientHandler CreateHandler() => new()
    {
        UseCookies = false,
        AutomaticDecompression = DecompressionMethods.GZip | DecompressionMethods.Deflate,
    };

    public Task<DatumResponse> GetAsync(
        string path,
        IEnumerable<KeyValuePair<string, string?>>? query = null,
        CancellationToken cancellationToken = default) =>
        SendAsync(HttpMethod.Get, path, null, query, cancellationToken);

    public Task<DatumResponse> PostJsonAsync(
        string path,
        string jsonBody,
        CancellationToken cancellationToken = default) =>
        SendAsync(HttpMethod.Post, path, JsonContent(jsonBody), null, cancellationToken);

    public Task<DatumResponse> PutJsonAsync(
        string path,
        string jsonBody,
        CancellationToken cancellationToken = default) =>
        SendAsync(HttpMethod.Put, path, JsonContent(jsonBody), null, cancellationToken);

    public Task<DatumResponse> DeleteAsync(
        string path,
        CancellationToken cancellationToken = default) =>
        SendAsync(HttpMethod.Delete, path, null, null, cancellationToken);

    /// <summary>
    /// 文件流请求。成功时返回原始响应，调用方负责 Dispose；失败时按信封解析并抛 <see cref="DatumException"/>。
    /// </summary>
    public Task<HttpResponseMessage> GetStreamAsync(
        string path,
        IEnumerable<KeyValuePair<string, string?>>? query = null,
        CancellationToken cancellationToken = default) =>
        GetStreamAsync(path, query, null, cancellationToken);

    /// <summary>
    /// 带附加请求头的文件流请求（续传要 <c>Range</c> / <c>If-Range</c> / <c>Accept-Encoding: identity</c>）。
    /// 非 2xx 仍然抛 <see cref="DatumException"/>：**416 也走这里**，调用方按 <see cref="DatumException.HttpStatus"/>
    /// 交给 <c>DownloadPlanner</c> 判断要不要丢弃半成品重下。
    /// </summary>
    public async Task<HttpResponseMessage> GetStreamAsync(
        string path,
        IEnumerable<KeyValuePair<string, string?>>? query,
        IEnumerable<KeyValuePair<string, string>>? headers,
        CancellationToken cancellationToken = default)
    {
        using var request = CreateRequest(HttpMethod.Get, path, query, null, headers);
        using var timeout = WithTimeout(RequestTimeout, cancellationToken);

        HttpResponseMessage response;
        try
        {
            response = await _client
                .SendAsync(request, HttpCompletionOption.ResponseHeadersRead, timeout.Token)
                .ConfigureAwait(false);
        }
        catch (OperationCanceledException) when (!cancellationToken.IsCancellationRequested)
        {
            throw Timeout(path);
        }

        var status = (int)response.StatusCode;
        if (status < 200 || status >= 300)
        {
            var contentType = response.Content?.Headers.ContentType?.MediaType;
            var body = response.Content == null ? "" : await response.Content.ReadAsStringAsync().ConfigureAwait(false);
            response.Dispose();
            var parsed = DatumEnvelope.Parse(status, contentType, body);
            HandleFailure(parsed);
            throw parsed.ToException();
        }

        return response;
    }

    public async Task<DatumResponse> UploadAsync(
        string path,
        IEnumerable<KeyValuePair<string, string?>> fields,
        string fileName,
        Stream fileStream,
        string contentType,
        CancellationToken cancellationToken = default,
        IProgress<long>? bytesSent = null)
    {
        using var form = new MultipartFormDataContent();
        foreach (var field in fields)
        {
            if (field.Value == null) continue;
            form.Add(new StringContent(field.Value, Encoding.UTF8), field.Key);
        }

        var content = bytesSent == null ? fileStream : new ProgressStream(fileStream, bytesSent);
        var file = new StreamContent(content);
        file.Headers.ContentType = MediaTypeHeaderValue.Parse(contentType);
        form.Add(file, "file", fileName);

        using var request = new HttpRequestMessage(HttpMethod.Post, BuildUri(path, null)) { Content = form };
        ApplyHeaders(request);
        using var timeout = WithTimeout(UploadTimeout, cancellationToken);
        try
        {
            using var response = await _client.SendAsync(request, timeout.Token).ConfigureAwait(false);
            return await ReadResponseAsync(response).ConfigureAwait(false);
        }
        catch (OperationCanceledException) when (!cancellationToken.IsCancellationRequested)
        {
            throw Timeout(path);
        }
    }

    private async Task<DatumResponse> SendAsync(
        HttpMethod method,
        string path,
        HttpContent? content,
        IEnumerable<KeyValuePair<string, string?>>? query,
        CancellationToken cancellationToken)
    {
        using var request = CreateRequest(method, path, query, content);
        using var timeout = WithTimeout(RequestTimeout, cancellationToken);
        try
        {
            using var response = await _client.SendAsync(request, timeout.Token).ConfigureAwait(false);
            return await ReadResponseAsync(response).ConfigureAwait(false);
        }
        catch (OperationCanceledException) when (!cancellationToken.IsCancellationRequested)
        {
            throw Timeout(path);
        }
    }

    /// <summary>
    /// 按请求限时：<see cref="HttpClient.Timeout"/> 是全局的，而上传与普通请求要的时限差一个数量级。
    /// 调用方自己的取消（用户点「取消」）原样透传，不被误判成超时。
    /// </summary>
    private static CancellationTokenSource WithTimeout(TimeSpan timeout, CancellationToken cancellationToken)
    {
        var source = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        if (timeout > TimeSpan.Zero && timeout != System.Threading.Timeout.InfiniteTimeSpan)
        {
            source.CancelAfter(timeout);
        }

        return source;
    }

    private static TimeoutException Timeout(string path) => new("请求超时：" + path);

    private HttpRequestMessage CreateRequest(
        HttpMethod method,
        string path,
        IEnumerable<KeyValuePair<string, string?>>? query,
        HttpContent? content,
        IEnumerable<KeyValuePair<string, string>>? headers = null)
    {
        var request = new HttpRequestMessage(method, BuildUri(path, query)) { Content = content };
        ApplyHeaders(request);
        if (headers != null)
        {
            foreach (var header in headers) request.Headers.TryAddWithoutValidation(header.Key, header.Value);
        }
        return request;
    }

    private void ApplyHeaders(HttpRequestMessage request)
    {
        request.Headers.Accept.ParseAdd("application/json");
        var token = _token;
        if (!string.IsNullOrEmpty(token))
        {
            request.Headers.Authorization = new AuthenticationHeaderValue("Bearer", token);
        }
    }

    private Uri BuildUri(string path, IEnumerable<KeyValuePair<string, string?>>? query)
    {
        var normalized = string.IsNullOrEmpty(path) ? "/" : path;
        if (!normalized.StartsWith("/", StringComparison.Ordinal)) normalized = "/" + normalized;

        var sb = new StringBuilder(BaseUrl).Append(normalized);
        var first = true;
        if (query != null)
        {
            foreach (var pair in query)
            {
                if (string.IsNullOrEmpty(pair.Value)) continue;
                sb.Append(first ? '?' : '&');
                first = false;
                sb.Append(Uri.EscapeDataString(pair.Key)).Append('=').Append(Uri.EscapeDataString(pair.Value));
            }
        }
        return new Uri(sb.ToString(), UriKind.Absolute);
    }

    private async Task<DatumResponse> ReadResponseAsync(HttpResponseMessage response)
    {
        var status = (int)response.StatusCode;
        var contentType = response.Content?.Headers.ContentType?.MediaType;
        var body = response.Content == null
            ? ""
            : await response.Content.ReadAsStringAsync().ConfigureAwait(false);

        var parsed = DatumEnvelope.Parse(status, contentType, body);
        HandleFailure(parsed);
        return parsed;
    }

    private void HandleFailure(DatumResponse parsed)
    {
        if (!parsed.IsSessionExpired) return;
        _token = null;
        SessionExpired?.Invoke();
    }

    private static HttpContent JsonContent(string jsonBody) =>
        new StringContent(jsonBody, Encoding.UTF8, "application/json");

    public void Dispose()
    {
        if (_ownsClient) _client.Dispose();
    }
}
