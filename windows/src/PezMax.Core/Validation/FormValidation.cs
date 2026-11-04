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

    public static string? ValidateRegister(RegisterRequest request, bool validateCaptcha = true)
    {
        var userName = ValidateUserName(request.UserName);
        if (userName != null) return userName;

        var password = ValidatePassword(request.Password);
        if (password != null) return password;

        if (request.Password != request.ConfirmPassword) return "两次输入的密码不一致";
        if (string.IsNullOrWhiteSpace(request.SecurityQuestionOne) ||
            string.IsNullOrWhiteSpace(request.SecurityQuestionTwo) ||
            string.IsNullOrWhiteSpace(request.SecurityQuestionThree))
            return "密保问题不能为空";
        if (string.IsNullOrWhiteSpace(request.SecurityAnswerOne) ||
            string.IsNullOrWhiteSpace(request.SecurityAnswerTwo) ||
            string.IsNullOrWhiteSpace(request.SecurityAnswerThree))
            return "密保答案不能为空";
        if (Encoding.UTF8.GetByteCount(request.Avatar ?? "") > 255) return "头像地址格式错误";
        if (validateCaptcha && (string.IsNullOrWhiteSpace(request.Code) || string.IsNullOrWhiteSpace(request.Uuid)))
            return "验证码错误或已过期";
        return null;
    }

    public static string? ValidateResetPassword(ResetPasswordRequest request)
    {
        if (string.IsNullOrWhiteSpace(request.UserName) ||
            string.IsNullOrWhiteSpace(request.Uuid) ||
            string.IsNullOrWhiteSpace(request.Code))
            return "验证码错误或已过期";
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
