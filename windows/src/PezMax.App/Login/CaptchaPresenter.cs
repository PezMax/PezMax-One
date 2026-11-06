using System;
using System.Threading.Tasks;
using Windows.Storage.Streams;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Media.Imaging;

namespace PezMax.Login
{
    /// <summary>
    /// 验证码图片解码：上游给的是**不带 `data:` 前缀**的 base64 JPEG（`spec/api/auth.md` §1），
    /// 登录、注册、找回密码三处共用这一份解码。
    /// </summary>
    internal static class CaptchaPresenter
    {
        public static async Task ShowAsync(Image target, string base64)
        {
            if (string.IsNullOrWhiteSpace(base64))
            {
                target.Source = null;
                return;
            }

            var bytes = Convert.FromBase64String(base64);
            var stream = new InMemoryRandomAccessStream();
            using (var writer = new DataWriter(stream))
            {
                writer.WriteBytes(bytes);
                await writer.StoreAsync();
                await writer.FlushAsync();
                writer.DetachStream();
            }

            stream.Seek(0);
            var bitmap = new BitmapImage();
            await bitmap.SetSourceAsync(stream);
            target.Source = bitmap;
        }
    }
}
