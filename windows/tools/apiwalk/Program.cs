// apiwalk —— 用 PezMax.Core 自己的协议代码走一遍「注册 → 忘记密码 → 重置 → 登录 → 看内容」。
// 不碰 UWP：没有打包、没有回环豁免、不需要管理员。默认对着 spec/tools/devserver 跑。
//
//   dotnet run --project windows/tools/apiwalk -- http://127.0.0.1:8080

using PezMax.Core.Api;
using PezMax.Core.Net;

var baseUrl = args.Length > 0 ? args[0] : "http://127.0.0.1:8080";
var user = "小明" + DateTime.Now.Second;
var pass = "abc12345";
var next = "new12345";

var http = new DatumHttp(baseUrl);
var auth = new AuthApi(http);
var files = new FilesApi(http);

void Step(string title) => Console.WriteLine($"\n=== {title} ===");
void Ok(string text) => Console.WriteLine("  ✓ " + text);
void Bad(string text) => Console.WriteLine("  ✗ " + text);

// 断言失败即退出：这条链是用来核对契约的，绿了才算绿。
void Assert(bool condition, string what)
{
    if (condition)
    {
        Ok(what);
        return;
    }

    Bad(what);
    Environment.Exit(1);
}

try
{
    Step("1. 验证码");
    var captcha = await auth.GetCaptchaAsync();
    Ok($"captchaEnabled={captcha.CaptchaEnabled} uuid={captcha.Uuid} img={captcha.ImageBase64.Length} 字符 base64");
    var code = "0000";

    Step($"2. 注册 {user} / {pass}");
    var userId = await auth.RegisterAsync(new RegisterRequest
    {
        UserName = user,
        Password = pass,
        ConfirmPassword = pass,
        SecurityQuestionOne = "我的学号？",
        SecurityAnswerOne = "2025001",
        SecurityQuestionTwo = "我妈的姓？",
        SecurityAnswerTwo = "王",
        SecurityQuestionThree = "我的城市？",
        SecurityAnswerThree = "济南",
        Code = code,
        Uuid = captcha.Uuid,
    });
    // spec/api/auth.md：成功时 data = {"userId": 7}。读成 0 说明对端返回形状不对。
    Assert(userId > 0, $"userId = {userId}");

    Step("3. 旧密码能登录（建立基线）");
    var first = await auth.LoginAsync(user, pass, code, captcha.Uuid);
    http.Token = first.Token;
    var me = await auth.GetInfoAsync();
    Assert(me.User.UserId == userId, $"getInfo = {me.User.UserName} (id {me.User.UserId})，与注册返回的 id 一致");

    Step("4. 假装忘了密码：查密保问题");
    http.Token = null;
    var questions = await auth.GetSecurityQuestionsAsync(user, captcha.Uuid, code);
    Ok($"{questions.Count} 个问题：{string.Join(" / ", questions)}");

    Step("5. 答案填错，必须被拒");
    try
    {
        await auth.ResetPasswordBySecurityAsync(new ResetPasswordRequest
        {
            UserName = user, Code = code, Uuid = captcha.Uuid,
            SecurityAnswerOne = "错", SecurityAnswerTwo = "错", SecurityAnswerThree = "错",
            NewPassword = next, ConfirmPassword = next,
        });
        Bad("答案错了居然也重置成功 —— 服务端没校验");
    }
    catch (DatumException ex)
    {
        Ok($"被拒：{ex.Message}（HTTP {ex.HttpStatus}）");
    }

    Step("6. 答案填对，重置成 " + next);
    await auth.ResetPasswordBySecurityAsync(new ResetPasswordRequest
    {
        UserName = user, Code = code, Uuid = captcha.Uuid,
        SecurityAnswerOne = "2025001", SecurityAnswerTwo = "王", SecurityAnswerThree = "济南",
        NewPassword = next, ConfirmPassword = next,
    });
    Ok("重置成功");

    Step("7. 旧密码必须登不上");
    try
    {
        await auth.LoginAsync(user, pass, code, captcha.Uuid);
        Bad($"旧密码 {pass} 仍然能登录 —— 密码没真的换");
    }
    catch (DatumException ex)
    {
        Ok($"旧密码被拒：{ex.Message}");
    }

    Step("8. 新密码登录");
    var second = await auth.LoginAsync(user, next, code, captcha.Uuid);
    http.Token = second.Token;
    var me2 = await auth.GetInfoAsync();
    Ok($"getInfo = {me2.User.UserName} (id {me2.User.UserId})，expiresAt={second.ExpiresAt:u}");

    Step("9. 看内容");
    var page = await files.ListAsync(new FileQuery { PageNum = 1, PageSize = 5 });
    Ok($"公开资料 {page.Total} 份，首条 = {page.Rows.FirstOrDefault()?.DisplayName}");
    var tree = await files.GetTreeAsync();
    Ok($"树 {tree.Nodes.Count} 个学科：{string.Join("、", tree.Nodes.Select(n => n.Label))}");
    var mine = await files.ListAsync(new FileQuery { PageNum = 1, PageSize = 20, UserId = me2.User.UserId });
    Ok($"我的上传 {mine.Rows.Count} 份");

    Step("10. 下载一份 PDF（走 DownloadPlanner 那套流）");
    using var response = await http.GetStreamAsync("/datum/download/file",
        new[] { new KeyValuePair<string, string?>("fileId", "101") }, null, CancellationToken.None);
    var bytes = await response.Content.ReadAsByteArrayAsync();
    Ok($"{bytes.Length} 字节，头部 = {(bytes.Length > 4 ? System.Text.Encoding.ASCII.GetString(bytes, 0, 4) : "?")}，" +
       $"ETag = {response.Headers.ETag}");

    Console.WriteLine("\n全部走通。");
    return 0;
}
catch (Exception ex)
{
    Console.WriteLine($"\n中断：{ex.GetType().Name}: {ex.Message}");
    return 1;
}
