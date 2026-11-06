using System.Text.Json;
using PezMax.Core.Api;
using PezMax.Core.Validation;
using Xunit;

namespace PezMax.Core.Tests;

/// <summary>
/// 账户表单校验矩阵：与 `spec/fixtures/auth/validation.json` 逐条对齐。
/// 文案与判定顺序都照上游 handler 抄，改了这里就等于改了界面提示。
/// </summary>
public class AuthFixtureTests
{
    [Fact]
    public void FormValidation_MatchesAuthFixtures()
    {
        using var document = Spec.LoadJson("fixtures", "auth", "validation.json");
        var cases = document.RootElement;
        Assert.Equal(JsonValueKind.Array, cases.ValueKind);

        var checkedCases = 0;
        foreach (var item in cases.EnumerateArray())
        {
            var id = item.GetProperty("id").GetString()!;
            var input = item.GetProperty("input");
            var expect = item.GetProperty("expect");

            string? expected = expect.TryGetProperty("error", out var error) && error.ValueKind == JsonValueKind.String
                ? error.GetString()
                : null;

            var actual = input.GetProperty("action").GetString() switch
            {
                "register" => FormValidation.ValidateRegister(BuildRegister(input)),
                "reset" => FormValidation.ValidateResetPassword(BuildReset(input)),
                "login" => FormValidation.ValidateLogin(Text(input, "username"), Text(input, "password")),
                var action => throw new Xunit.Sdk.XunitException($"{id}: 未知 action {action}"),
            };

            Assert.True(
                expected == actual,
                $"{id}: 期望「{expected ?? "(通过)"}」，实际「{actual ?? "(通过)"}」");
            checkedCases++;
        }

        Assert.True(checkedCases >= 25, $"账户校验夹具跑到的用例太少：{checkedCases}");
    }

    [Fact]
    public void ValidateRegister_TrimsUserNameBeforeLengthCheck()    {
        var request = ValidRegister();
        request.UserName = "  " + new string('中', 62) + "  ";
        Assert.Null(FormValidation.ValidateRegister(request));

        request.UserName = "  " + new string('中', 65) + "  ";
        Assert.Equal("用户名格式错误", FormValidation.ValidateRegister(request));
    }

    [Fact]
    public void ValidateRegister_CanSkipCaptchaCheckForLocalPreviews()
    {
        var request = ValidRegister();
        request.Code = "";
        request.Uuid = "";
        Assert.Equal("验证码错误或已过期", FormValidation.ValidateRegister(request));
        Assert.Null(FormValidation.ValidateRegister(request, validateCaptcha: false));
    }

    [Fact]
    public void ValidateResetPassword_LeavesTicketMatchingToTheServer()
    {
        var request = new ResetPasswordRequest
        {
            UserName = "alice",
            Code = "any-non-empty",
            Uuid = "uuid-fixture",
            SecurityAnswerOne = "a1",
            SecurityAnswerTwo = "a2",
            SecurityAnswerThree = "a3",
            NewPassword = "newsecret",
            ConfirmPassword = "newsecret",
        };

        // 第二步的 code 只做非空校验，真正的凭证是第一步签发的工单；客户端无从判断对错。
        Assert.Null(FormValidation.ValidateResetPassword(request));
    }

    [Fact]
    public void StepHelpers_SplitTheSameRulesAsTheWholeRegisterCheck()
    {
        // 界面把注册拆成三步，每步用更小的校验函数；拆开后拼起来必须与整体一致。
        Assert.Equal("用户名不能为空", FormValidation.ValidateAccount("  ", "12345", "12345"));
        Assert.Equal("两次输入的密码不一致", FormValidation.ValidateAccount("alice", "12345", "54321"));
        Assert.Null(FormValidation.ValidateAccount("alice", "12345", "12345"));

        Assert.Equal("密保问题不能为空", FormValidation.ValidateSecurity("q1", "", "q3", "a1", "a2", "a3"));
        Assert.Equal("密保答案不能为空", FormValidation.ValidateSecurity("q1", "q2", "q3", "a1", " ", "a3"));
        Assert.Null(FormValidation.ValidateSecurity("q1", "q2", "q3", "a1", "a2", "a3"));

        Assert.Equal("验证码错误或已过期", FormValidation.ValidateCaptchaEntry("", "uuid-fixture"));
        Assert.Equal("验证码错误或已过期", FormValidation.ValidateCaptchaEntry("123456", ""));
        Assert.Null(FormValidation.ValidateCaptchaEntry("123456", "uuid-fixture"));
    }

    private static RegisterRequest ValidRegister() => new()
    {
        UserName = "alice",
        Password = "12345",
        ConfirmPassword = "12345",
        SecurityQuestionOne = "q1",
        SecurityQuestionTwo = "q2",
        SecurityQuestionThree = "q3",
        SecurityAnswerOne = "a1",
        SecurityAnswerTwo = "a2",
        SecurityAnswerThree = "a3",
        Code = "123456",
        Uuid = "uuid-fixture",
    };

    private static RegisterRequest BuildRegister(JsonElement input) => new()
    {
        UserName = Text(input, "username"),
        Password = Text(input, "password"),
        ConfirmPassword = Text(input, "confirmPassword"),
        SecurityQuestionOne = Text(input, "securityQuestionOne"),
        SecurityQuestionTwo = Text(input, "securityQuestionTwo"),
        SecurityQuestionThree = Text(input, "securityQuestionThree"),
        SecurityAnswerOne = Text(input, "securityAnswerOne"),
        SecurityAnswerTwo = Text(input, "securityAnswerTwo"),
        SecurityAnswerThree = Text(input, "securityAnswerThree"),
        Avatar = Text(input, "avatar"),
        Code = Text(input, "code"),
        Uuid = Text(input, "uuid"),
    };

    private static ResetPasswordRequest BuildReset(JsonElement input) => new()
    {
        UserName = Text(input, "username"),
        Code = Text(input, "code"),
        Uuid = Text(input, "uuid"),
        SecurityAnswerOne = Text(input, "securityAnswerOne"),
        SecurityAnswerTwo = Text(input, "securityAnswerTwo"),
        SecurityAnswerThree = Text(input, "securityAnswerThree"),
        NewPassword = Text(input, "newPassword"),
        ConfirmPassword = Text(input, "confirmPassword"),
    };

    private static string Text(JsonElement input, string name) =>
        input.TryGetProperty(name, out var value) && value.ValueKind == JsonValueKind.String
            ? value.GetString()!
            : "";
}
