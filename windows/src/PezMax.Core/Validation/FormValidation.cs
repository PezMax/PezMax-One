using System.Text;
using PezMax.Core.Api;

namespace PezMax.Core.Validation;

public static class FormValidation
{
    public static string? ValidateLogin(string userName, string password)
    {
        if (string.IsNullOrWhiteSpace(userName) || string.IsNullOrEmpty(password))
            return "用户名或密码错误";
        return null;
    }

    public static string? ValidateUserName(string userName)
    {
        var text = (userName ?? "").Trim();
        if (text.Length == 0) return "用户名不能为空";
        return RuneCount(text) > 64 ? "用户名格式错误" : null;
    }

    public static string? ValidatePassword(string password)
    {
        var bytes = Encoding.UTF8.GetByteCount(password ?? "");
        return bytes is < 5 or > 64 ? "密码格式错误：长度需在 5 到 64 位之间" : null;
    }

    /// <summary>注册第一步：用户名与两次密码。互不相同的判定顺序与上游 592-604 行一致。</summary>
    public static string? ValidateAccount(string? userName, string? password, string? confirmPassword)
    {
        var name = ValidateUserName(userName);
        if (name != null) return name;

        var secret = ValidatePassword(password);
        if (secret != null) return secret;

        return password != confirmPassword ? "两次输入的密码不一致" : null;
    }

    /// <summary>注册第二步：三问三答。空白与空串同等对待（上游先 TrimSpace）。</summary>
    public static string? ValidateSecurity(
        string? questionOne,
        string? questionTwo,
        string? questionThree,
        string? answerOne,
        string? answerTwo,
        string? answerThree)
    {
        if (string.IsNullOrWhiteSpace(questionOne) ||
            string.IsNullOrWhiteSpace(questionTwo) ||
            string.IsNullOrWhiteSpace(questionThree))
            return "密保问题不能为空";
        if (string.IsNullOrWhiteSpace(answerOne) ||
            string.IsNullOrWhiteSpace(answerTwo) ||
            string.IsNullOrWhiteSpace(answerThree))
            return "密保答案不能为空";
        return null;
    }

    public static string? ValidateRegister(RegisterRequest request, bool validateCaptcha = true)
    {
        var account = ValidateAccount(request.UserName, request.Password, request.ConfirmPassword);
        if (account != null) return account;

        var security = ValidateSecurity(
            request.SecurityQuestionOne, request.SecurityQuestionTwo, request.SecurityQuestionThree,
            request.SecurityAnswerOne, request.SecurityAnswerTwo, request.SecurityAnswerThree);
        if (security != null) return security;

        if (Encoding.UTF8.GetByteCount(request.Avatar ?? "") > 255) return "头像地址格式错误";
        if (validateCaptcha)
        {
            var captcha = ValidateCaptchaEntry(request.Code, request.Uuid);
            if (captcha != null) return captcha;
        }
        return null;
    }

    /// <summary>
    /// 验证码两件套：`code` 与 `uuid` 缺一不可，缺哪一种上游都回同一句（`datum_user_auth.go:718-720`）。
    /// 验证码本身是否正确只能由服务端判断，这里只管「填没填」。
    /// </summary>
    public static string? ValidateCaptchaEntry(string? code, string? uuid) =>
        string.IsNullOrWhiteSpace(code) || string.IsNullOrWhiteSpace(uuid) ? "验证码错误或已过期" : null;

    public static string? ValidateResetPassword(ResetPasswordRequest request)
    {
        // 上游把「用户名 / uuid / code 为空」合并成一个分支，文案都是验证码错误（:718-720）。
        if (string.IsNullOrWhiteSpace(request.UserName)) return "验证码错误或已过期";
        var captcha = ValidateCaptchaEntry(request.Code, request.Uuid);
        if (captcha != null) return captcha;
        if (string.IsNullOrWhiteSpace(request.SecurityAnswerOne) ||
            string.IsNullOrWhiteSpace(request.SecurityAnswerTwo) ||
            string.IsNullOrWhiteSpace(request.SecurityAnswerThree))
            return "密保答案不能为空";

        var password = ValidatePassword(request.NewPassword);
        if (password != null) return password;
        return request.NewPassword != request.ConfirmPassword ? "两次输入的密码不一致" : null;
    }

    public static string? ValidateReportReason(string reason)
    {
        var text = (reason ?? "").Trim();
        if (text.Length == 0 || RuneCount(text) > 500) return "举报原因不能为空且不超过 500 字";
        return null;
    }

    public static string? ValidateBookmark(string? url, string? title)
    {
        if (string.IsNullOrWhiteSpace(url) || Encoding.UTF8.GetByteCount(url) > 512)
            return "网址不能为空且不超过 512 个字符";
        if (string.IsNullOrWhiteSpace(title) || RuneCount(title!) > 500)
            return "标题不能为空且不超过 500 个字符";
        return null;
    }

    public static int RuneCount(string value)
    {
        var count = 0;
        foreach (var _ in EnumerateRunes(value)) count++;
        return count;
    }

    private static System.Collections.Generic.IEnumerable<int> EnumerateRunes(string value)
    {
        for (var i = 0; i < value.Length; i++)
        {
            var code = value[i];
            if (char.IsHighSurrogate(code) && i + 1 < value.Length && char.IsLowSurrogate(value[i + 1]))
            {
                yield return char.ConvertToUtf32(code, value[++i]);
            }
            else
            {
                yield return code;
            }
        }
    }
}
