using System;
using System.Threading.Tasks;
using PezMax.Core.Models;
using PezMax.Core.Net;
using PezMax.Core.Validation;
using PezMax.Shell;
using Windows.Storage.Streams;
using Windows.System;
using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Input;
using Windows.UI.Xaml.Media.Imaging;
using Windows.UI.Xaml.Navigation;

namespace PezMax.Login
{
    public sealed partial class LoginPage : Page
    {
        private CaptchaResult? _captcha;
        private bool _busy;
        private bool _dialogOpen;

        public LoginPage()
        {
            InitializeComponent();
        }

        protected override async void OnNavigatedTo(NavigationEventArgs e)
        {
            base.OnNavigatedTo(e);
            var version = Windows.ApplicationModel.Package.Current.Id.Version;
            VersionText.Text = $"版本 {version.Major}.{version.Minor}.{version.Build}";
            UpdateServerLink();
            ApplyTitleBar();
            UsernameBox.Focus(FocusState.Programmatic);
            await LoadCaptchaAsync();
        }

        private void ApplyTitleBar()
        {
            try
            {
                var titleBar = Windows.UI.ViewManagement.ApplicationView.GetForCurrentView().TitleBar;
                titleBar.BackgroundColor = Windows.UI.Color.FromArgb(255, 0x10, 0x14, 0x18);
                titleBar.ForegroundColor = Windows.UI.Colors.White;
                titleBar.InactiveBackgroundColor = Windows.UI.Color.FromArgb(255, 0x10, 0x14, 0x18);
                titleBar.InactiveForegroundColor = Windows.UI.Color.FromArgb(255, 0xA7, 0xB0, 0xBC);
                titleBar.ButtonBackgroundColor = Windows.UI.Colors.Transparent;
                titleBar.ButtonForegroundColor = Windows.UI.Colors.White;
                titleBar.ButtonInactiveBackgroundColor = Windows.UI.Colors.Transparent;
                titleBar.ButtonInactiveForegroundColor = Windows.UI.Color.FromArgb(255, 0xA7, 0xB0, 0xBC);
            }
            catch (Exception ex)
            {
                App.WriteCrashLog(ex);
            }
        }

        private void UpdateServerLink()
        {
            ServerLink.Content = "服务器：" + AppServices.Settings.ServerUrl;
        }

        private async void ServerClick(object sender, RoutedEventArgs e)
        {
            if (_dialogOpen) return;
            _dialogOpen = true;
            try
            {
                await ShowServerDialogAsync();
            }
            finally
            {
                _dialogOpen = false;
            }
        }

        private async Task ShowServerDialogAsync()
        {
            var box = new TextBox
            {
                Header = "服务器地址",
                Text = AppServices.Settings.ServerUrl,
                PlaceholderText = "https://example.com 或 http://10.0.0.2:8080",
                CornerRadius = new CornerRadius(0),
            };
            var dialog = new PezDialog
            {
                Title = "服务器地址",
                Content = box,
                PrimaryButtonText = "保存并刷新验证码",
                CloseButtonText = "取消",
                DefaultButton = ContentDialogButton.Primary,
            };
            if (await dialog.ShowAsync() != ContentDialogResult.Primary) return;

            var server = (box.Text ?? "").Trim();
            if (!Uri.TryCreate(server, UriKind.Absolute, out var uri) ||
                (uri.Scheme != Uri.UriSchemeHttp && uri.Scheme != Uri.UriSchemeHttps))
            {
                ShowError("服务器地址必须是 http:// 或 https:// 开头的完整地址");
                return;
            }

            AppServices.Settings.ServerUrl = PezMax.Core.Net.DatumHttp.NormalizeBaseUrl(server);
            AppServices.Http.BaseUrl = AppServices.Settings.ServerUrl;
            await AppServices.SaveSettingsAsync();
            UpdateServerLink();
            await LoadCaptchaAsync();
        }

        private async Task LoadCaptchaAsync()
        {
            try
            {
                SetBusy(true);
                _captcha = await AppServices.Auth.GetCaptchaAsync();
                CodeBox.Text = "";
                await ShowCaptchaAsync(_captcha.ImageBase64);
            }
            catch (Exception ex)
            {
                ShowError(ex is DatumException datum ? datum.Message : "验证码获取失败，请稍后重试");
            }
            finally
            {
                SetBusy(false);
            }
        }

        private async Task ShowCaptchaAsync(string base64)
        {
            if (string.IsNullOrWhiteSpace(base64))
            {
                CaptchaImage.Source = null;
                return;
            }

            var bytes = Convert.FromBase64String(base64);
            using var stream = new InMemoryRandomAccessStream();
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
            CaptchaImage.Source = bitmap;
        }

        private async void LoginClick(object sender, RoutedEventArgs e) => await LoginAsync();

        private async void CaptchaTapped(object sender, TappedRoutedEventArgs e) => await LoadCaptchaAsync();

        private async void InputKeyDown(object sender, KeyRoutedEventArgs e)
        {
            if (e.Key == VirtualKey.Enter) await LoginAsync();
        }

        private async Task LoginAsync()
        {
            if (_busy) return;

            var validation = FormValidation.ValidateLogin(UsernameBox.Text, PasswordBox.Password);
            if (validation != null)
            {
                ShowError(validation);
                return;
            }

            if (_captcha == null)
            {
                ShowError("验证码尚未加载，请点击验证码重试");
                return;
            }

            SetBusy(true);
            ErrorBar.IsOpen = false;
            try
            {
                var login = await AppServices.Auth.LoginAsync(
                    UsernameBox.Text.Trim(),
                    PasswordBox.Password,
                    CodeBox.Text.Trim(),
                    _captcha.Uuid);
                await AppServices.CompleteLoginAsync(login);

                Frame.Navigate(typeof(ShellPage));
                Frame.BackStack.Clear();
            }
            catch (DatumException ex)
            {
                ShowError(ex.Message);
                await LoadCaptchaAsync();
            }
            catch (Exception)
            {
                ShowError("网络异常，请检查后端地址与网络连接");
                await LoadCaptchaAsync();
            }
            finally
            {
                SetBusy(false);
            }
        }

        private void PreviewClick(object sender, RoutedEventArgs e)
        {
            AppServices.EnterPreviewMode();
            Frame.Navigate(typeof(ShellPage));
            Frame.BackStack.Clear();
        }

        private async void RegisterClick(object sender, RoutedEventArgs e)
        {
            if (_dialogOpen) return;
            _dialogOpen = true;
            try
            {
                await new PezDialog
                {
                    Title = "注册账号",
                    Content = "注册流程将在下一批接入；当前可先在官方客户端或网页完成注册。",
                    CloseButtonText = "知道了",
                    DefaultButton = ContentDialogButton.Close,
                }.ShowAsync();
            }
            finally
            {
                _dialogOpen = false;
            }
        }

        private async void ForgotClick(object sender, RoutedEventArgs e)
        {
            if (_dialogOpen) return;
            _dialogOpen = true;
            try
            {
                await new PezDialog
                {
                    Title = "找回密码",
                    Content = "两步密保找回将在下一批接入；当前可先在官方客户端或网页完成重置。",
                    CloseButtonText = "知道了",
                    DefaultButton = ContentDialogButton.Close,
                }.ShowAsync();
            }
            finally
            {
                _dialogOpen = false;
            }
        }

        private void SetBusy(bool busy)
        {
            _busy = busy;
            LoginButton.IsEnabled = !busy;
            LoginProgress.IsActive = busy;
            LoginProgress.Visibility = busy ? Visibility.Visible : Visibility.Collapsed;
            UsernameBox.IsEnabled = !busy;
            PasswordBox.IsEnabled = !busy;
            CodeBox.IsEnabled = !busy;
        }

        private void ShowError(string message)
        {
            ErrorBar.Message = message;
            ErrorBar.IsOpen = true;
        }
    }
}
