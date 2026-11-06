using System.Text.Json;
using PezMax.Core.Net;
using Xunit;

namespace PezMax.Core.Tests;

public class EnvelopeFixtureTests
{
    [Fact]
    public void Parse_MatchesEnvelopeFixtures()
    {
        using var document = Spec.LoadJson("fixtures", "envelope", "cases.json");
        var cases = document.RootElement;
        Assert.Equal(JsonValueKind.Array, cases.ValueKind);

        var checkedCases = 0;
        foreach (var item in cases.EnumerateArray())
        {
            var id = item.GetProperty("id").GetString()!;
            var input = item.GetProperty("input");

            if (input.TryGetProperty("bodyBase64", out _))
            {
                Assert.Equal("stream", input.GetProperty("requestKind").GetString());
                continue;
            }

            var status = input.GetProperty("status").GetInt32();
            var contentType = input.GetProperty("contentType").GetString();
            var body = input.GetProperty("body").GetString();

            var response = DatumEnvelope.Parse(status, contentType, body);
            var expect = item.GetProperty("expect");
            var expectedKind = expect.GetProperty("kind").GetString();

            switch (expectedKind)
            {
                case "success":
                    Assert.True(response.Kind == DatumResultKind.Success, $"{id}: kind = {response.Kind}");
                    Assert.Equal(0, response.Code);
                    if (expect.TryGetProperty("data", out var data) && data.ValueKind == JsonValueKind.Null)
                        Assert.True(response.Data == null || response.Data.IsNull, $"{id}: data should be null");
                    break;
                case "list":
                    Assert.True(response.Kind == DatumResultKind.List, $"{id}: kind = {response.Kind}");
                    Assert.True(expect.GetProperty("total").GetInt64() == response.Total, $"{id}: total");
                    Assert.True(expect.GetProperty("rows").GetArrayLength() == response.Rows!.Count, $"{id}: rows");
                    Assert.False(response.Data?.IsObject == true, $"{id}: list must not expose data.rows");
                    break;
                case "error":
                    Assert.True(response.Kind == DatumResultKind.Error, $"{id}: kind = {response.Kind}");
                    Assert.True(expect.GetProperty("code").GetInt32() == response.Code, $"{id}: code");
                    AssertMessage(expect, response, id);
                    break;
                case "transport":
                    Assert.True(response.Kind == DatumResultKind.Transport, $"{id}: kind = {response.Kind}");
                    break;
                case "session-expired":
                    Assert.True(response.Kind == DatumResultKind.SessionExpired, $"{id}: kind = {response.Kind}");
                    Assert.Equal(401, response.HttpStatus);
                    Assert.Equal("clear", expect.GetProperty("session").GetString());
                    AssertMessage(expect, response, id);
                    break;
                case "state":
                    Assert.True(response.Kind == DatumResultKind.Success, $"{id}: kind = {response.Kind}");
                    Assert.True(expect.GetProperty("unchanged").GetBoolean() == response.Unchanged, $"{id}: unchanged");
                    Assert.True(expect.GetProperty("hash").GetString() == response.Hash, $"{id}: hash");
                    break;
                default:
                    throw new Xunit.Sdk.XunitException($"{id}: 未知 kind {expectedKind}");
            }

            checkedCases++;
        }

        Assert.True(checkedCases >= 12, $"信封夹具跑到的用例太少：{checkedCases}");
    }

    [Fact]
    public void Parse_NonJsonError_IsTransportBut401StillExpiresSession()
    {
        var transport = DatumEnvelope.Parse(502, "text/html", "<html>bad gateway</html>");
        Assert.Equal(DatumResultKind.Transport, transport.Kind);
        Assert.False(transport.IsSessionExpired);

        var expired = DatumEnvelope.Parse(401, "text/html", "<html>unauthorized</html>");
        Assert.Equal(DatumResultKind.SessionExpired, expired.Kind);
        Assert.True(expired.IsSessionExpired);
    }

    private static void AssertMessage(JsonElement expect, DatumResponse response, string id)
    {
        var expected = expect.TryGetProperty("message", out var value) ? value.GetString() : null;
        if (!string.IsNullOrEmpty(expected)) Assert.True(expected == response.Message, $"{id}: message = {response.Message}");
    }
}
