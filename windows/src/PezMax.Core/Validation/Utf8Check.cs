using System;
using System.IO;
using System.Text;

namespace PezMax.Core.Validation;

/// <summary>
/// 文本文件（`txt` / `md`）的 UTF-8 判定，逐条对齐 <c>spec/fixtures/upload/</c> 的 <c>kind: encoding</c> 用例。
/// 服务端要求 UTF-8（`datum_file.go:790-799`），本地先判一次能省一次往返，也能把错误落到具体那个文件上。
/// </summary>
public static class Utf8Check
{
    /// <summary>与上游 400 文案一致：客户端拦下时用同一句话，用户不会看到两种说法。</summary>
    public const string RejectMessage = "文本文件仅支持 UTF-8 编码";

    private const int BufferSize = 81920;

    /// <summary>判定一段字节是不是合法 UTF-8。空内容算合法（体积另算）。</summary>
    public static bool IsUtf8(byte[]? bytes)
    {
        if (bytes == null) return false;
        using var stream = new MemoryStream(bytes, false);
        return IsUtf8(stream);
    }

    /// <summary>
    /// 边读边判，不整份读进内存：200MB 的文本在 AppContainer 里整读会直接 OOM。
    /// <para>
    /// 三条容易写错的规则：<br/>
    /// 1. UTF-8 BOM（<c>EF BB BF</c>）**合法**，跳过再解码；<br/>
    /// 2. UTF-16 / UTF-32 的 BOM **不合法**——那不是 UTF-8；<br/>
    /// 3. 结尾要 flush：最后几个字节可能是被截断的多字节序列，只看「每块内部能否解码」会放过它。
    /// </para>
    /// </summary>
    public static bool IsUtf8(Stream stream)
    {
        if (stream == null) return false;

        var decoder = new UTF8Encoding(false, true).GetDecoder();
        var bytes = new byte[BufferSize];
        var chars = new char[BufferSize];

        try
        {
            var first = true;
            while (true)
            {
                var read = stream.Read(bytes, 0, bytes.Length);
                if (read <= 0) break;

                var offset = 0;
                if (first)
                {
                    first = false;
                    if (read >= 2 && IsWideBom(bytes[0], bytes[1])) return false;
                    if (read >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF) offset = 3;
                }

                decoder.Convert(bytes, offset, read - offset, chars, 0, chars.Length, false, out _, out _, out _);
            }

            decoder.Convert(Array.Empty<byte>(), 0, 0, chars, 0, chars.Length, true, out _, out _, out _);
            return true;
        }
        catch (DecoderFallbackException)
        {
            return false;
        }
    }

    /// <summary>非 UTF-8 的 BOM：UTF-16LE/BE（`FF FE` / `FE FF`）与 UTF-32（`00 00 FE FF`）。</summary>
    private static bool IsWideBom(byte first, byte second) =>
        (first == 0xFF && second == 0xFE) || (first == 0xFE && second == 0xFF);
}
