using System;
using System.Threading.Tasks;
using PezMax.Core.Api;
using PezMax.Core.Models;
using PezMax.Core.Net;
using PezMax.Core.Validation;
using Windows.System;
using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Input;

namespace PezMax.Login
{
    /// <summary>
    /// 注册三步（账号 → 密保 → 验证码）。校验文案与判定顺序全部来自
    /// <see cref="FormValidation"/>（对 `spec/fixtures/auth/validation.json` 负责），页面只管布局与步骤。
    /// 注册成功**不自动登录**（`spec/api/auth.md` §3），交回登录页并带上用户名。
    /// </summary>
    public sealed partial class RegisterView : UserControl
    {
        private CaptchaResult? _captcha;
        private bool _busy;

        public RegisterView()
        {
            InitializeComponent();
        }

        /// <summary>本层关闭；参数为注册成功的用户名，取消时为 null。</summary>
        public event EventHandler<string?>? Closed;

        /// <summary>每次打开都从第一步开始，且不保留上一次的输入。</summary>
        public void Begin()
        {
            _captcha = null;
            _busy = false;
            UserNameBox.Text = "";
            PasswordBox.Password = "";
            ConfirmBox.Password = "";
            QuestionOneBox.Text = "";
            QuestionTwoBox.Text = "";
            QuestionThreeBox.Text = "";
            AnswerOneBox.Text = "";
            AnswerTwoBox.Text = "";
            AnswerThreeBox.Text = "";
            CodeBox.Text = "";
            ApplyStep(1);
            UserNameBox.Focus(FocusState.Programmatic);
        }

        private void ApplyStep(int step)
        {
            AccountStep.Visibility = step == 1 ? Visibility.Visible : Visibility.Collapsed;
            SecurityStep.Visibility = step == 2 ? Visibility.Visible : Visibility.Collapsed;
            CaptchaStep.Visibility = step == 3 ? Visibility.Visible : Visibility.Collapsed;
            DoneStep.Visibility = step == 4 ? Visibility.Visible : Visibility.Collapsed;

            AccountError.IsOpen = false;
            SecurityError.IsOpen = false;
            CaptchaError.IsOpen = false;

            StepCaption.Text = step switch
            {
                1 => "第 1 步，共 3 步 · 账号与密码",
                2 => "第 2 步，共 3 步 · 密保问题",
                3 => "第 3 步，共 3 步 · 验证码",
                _ => "已完成",
            };
        }

        private RegisterRequest Collect()
        {
            var request = new RegisterRequest
            {
                // 上游先 TrimSpace 再判长度与查重（datum_user_auth.go:591），这里提交前就裁掉。
                UserName = (UserNameBox.Text ?? "").Trim(),
                Password = PasswordBox.Password,
                ConfirmPassword = ConfirmBox.Password,
                SecurityQuestionOne = (QuestionOneBox.Text ?? "").Trim(),
                SecurityQuestionTwo = (QuestionTwoBox.Text ?? "").Trim(),
                SecurityQuestionThree = (QuestionThreeBox.Text ?? "").Trim(),
                SecurityAnswerOne = (AnswerOneBox.Text ?? "").Trim(),
                SecurityAnswerTwo = (AnswerTwoBox.Text ?? "").Trim(),
                SecurityAnswerThree = (AnswerThreeBox.Text ?? "").Trim(),
                Code = (CodeBox.Text ?? "").Trim(),
                Uuid = _captcha?.Uuid ?? "",
            };
            return request;
        }

        private void StepKeyDown(object sender, KeyRoutedEventArgs e)
        {
            if (e.Key != VirtualKey.Enter || _busy) return;
            e.Handled = true;
            if (AccountStep.Visibility == Visibility.Visible)
            {
                AccountNextClick(sender, new RoutedEventArgs());
            }
            else if (CaptchaStep.Visibility == Visibility.Visible)
            {
                SubmitClick(sender, new RoutedEventArgs());
            }
        }

        private void AccountNextClick(object sender, RoutedEventArgs e)
        {
            var error = FormValidation.ValidateAccount(
                UserNameBox.Text, PasswordBox.Password, ConfirmBox.Password);
            if (error != null)
            {
                ShowError(AccountError, error);
                return;
            }

            ApplyStep(2);
            QuestionOneBox.Focus(FocusState.Programmatic);
        }

        private void SecurityBackClick(object sender, RoutedEventArgs e) => ApplyStep(1);

        private async void SecurityNextClick(object sender, RoutedEventArgs e)
        {
            var error = FormValidation.ValidateSecurity(
                QuestionOneBox.Text, QuestionTwoBox.Text, QuestionThreeBox.Text,
                AnswerOneBox.Text, AnswerTwoBox.Text, AnswerThreeBox.Text);
            if (error != null)
            {
                ShowError(SecurityError, error);
                return;
            }

            ApplyStep(3);
            await LoadCaptchaAsync();
            CodeBox.Focus(FocusState.Programmatic);
        }

        private void CaptchaBackClick(object sender, RoutedEventArgs e) => ApplyStep(2);

        private void CancelClick(object sender, RoutedEventArgs e) => Closed?.Invoke(this, null);

        private void DoneClick(object sender, RoutedEventArgs e) => Closed?.Invoke(this, Collect().UserName);

        private async void CaptchaTapped(object sender, TappedRoutedEventArgs e)
        {
            if (_busy) return;
            await LoadCaptchaAsync();
        }

        private async Task LoadCaptchaAsync()
        {
            SetBusy(true);
            try
            {
                _captcha = await AppServices.Auth.GetCaptchaAsync();
                CodeBox.Text = "";
                await CaptchaPresenter.ShowAsync(CaptchaImage, _captcha.ImageBase64);
            }
            catch (DatumException ex)
            {
                _captcha = null;
                CaptchaImage.Source = null;
                ShowError(CaptchaError, ex.Message);
            }
            catch (Exception)
            {
                _captcha = null;
                CaptchaImage.Source = null;
                ShowError(CaptchaError, "验证码获取失败，请稍后重试");
            }
            finally
            {
                SetBusy(false);
            }
        }

        private async void SubmitClick(object sender, RoutedEventArgs e)
        {
            if (_busy) return;

            var request = Collect();
            var error = FormValidation.ValidateRegister(request);
            if (error != null)
            {
                ShowError(CaptchaError, error);
                return;
            }

            SetBusy(true);
            try
            {
                var userId = await AppServices.Auth.RegisterAsync(request);
                DoneHint.Text = $"账号 {request.UserName} 已创建（编号 {userId}）。";
                ApplyStep(4);
            }
            catch (DatumException ex)
            {
                // 用户名被占用是第一步的问题：退回去改名字，比让用户在验证码页反复试更省事。
                if (ex.HttpStatus == 409)
                {
                    ApplyStep(1);
                    ShowError(AccountError, ex.Message);
                }
                else
                {
                    ShowError(CaptchaError, ex.Message);
                }

                // 上游每次提交都会消耗验证码，失败也一样（datum_user_auth.go:615）；必须换图。
                await LoadCaptchaAsync();
            }
            catch (Exception)
            {
                ShowError(CaptchaError, "网络异常，请检查后端地址与网络连接");
                await LoadCaptchaAsync();
            }
            finally
            {
                SetBusy(false);
            }
        }

        private void SetBusy(bool busy)
        {
            _busy = busy;
            SubmitButton.IsEnabled = !busy;
            SubmitProgress.IsActive = busy;
            SubmitProgress.Visibility = busy ? Visibility.Visible : Visibility.Collapsed;
            CodeBox.IsEnabled = !busy;
        }

        private static void ShowError(Microsoft.UI.Xaml.Controls.InfoBar bar, string message)
        {
            bar.Message = message;
            bar.IsOpen = true;
        }
    }
}
