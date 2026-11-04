using System;
using System.Collections.Generic;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using PezMax.Core.Json;
using PezMax.Core.Models;
using PezMax.Core.Net;

namespace PezMax.Core.Api;

public sealed class AvatarUploadResult
{
    public string Url { get; init; } = "";
    public string Storage { get; init; } = "";
    public string Name { get; init; } = "";

    public static AvatarUploadResult From(JsonValue? value) => new()
    {
        Url = JsonModel.Str(value, "url"),
        Storage = JsonModel.Str(value, "storage"),
        Name = JsonModel.Str(value, "name"),
    };
}

public sealed class SecurityAnswerSet
{
    public string One { get; set; } = "";
    public string Two { get; set; } = "";
    public string Three { get; set; } = "";
}

public sealed class SecurityQuestionSet
{
    public string One { get; set; } = "";
    public string Two { get; set; } = "";
    public string Three { get; set; } = "";
}

public sealed class ProfileApi
{
    private readonly DatumHttp _http;

    public ProfileApi(DatumHttp http) => _http = http;

    public async Task<UserInfo> GetProfileAsync(CancellationToken cancellationToken = default) =>
        ApiHelpers.EnsureData(
            await _http.GetAsync("/datum/desktop/user/profile", cancellationToken: cancellationToken).ConfigureAwait(false),
            UserInfo.From);

    public async Task<ProfileStats> GetStatsAsync(CancellationToken cancellationToken = default) =>
        ApiHelpers.EnsureData(
            await _http.GetAsync("/datum/desktop/user/profile/stats", cancellationToken: cancellationToken).ConfigureAwait(false),
            ProfileStats.From);

    public async Task<string> UpdateUserNameAsync(string userName, CancellationToken cancellationToken = default)
    {
        var body = new JsonObjectBuilder().String("userName", userName).ToString();
        var response = ApiHelpers.EnsureSuccess(
            await _http.PutJsonAsync("/datum/desktop/user/profile/username", body, cancellationToken).ConfigureAwait(false));
        return response.Data?.GetString("userName") ?? userName;
    }

    public async Task<string> UpdateAvatarAsync(string avatar, CancellationToken cancellationToken = default)
    {
        var body = new JsonObjectBuilder().String("avatar", avatar).ToString();
        var response = ApiHelpers.EnsureSuccess(
            await _http.PutJsonAsync("/datum/desktop/user/profile/avatar", body, cancellationToken).ConfigureAwait(false));
        return response.Data?.GetString("avatar") ?? avatar;
    }

    public Task<AvatarUploadResult> UploadAvatarAsync(
        string fileName,
        Stream stream,
        string contentType,
        CancellationToken cancellationToken = default) =>
        UploadAvatarCoreAsync(fileName, stream, contentType, cancellationToken);

    private async Task<AvatarUploadResult> UploadAvatarCoreAsync(
        string fileName,
        Stream stream,
        string contentType,
        CancellationToken cancellationToken)
    {
        var response = await _http.UploadAsync(
            "/datum/desktop/user/profile/avatar/upload",
            Array.Empty<KeyValuePair<string, string?>>(),
            fileName,
            stream,
            contentType,
            cancellationToken).ConfigureAwait(false);
        return ApiHelpers.EnsureData(response, AvatarUploadResult.From);
    }

    public async Task VerifyPasswordAsync(string password, CancellationToken cancellationToken = default)
    {
        var body = new JsonObjectBuilder().String("password", password).ToString();
        ApiHelpers.EnsureTrue(
            await _http.PostJsonAsync("/datum/desktop/user/profile/password/verify", body, cancellationToken).ConfigureAwait(false));
    }

    public async Task UpdatePasswordAsync(string oldPassword, string newPassword, CancellationToken cancellationToken = default)
    {
        var body = new JsonObjectBuilder()
            .String("oldPassword", oldPassword)
            .String("newPassword", newPassword)
            .ToString();
        ApiHelpers.EnsureTrue(
            await _http.PutJsonAsync("/datum/desktop/user/profile/password", body, cancellationToken).ConfigureAwait(false));
    }

    public async Task ResetPasswordBySecurityAsync(
        SecurityAnswerSet answers,
        string newPassword,
        CancellationToken cancellationToken = default)
    {
        var body = new JsonObjectBuilder()
            .String("securityAnswerOne", answers.One)
            .String("securityAnswerTwo", answers.Two)
            .String("securityAnswerThree", answers.Three)
            .String("newPassword", newPassword)
            .ToString();
        ApiHelpers.EnsureTrue(
            await _http.PutJsonAsync("/datum/desktop/user/profile/password/by-security", body, cancellationToken).ConfigureAwait(false));
    }

    public async Task<IReadOnlyList<string>> GetSecurityQuestionsAsync(CancellationToken cancellationToken = default)
    {
        var response = ApiHelpers.EnsureSuccess(
            await _http.GetAsync("/datum/desktop/user/profile/security", cancellationToken: cancellationToken).ConfigureAwait(false));
        var result = new List<string>();
        if (response.Data?.GetArray("questions") is { } values)
        {
            foreach (var value in values)
            {
                if (value.StringValue is { } text && !string.IsNullOrWhiteSpace(text)) result.Add(text);
            }
        }
        return result;
    }

    public async Task<bool> VerifySecurityAnswerAsync(string answer, CancellationToken cancellationToken = default)
    {
        var body = new JsonObjectBuilder().String("answer", answer).ToString();
        var response = ApiHelpers.EnsureSuccess(
            await _http.PostJsonAsync("/datum/desktop/user/profile/security/answer/verify", body, cancellationToken).ConfigureAwait(false));
        return response.Data?.GetBool("verified") ?? false;
    }

    public async Task UpdateSecurityAsync(
        string password,
        SecurityQuestionSet questions,
        SecurityAnswerSet answers,
        CancellationToken cancellationToken = default)
    {
        var body = new JsonObjectBuilder()
            .String("password", password)
            .String("securityQuestionOne", questions.One)
            .String("securityQuestionTwo", questions.Two)
            .String("securityQuestionThree", questions.Three)
            .String("securityAnswerOne", answers.One)
            .String("securityAnswerTwo", answers.Two)
            .String("securityAnswerThree", answers.Three)
            .ToString();
        ApiHelpers.EnsureTrue(
            await _http.PutJsonAsync("/datum/desktop/user/profile/security", body, cancellationToken).ConfigureAwait(false));
    }
}
