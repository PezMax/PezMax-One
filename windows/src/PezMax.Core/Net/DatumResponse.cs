using System;
using System.Collections.Generic;
using PezMax.Core.Json;

namespace PezMax.Core.Net;

public enum DatumResultKind
{
    Success,
    List,
    Error,
    Transport,
    SessionExpired,
}

public sealed class DatumResponse
{
    public DatumResultKind Kind { get; init; }
    public int HttpStatus { get; init; }
    public int Code { get; init; }
    public string Message { get; init; } = "";
    public JsonValue? Data { get; init; }
    public IReadOnlyList<JsonValue>? Rows { get; init; }
    public long Total { get; init; }
    public string? Hash { get; init; }
    public bool Unchanged { get; init; }

    public bool IsSuccess => Kind == DatumResultKind.Success;
    public bool IsList => Kind == DatumResultKind.List;
    public bool IsSessionExpired => Kind == DatumResultKind.SessionExpired;
    public bool IsTransportError => Kind == DatumResultKind.Transport;

    public DatumException ToException() => new(
        Message,
        HttpStatus,
        Code,
        Kind == DatumResultKind.SessionExpired,
        Kind == DatumResultKind.Transport && HttpStatus >= 500);
}

public sealed class DatumException : Exception
{
    public DatumException(string message, int httpStatus, int code, bool sessionExpired, bool retryable)
        : base(message)
    {
        HttpStatus = httpStatus;
        Code = code;
        SessionExpired = sessionExpired;
        Retryable = retryable;
    }

    public int HttpStatus { get; }
    public int Code { get; }
    public bool SessionExpired { get; }
    public bool Retryable { get; }
}

public static class DatumEnvelope
{
    public static DatumResponse Parse(int status, string? contentType, string? body)
    {
        var text = body ?? "";
        var isJson = IsJsonContentType(contentType);

        if (status == 401)
        {
            var message = isJson ? ReadMessage(text) : "";
            return new DatumResponse
            {
                Kind = DatumResultKind.SessionExpired,
                HttpStatus = status,
                Code = 401,
                Message = message,
            };
        }

        if (!isJson)
        {
            return new DatumResponse
            {
                Kind = DatumResultKind.Transport,
                HttpStatus = status,
                Message = DefaultTransportMessage(status),
            };
        }

        JsonValue root;
        try
        {
            root = JsonValue.Parse(text);
        }
        catch (JsonParseException)
        {
            return new DatumResponse
            {
                Kind = DatumResultKind.Transport,
                HttpStatus = status,
                Message = DefaultTransportMessage(status),
            };
        }

        if (!root.IsObject)
        {
            return new DatumResponse
            {
                Kind = DatumResultKind.Transport,
                HttpStatus = status,
                Message = DefaultTransportMessage(status),
            };
        }

        var messageText = ReadMessage(root);
        var code = root.GetInt("code", status);

        if (status < 200 || status >= 300)
        {
            return new DatumResponse
            {
                Kind = DatumResultKind.Error,
                HttpStatus = status,
                Code = code,
                Message = messageText,
                Data = root["data"],
            };
        }

        var rows = root.GetArray("rows");
        if (rows != null)
        {
            return new DatumResponse
            {
                Kind = DatumResultKind.List,
                HttpStatus = status,
                Code = code,
                Message = messageText,
                Rows = rows,
                Total = root.Has("total") ? root.GetLong("total") : rows.Count,
            };
        }

        if (code == 0)
        {
            return new DatumResponse
            {
                Kind = DatumResultKind.Success,
                HttpStatus = status,
                Code = 0,
                Message = messageText,
                Data = root["data"],
                Hash = root.GetString("hash"),
                Unchanged = root.GetBool("unchanged"),
            };
        }

        return new DatumResponse
        {
            Kind = DatumResultKind.Error,
            HttpStatus = status,
            Code = code,
            Message = messageText,
            Data = root["data"],
        };
    }

    private static bool IsJsonContentType(string? contentType) =>
        contentType != null &&
        contentType.IndexOf("application/json", StringComparison.OrdinalIgnoreCase) >= 0;

    private static string ReadMessage(string text)
    {
        try
        {
            return ReadMessage(JsonValue.Parse(text));
        }
        catch (JsonParseException)
        {
            return "";
        }
    }

    private static string ReadMessage(JsonValue root)
    {
        var msg = root.GetString("msg");
        if (!string.IsNullOrWhiteSpace(msg)) return msg!;
        var message = root.GetString("message");
        return string.IsNullOrWhiteSpace(message) ? "" : message!;
    }

    private static string DefaultTransportMessage(int status) =>
        status >= 500 ? "服务暂不可用，请稍后重试" : "响应格式不正确";
}
