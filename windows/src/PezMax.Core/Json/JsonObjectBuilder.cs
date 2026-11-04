using System.Collections.Generic;
using System.Globalization;
using System.Text;

namespace PezMax.Core.Json;

/// <summary>
/// 手写 JSON 对象写入器：只支持扁平的「键 → 标量 / 预编码片段」。
/// 请求体都是这种形状；不引入序列化库，避免 .NET Native 反射裁剪问题。
/// </summary>
public sealed class JsonObjectBuilder
{
    private readonly List<KeyValuePair<string, string>> _fields = new();

    public JsonObjectBuilder String(string name, string? value)
    {
        _fields.Add(new KeyValuePair<string, string>(name, value == null ? "null" : JsonText.Escape(value)));
        return this;
    }

    public JsonObjectBuilder Number(string name, long value)
    {
        _fields.Add(new KeyValuePair<string, string>(name, value.ToString(CultureInfo.InvariantCulture)));
        return this;
    }

    public JsonObjectBuilder Bool(string name, bool value)
    {
        _fields.Add(new KeyValuePair<string, string>(name, value ? "true" : "false"));
        return this;
    }

    public JsonObjectBuilder Null(string name)
    {
        _fields.Add(new KeyValuePair<string, string>(name, "null"));
        return this;
    }

    /// <summary>插入已经编码好的 JSON 片段（对象 / 数组 / 数字）。调用方负责合法性。</summary>
    public JsonObjectBuilder Raw(string name, string json)
    {
        _fields.Add(new KeyValuePair<string, string>(name, json));
        return this;
    }

    public override string ToString()
    {
        var sb = new StringBuilder();
        sb.Append('{');
        for (var i = 0; i < _fields.Count; i++)
        {
            if (i > 0) sb.Append(',');
            JsonText.AppendQuoted(sb, _fields[i].Key);
            sb.Append(':').Append(_fields[i].Value);
        }
        sb.Append('}');
        return sb.ToString();
    }
}

public static class JsonText
{
    public static string Escape(string value)
    {
        var sb = new StringBuilder(value.Length + 2);
        AppendQuoted(sb, value);
        return sb.ToString();
    }

    internal static void AppendQuoted(StringBuilder sb, string value)
    {
        sb.Append('"');
        foreach (var c in value)
        {
            switch (c)
            {
                case '"': sb.Append("\\\""); break;
                case '\\': sb.Append("\\\\"); break;
                case '\b': sb.Append("\\b"); break;
                case '\f': sb.Append("\\f"); break;
                case '\n': sb.Append("\\n"); break;
                case '\r': sb.Append("\\r"); break;
                case '\t': sb.Append("\\t"); break;
                default:
                    if (c < ' ') sb.Append("\\u").Append(((int)c).ToString("x4", CultureInfo.InvariantCulture));
                    else sb.Append(c);
                    break;
            }
        }
        sb.Append('"');
    }
}
