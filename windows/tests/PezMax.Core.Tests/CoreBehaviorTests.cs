using PezMax.Core.Api;
using PezMax.Core.State;
using PezMax.Core.Urls;
using PezMax.Core.Validation;
using Xunit;

namespace PezMax.Core.Tests;

public class CoreBehaviorTests
{
    [Fact]
    public void RegisterValidation_MatchesServerRules()
    {
        var request = new RegisterRequest
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
            Uuid = "u",
        };
        Assert.Null(FormValidation.ValidateRegister(request));

        request.Password = "1234";
        Assert.Contains("5 到 64", FormValidation.ValidateRegister(request));
        request.Password = request.ConfirmPassword = "12345";

        request.ConfirmPassword = "54321";
        Assert.Equal("两次输入的密码不一致", FormValidation.ValidateRegister(request));
        request.ConfirmPassword = "12345";

        request.SecurityQuestionTwo = " ";
        Assert.Equal("密保问题不能为空", FormValidation.ValidateRegister(request));
    }

    [Fact]
    public void RegisterValidation_CountsUserNameByRuneAndPasswordByByte()
    {
        Assert.Null(FormValidation.ValidateUserName(new string('a', 64)));
        Assert.Equal("用户名格式错误", FormValidation.ValidateUserName(new string('a', 65)));
        Assert.Null(FormValidation.ValidateUserName(new string('中', 64)));
        Assert.Equal("用户名格式错误", FormValidation.ValidateUserName(new string('中', 65)));

        Assert.Null(FormValidation.ValidatePassword("12345"));
        Assert.Equal("密码格式错误：长度需在 5 到 64 位之间", FormValidation.ValidatePassword("1234"));
    }

    [Fact]
    public void UrlNormalizer_HandlesRelativeAndInternalHosts()
    {
        const string baseUrl = "https://api.example.com";

        Assert.Equal("https://api.example.com/upload/a.pdf", UrlNormalizer.Normalize("/upload/a.pdf", baseUrl));
        Assert.Equal("https://api.example.com/upload/a.pdf", UrlNormalizer.Normalize("upload/a.pdf", baseUrl));
        Assert.Equal("data:image/png;base64,abc", UrlNormalizer.Normalize("data:image/png;base64,abc", baseUrl));
        Assert.Equal("blob:https://x/y", UrlNormalizer.Normalize("blob:https://x/y", baseUrl));
        Assert.Equal("https://api.example.com/f/a.pdf?x=1",
            UrlNormalizer.Normalize("http://localhost:9000/f/a.pdf?x=1", baseUrl));
        Assert.Equal("https://api.example.com/bucket/a.pdf",
            UrlNormalizer.Normalize("minio://bucket/a.pdf", baseUrl));
        Assert.Equal("https://cdn.example.net/a.pdf",
            UrlNormalizer.Normalize("https://cdn.example.net/a.pdf", baseUrl));
    }

    [Fact]
    public void HashCache_KeepsLocalPayloadOnUnchanged()
    {
        var cache = new HashStateCache<string>();
        Assert.True(cache.Apply("h1", false, "first"));
        Assert.Equal("first", cache.Payload);

        Assert.False(cache.Apply("h1", true, null));
        Assert.Equal("first", cache.Payload);
        Assert.Equal("h1", cache.Hash);

        Assert.True(cache.Apply("h2", false, "second"));
        Assert.Equal("second", cache.Payload);
    }

    [Fact]
    public void HashCache_RefetchesWhenUnchangedWithoutLocalPayload()
    {
        var cache = new HashStateCache<string>();
        Assert.False(cache.Apply("h1", true, null));
        Assert.False(cache.HasPayload);
    }

    [Fact]
    public void BookmarkAndReportValidation_MatchServerLimits()
    {
        Assert.Null(FormValidation.ValidateBookmark("https://x", "标题"));
        Assert.Equal("网址不能为空且不超过 512 个字符", FormValidation.ValidateBookmark("", "标题"));
        Assert.Equal("标题不能为空且不超过 500 个字符", FormValidation.ValidateBookmark("https://x", ""));
        Assert.Null(FormValidation.ValidateReportReason("重复资料"));
        Assert.Contains("500", FormValidation.ValidateReportReason(new string('a', 501)));
    }
}
