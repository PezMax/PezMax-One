using System;
using System.Collections.Generic;
using PezMax.Core.Json;
using PezMax.Core.Models;
using PezMax.Core.Net;

namespace PezMax.Core.Api;

internal static class ApiHelpers
{
    public static DatumResponse EnsureSuccess(DatumResponse response)
    {
        if (response.IsSuccess) return response;
        throw response.ToException();
    }

    public static T EnsureData<T>(DatumResponse response, Func<JsonValue?, T> map)
    {
        EnsureSuccess(response);
        return map(response.Data);
    }

    public static PageResult<T> EnsurePage<T>(DatumResponse response, Func<JsonValue?, T> map)
    {
        if (!response.IsList) throw response.ToException();
        var rows = new List<T>();
        if (response.Rows != null)
        {
            foreach (var row in response.Rows) rows.Add(map(row));
        }
        return new PageResult<T>(rows, response.Total);
    }

    public static IReadOnlyList<T> MapArray<T>(JsonValue? data, Func<JsonValue?, T> map)
    {
        var result = new List<T>();
        if (data?.IsArray != true) return result;
        foreach (var item in data.Items) result.Add(map(item));
        return result;
    }

    public static bool EnsureTrue(DatumResponse response)
    {
        EnsureSuccess(response);
        return response.Data == null || response.Data.IsNull || response.Data.BoolValue;
    }
}
