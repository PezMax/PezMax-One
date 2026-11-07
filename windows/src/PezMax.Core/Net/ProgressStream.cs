using System;
using System.IO;
using System.Threading;
using System.Threading.Tasks;

namespace PezMax.Core.Net;

/// <summary>
/// 只读包装流：读出去多少字节就报一次累计字节数，用来做上传进度。
/// <para>
/// 为什么不用 <c>IProgress&lt;HttpProgress&gt;</c>：那是 <c>Windows.Web.Http</c> 的接口，
/// 用它就得让 App 自己拼 multipart、自己带 Bearer——协议与鉴权就飘出 Core 了。
/// <c>StreamContent</c> 是在 <c>SendAsync</c> 内部边读边发的，所以包一层流是唯一不动架构的做法。
/// </para>
/// </summary>
public sealed class ProgressStream : Stream
{
    private readonly Stream _inner;
    private readonly IProgress<long>? _progress;
    private long _sent;

    public ProgressStream(Stream inner, IProgress<long>? progress = null)
    {
        _inner = inner ?? throw new ArgumentNullException(nameof(inner));
        _progress = progress;
    }

    /// <summary>已经读出去（也就是发出）的字节数。</summary>
    public long BytesSent => _sent;

    public override bool CanRead => true;

    public override bool CanSeek => false;

    public override bool CanWrite => false;

    public override long Length => _inner.Length;

    public override long Position
    {
        get => _inner.Position;
        set => throw new NotSupportedException();
    }

    public override void Flush()
    {
    }

    public override long Seek(long offset, SeekOrigin origin) => throw new NotSupportedException();

    public override void SetLength(long value) => throw new NotSupportedException();

    public override void Write(byte[] buffer, int offset, int count) => throw new NotSupportedException();

    public override int Read(byte[] buffer, int offset, int count)
    {
        var read = _inner.Read(buffer, offset, count);
        Report(read);
        return read;
    }

    public override async Task<int> ReadAsync(byte[] buffer, int offset, int count, CancellationToken cancellationToken)
    {
        var read = await _inner.ReadAsync(buffer, offset, count, cancellationToken).ConfigureAwait(false);
        Report(read);
        return read;
    }

    /// <summary>不接管内层流：调用方（上传服务）负责关文件句柄。</summary>
    protected override void Dispose(bool disposing)
    {
    }

    private void Report(int read)
    {
        if (read <= 0) return;
        _sent += read;
        _progress?.Report(_sent);
    }
}
