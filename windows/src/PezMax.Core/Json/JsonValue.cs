using System;
using System.Collections.Generic;
using System.Globalization;
using System.Text;

namespace PezMax.Core.Json;

public enum JsonKind
{
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
}

public sealed class JsonParseException : Exception
{
    public JsonParseException(string message, int position)
        : base($"{message}（位置 {position}）")
    {
        Position = position;
    }

    public int Position { get; }
}

/// <summary>
/// 只读 JSON DOM。零反射、零外部依赖，供 netstandard2.0 / .NET Native 使用。
/// 写入见 <see cref="JsonObjectBuilder"/>；这里只负责读。
/// </summary>
public sealed class JsonValue
{
    private readonly JsonKind _kind;
    private readonly string? _text;
    private readonly bool _bool;
    private readonly List<JsonValue>? _items;
    private readonly List<KeyValuePair<string, JsonValue>>? _members;
    private readonly Dictionary<string, JsonValue>? _index;

    private JsonValue(JsonKind kind) => _kind = kind;

    private JsonValue(string text) { _kind = JsonKind.String; _text = text; }

    private JsonValue(JsonKind kind, string text) { _kind = kind; _text = text; }

    private JsonValue(bool value) { _kind = JsonKind.Bool; _bool = value; }

    private JsonValue(List<JsonValue> items) { _kind = JsonKind.Array; _items = items; }

    private JsonValue(List<KeyValuePair<string, JsonValue>> members)
    {
        _kind = JsonKind.Object;
        _members = members;
        _index = new Dictionary<string, JsonValue>(members.Count, StringComparer.Ordinal);
        foreach (var member in members)
        {
            _index[member.Key] = member.Value;
        }
    }

    public static JsonValue Null { get; } = new(JsonKind.Null);

    public JsonKind Kind => _kind;
    public bool IsNull => _kind == JsonKind.Null;
    public bool IsObject => _kind == JsonKind.Object;
    public bool IsArray => _kind == JsonKind.Array;
    public bool IsString => _kind == JsonKind.String;
    public bool IsNumber => _kind == JsonKind.Number;
    public bool IsBool => _kind == JsonKind.Bool;

    public int Count => _kind == JsonKind.Array ? _items!.Count : _kind == JsonKind.Object ? _members!.Count : 0;

    public IReadOnlyList<JsonValue> Items =>
        _items ?? throw new InvalidOperationException($"不是数组（实际是 {_kind}）");

    public IReadOnlyList<KeyValuePair<string, JsonValue>> Members =>
        _members ?? throw new InvalidOperationException($"不是对象（实际是 {_kind}）");

    public string? StringValue => _kind == JsonKind.String ? _text : null;
    public bool BoolValue => _kind == JsonKind.Bool && _bool;

    public long LongValue
    {
        get
        {
            if (_kind != JsonKind.Number || _text == null) return 0;
            return long.TryParse(_text, NumberStyles.Integer, CultureInfo.InvariantCulture, out var value)
                ? value
                : (long)DoubleValue;
        }
    }

    public double DoubleValue =>
        _kind == JsonKind.Number && _text != null
            ? double.Parse(_text, NumberStyles.Float, CultureInfo.InvariantCulture)
            : 0;

    public JsonValue? this[string key] =>
        _kind == JsonKind.Object && key != null && _index!.TryGetValue(key, out var value) ? value : null;

    public JsonValue? this[int index] =>
        _kind == JsonKind.Array && index >= 0 && index < _items!.Count ? _items[index] : null;

    public bool Has(string key) => this[key] != null;

    public string? GetString(string key, string? fallback = null) => this[key]?.StringValue ?? fallback;

    public long GetLong(string key, long fallback = 0)
    {
        var value = this[key];
        return value != null && value.IsNumber ? value.LongValue : fallback;
    }

    public int GetInt(string key, int fallback = 0)
    {
        var value = this[key];
        return value != null && value.IsNumber ? (int)value.LongValue : fallback;
    }

    public double GetDouble(string key, double fallback = 0)
    {
        var value = this[key];
        return value != null && value.IsNumber ? value.DoubleValue : fallback;
    }

    public bool GetBool(string key, bool fallback = false)
    {
        var value = this[key];
        return value != null && value.IsBool ? value.BoolValue : fallback;
    }

    public IReadOnlyList<JsonValue>? GetArray(string key)
    {
        var value = this[key];
        return value != null && value.IsArray ? value.Items : null;
    }

    public JsonValue? GetObject(string key)
    {
        var value = this[key];
        return value != null && value.IsObject ? value : null;
    }

    public static JsonValue Parse(string text)
    {
        if (text == null) throw new ArgumentNullException(nameof(text));
        var parser = new Parser(text);
        var value = parser.ParseValue();
        parser.SkipWhitespace();
        if (!parser.AtEnd) throw new JsonParseException("JSON 结束后还有多余内容", parser.Position);
        return value;
    }

    private sealed class Parser
    {
        private readonly string _text;
        private int _pos;

        public Parser(string text)
        {
            _text = text;
            if (_text.Length > 0 && _text[0] == '\uFEFF') _pos = 1;
        }

        public int Position => _pos;
        public bool AtEnd => _pos >= _text.Length;

        public void SkipWhitespace()
        {
            while (_pos < _text.Length)
            {
                var c = _text[_pos];
                if (c == ' ' || c == '\t' || c == '\n' || c == '\r') _pos++;
                else break;
            }
        }

        public JsonValue ParseValue()
        {
            SkipWhitespace();
            if (AtEnd) throw new JsonParseException("JSON 意外结束", _pos);
            var c = _text[_pos];
            return c switch
            {
                '{' => ParseObject(),
                '[' => ParseArray(),
                '"' => new JsonValue(ParseString()),
                't' => ParseLiteral("true", true),
                'f' => ParseLiteral("false", false),
                'n' => ParseNull(),
                _ => ParseNumber(),
            };
        }

        private JsonValue ParseLiteral(string literal, bool value)
        {
            if (_pos + literal.Length > _text.Length ||
                string.CompareOrdinal(_text, _pos, literal, 0, literal.Length) != 0)
            {
                throw new JsonParseException($"无效的字面量，期望 {literal}", _pos);
            }
            _pos += literal.Length;
            return new JsonValue(value);
        }

        private JsonValue ParseNull()
        {
            var value = ParseLiteral("null", false);
            return Null;
        }

        private JsonValue ParseNumber()
        {
            var start = _pos;
            if (_text[_pos] == '-') _pos++;
            while (_pos < _text.Length && char.IsDigit(_text[_pos])) _pos++;
            if (_pos < _text.Length && _text[_pos] == '.')
            {
                _pos++;
                while (_pos < _text.Length && char.IsDigit(_text[_pos])) _pos++;
            }
            if (_pos < _text.Length && (_text[_pos] == 'e' || _text[_pos] == 'E'))
            {
                _pos++;
                if (_pos < _text.Length && (_text[_pos] == '+' || _text[_pos] == '-')) _pos++;
                while (_pos < _text.Length && char.IsDigit(_text[_pos])) _pos++;
            }
            if (_pos == start) throw new JsonParseException("无效的数字", _pos);
            var raw = _text.Substring(start, _pos - start);
            return new JsonValue(JsonKind.Number, raw);
        }

        private string ParseString()
        {
            if (_text[_pos] != '"') throw new JsonParseException("期望字符串", _pos);
            _pos++;
            var sb = new StringBuilder();
            while (_pos < _text.Length)
            {
                var c = _text[_pos++];
                if (c == '"') return sb.ToString();
                if (c != '\\') { sb.Append(c); continue; }
                if (_pos >= _text.Length) throw new JsonParseException("字符串转义意外结束", _pos);
                var esc = _text[_pos++];
                switch (esc)
                {
                    case '"': sb.Append('"'); break;
                    case '\\': sb.Append('\\'); break;
                    case '/': sb.Append('/'); break;
                    case 'b': sb.Append('\b'); break;
                    case 'f': sb.Append('\f'); break;
                    case 'n': sb.Append('\n'); break;
                    case 'r': sb.Append('\r'); break;
                    case 't': sb.Append('\t'); break;
                    case 'u':
                        if (_pos + 4 > _text.Length) throw new JsonParseException("Unicode 转义不完整", _pos);
                        var hex = _text.Substring(_pos, 4);
                        if (!ushort.TryParse(hex, NumberStyles.HexNumber, CultureInfo.InvariantCulture, out var code))
                        {
                            throw new JsonParseException("Unicode 转义无效", _pos);
                        }
                        sb.Append((char)code);
                        _pos += 4;
                        break;
                    default:
                        throw new JsonParseException($"未知转义 \\{esc}", _pos - 1);
                }
            }
            throw new JsonParseException("字符串未闭合", _pos);
        }

        private JsonValue ParseArray()
        {
            _pos++;
            var items = new List<JsonValue>();
            SkipWhitespace();
            if (!AtEnd && _text[_pos] == ']') { _pos++; return new JsonValue(items); }
            while (true)
            {
                items.Add(ParseValue());
                SkipWhitespace();
                if (AtEnd) throw new JsonParseException("数组未闭合", _pos);
                if (_text[_pos] == ',') { _pos++; continue; }
                if (_text[_pos] == ']') { _pos++; return new JsonValue(items); }
                throw new JsonParseException("数组元素之间缺少逗号", _pos);
            }
        }

        private JsonValue ParseObject()
        {
            _pos++;
            var members = new List<KeyValuePair<string, JsonValue>>();
            SkipWhitespace();
            if (!AtEnd && _text[_pos] == '}') { _pos++; return new JsonValue(members); }
            while (true)
            {
                SkipWhitespace();
                if (AtEnd || _text[_pos] != '"') throw new JsonParseException("对象键必须是字符串", _pos);
                var key = ParseString();
                SkipWhitespace();
                if (AtEnd || _text[_pos] != ':') throw new JsonParseException("对象键后缺少冒号", _pos);
                _pos++;
                members.Add(new KeyValuePair<string, JsonValue>(key, ParseValue()));
                SkipWhitespace();
                if (AtEnd) throw new JsonParseException("对象未闭合", _pos);
                if (_text[_pos] == ',') { _pos++; continue; }
                if (_text[_pos] == '}') { _pos++; return new JsonValue(members); }
                throw new JsonParseException("对象成员之间缺少逗号", _pos);
            }
        }
    }
}
