using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using PezMax.Core.Json;
using PezMax.Core.Models;
using PezMax.Core.Net;

namespace PezMax.Core.Api;

public sealed class RegisterRequest
{
    public string UserName { get; set; } = "";
    public string Password { get; set; } = "";
    public string ConfirmPassword { get; set; } = "";
    public string SecurityQuestionOne { get; set; } = "";
    public string SecurityAnswerOne { get; set; } = "";
    public string SecurityQuestionTwo { get; set; } = "";
    public string SecurityAnswerTwo { get; set; } = "";
    public string SecurityQuestionThree { get; set; } = "";
    public string SecurityAnswerThree { get; set; } = "";
    public string Avatar { get; set; } = "";
    public string Code { get; set; } = "";
    public string Uuid { get; set; } = "";
}

public sealed class ResetPasswordRequest
{
    public string UserName { get; set; } = "";
    public string Code { get; set; } = "";
    public string Uuid { get; set; } = "";
    public string SecurityAnswerOne { get; set; } = "";
    public string SecurityAnswerTwo { get; set; } = "";
    public string SecurityAnswerThree { get; set; } = "";
    public string NewPassword { get; set; } = "";
    public string ConfirmPassword { get; set; } = "";
}

public sealed class AuthApi
{
    private readonly DatumHttp _http;

    public AuthApi(DatumHttp http) => _http = http;

    public async Task<CaptchaResult> GetCaptchaAsync(CancellationToken cancellationToken = default) =>
        ApiHelpers.EnsureData(
            await _http.GetAsync("/datum/user/captchaImage", cancellationToken: cancellationToken).ConfigureAwait(false),
            CaptchaResult.From);

    public async Task<LoginResult> LoginAsync(
        string userName,
        string password,
        string code,
        string uuid,
        CancellationToken cancellationToken = default)
    {
        var body = new JsonObjectBuilder()
            .String("username", userName)
            .String("password", password)
            .String("code", code)
            .String("uuid", uuid)
            .ToString();

        return ApiHelpers.EnsureData(
            await _http.PostJsonAsync("/datum/user/login", body, cancellationToken).ConfigureAwait(false),
            LoginResult.From);
    }

    public async Task<long> RegisterAsync(RegisterRequest request, CancellationToken cancellationToken = default)
    {
        var body = new JsonObjectBuilder()
            .String("username", request.UserName)
            .String("password", request.Password)
            .String("confirmPassword", request.ConfirmPassword)
            .String("securityQuestionOne", request.SecurityQuestionOne)
            .String("securityAnswerOne", request.SecurityAnswerOne)
            .String("securityQuestionTwo", request.SecurityQuestionTwo)
            .String("securityAnswerTwo", request.SecurityAnswerTwo)
            .String("securityQuestionThree", request.SecurityQuestionThree)
            .String("securityAnswerThree", request.SecurityAnswerThree)
            .String("avatar", request.Avatar)
            .String("code", request.Code)
            .String("uuid", request.Uuid)
            .ToString();

        var response = ApiHelpers.EnsureSuccess(
            await _http.PostJsonAsync("/datum/user/register", body, cancellationToken).ConfigureAwait(false));
        return response.Data?.GetLong("userId") ?? 0;
    }

    public async Task LogoutAsync(CancellationToken cancellationToken = default) =>
        ApiHelpers.EnsureTrue(
            await _http.PostJsonAsync("/datum/user/logout", "{}", cancellationToken).ConfigureAwait(false));

    public async Task<GetInfoResult> GetInfoAsync(CancellationToken cancellationToken = default) =>
        ApiHelpers.EnsureData(
            await _http.GetAsync("/datum/user/getInfo", cancellationToken: cancellationToken).ConfigureAwait(false),
            GetInfoResult.From);

    public async Task<IReadOnlyList<string>> GetSecurityQuestionsAsync(
        string userName,
        string uuid,
        string code,
        CancellationToken cancellationToken = default)
    {
        var query = new[]
        {
            new KeyValuePair<string, string?>("userName", userName),
            new KeyValuePair<string, string?>("uuid", uuid),
            new KeyValuePair<string, string?>("code", code),
        };
        var response = ApiHelpers.EnsureSuccess(
            await _http.GetAsync("/datum/user/securityQuestions", query, cancellationToken).ConfigureAwait(false));

        var questions = new List<string>();
        if (response.Data?.IsArray == true)
        {
            foreach (var item in response.Data.Items)
            {
                var question = item.GetString("question");
                if (!string.IsNullOrWhiteSpace(question)) questions.Add(question!);
            }
        }
        return questions;
    }

    public async Task ResetPasswordBySecurityAsync(
        ResetPasswordRequest request,
        CancellationToken cancellationToken = default)
    {
        var body = new JsonObjectBuilder()
            .String("username", request.UserName)
            .String("code", request.Code)
            .String("uuid", request.Uuid)
            .String("securityAnswerOne", request.SecurityAnswerOne)
            .String("securityAnswerTwo", request.SecurityAnswerTwo)
            .String("securityAnswerThree", request.SecurityAnswerThree)
            .String("newPassword", request.NewPassword)
            .String("confirmPassword", request.ConfirmPassword)
            .ToString();

        ApiHelpers.EnsureTrue(
            await _http.PostJsonAsync("/datum/user/resetPasswordBySecurity", body, cancellationToken).ConfigureAwait(false));
    }
}
