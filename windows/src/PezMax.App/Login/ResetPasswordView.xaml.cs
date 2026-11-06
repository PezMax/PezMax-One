using System;
using System.Collections.Generic;
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
    /// 找回密码两步：验证码换密保问题（服务端同时签发 10 分钟重置工单）→ 回答 + 新密码。
    /// 第二步的 `code` 只做非空校验，真正凭证是工单，所以这里原样回传第一步的 `uuid` 与 `code`；
    /// 工单一次性，第二步失败必须回第一步换新验证码（`spec/api/auth.md` §4）。
    /// </summary>
    public sealed partial class ResetPasswordView : UserControl
    {
        private readonly List<TextBox> _answerBoxes = new();
        private CaptchaResult? _captcha;
        private string _userName = "";
        private bool _busy;

        public ResetPasswordView()
        {
            InitializeComponent();
        }

        /// <summary>本层关闭；参数为重置成功的用户名，取消时为 null。</summary>
        public event EventHandler<string?>? Closed;

        public void Begin()
        {
            _captcha = null;
            _userName = "";
            _busy = false;
            UserNameBox.Text = "";
            CodeBox.Text = "";
            NewPasswordBox.Password = "";
            ConfirmBox.Password = "";
            AnswersPanel.Children.Clear();
            _answerBoxes.Clear();
            ApplyStep(1);
            UserNameBox.Focus(FocusState.Programmatic);
        }

        /// <summary>第一步的验证码由宿主页在打开本层时索取（与登录页共用同一个入口）。</summary>
        public Task ShowCaptchaAsync() => LoadCaptchaAsync();

        private void ApplyStep(int step)        {
            VerifyStep.Visibility = step == 1 ? Visibility.Visible : Visibility.Collapsed;
            AnswerStep.Visibility = step == 2 ? Visibility.Visible : Visibility.Collapsed;
            DoneStep.Visibility = step == 3 ? Visibility.Visible : Visibility.Collapsed;

            VerifyError.IsOpen = false;
            AnswerError.IsOpen = false;

            StepCaption.Text = step switch
            {
                1 => "第 1 步，共 2 步 · 用户名与验证码",
                2 => "第 2 步，共 2 步 · 密保答案与新密码",
                _ => "已完成",
            };
        }

        private static void ShowError(Microsoft.UI.Xaml.Controls.InfoBar bar, string message)
        {
            bar.Message = message;
            bar.IsOpen = true;
        }

        private void CancelClick(object sender, RoutedEventArgs e) => Closed?.Invoke(this, null);

        private void DoneClick(object sender, RoutedEventArgs e) => Closed?.Invoke(this, _userName);

        private void VerifyKeyDown(object sender, KeyRoutedEventArgs e)
        {
            if (e.Key != VirtualKey.Enter || _busy) return;
            e.Handled = true;
            VerifyClick(sender, new RoutedEventArgs());
        }

        private void AnswerKeyDown(object sender, KeyRoutedEventArgs e)
        {
            if (e.Key != VirtualKey.Enter || _busy) return;
            e.Handled = true;
            ResetClick(sender, new RoutedEventArgs());
        }

        private async void CaptchaTapped(object sender, TappedRoutedEventArgs e)
        {
            if (_busy) return;
            await LoadCaptchaAsync();
        }

        private async Task LoadCaptchaAsync()
        {
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
                ShowError(VerifyError, ex.Message);
            }
            catch (Exception)
            {
                _captcha = null;
                CaptchaImage.Source = null;
                ShowError(VerifyError, "验证码获取失败，请稍后重试");
            }
        }

        private async void VerifyClick(object sender, RoutedEventArgs e)
        {
            if (_busy) return;

            var userName = (UserNameBox.Text ?? "").Trim();
            var code = (CodeBox.Text ?? "").Trim();
            var uuid = _captcha?.Uuid ?? "";

            var error = FormValidation.ValidateUserName(userName) ??
                        FormValidation.ValidateCaptchaEntry(code, uuid);
            if (error != null)
            {
                ShowError(VerifyError, error);
                return;
            }

            SetBusy(true);
            IReadOnlyList<string> questions;
            try
            {
                questions = await AppServices.Auth.GetSecurityQuestionsAsync(userName, uuid, code);
            }
            catch (DatumException ex)
            {
                ShowError(VerifyError, ex.Message);
                await LoadCaptchaAsync();
                return;
            }
            catch (Exception)
            {
                ShowError(VerifyError, "网络异常，请检查后端地址与网络连接");
                await LoadCaptchaAsync();
                return;
            }
            finally
            {
                SetBusy(false);
            }

            _userName = userName;
            if (questions.Count == 0)
            {
                ShowError(VerifyError, "该账号没有设置密保问题，无法通过密保找回密码。");
                await LoadCaptchaAsync();
                return;
            }

            BuildAnswerBoxes(questions);
            ApplyStep(2);
            _answerBoxes[0].Focus(FocusState.Programmatic);
        }

        private void BuildAnswerBoxes(IReadOnlyList<string> questions)
        {
            AnswersPanel.Children.Clear();
            _answerBoxes.Clear();

            foreach (var question in questions)
            {
                var box = new TextBox
                {
                    Header = question,
                    PlaceholderText = "答案",
                    TextWrapping = TextWrapping.Wrap,
                };
                _answerBoxes.Add(box);
                AnswersPanel.Children.Add(box);
            }
        }

        private void AnswerBackClick(object sender, RoutedEventArgs e) => ApplyStep(1);

        private async void ResetClick(object sender, RoutedEventArgs e)
        {
            if (_busy) return;

            var request = new ResetPasswordRequest
            {
                UserName = _userName,
                Code = (CodeBox.Text ?? "").Trim(),
                Uuid = _captcha?.Uuid ?? "",
                SecurityAnswerOne = _answerBoxes.Count > 0 ? (_answerBoxes[0].Text ?? "").Trim() : "",
                SecurityAnswerTwo = _answerBoxes.Count > 1 ? (_answerBoxes[1].Text ?? "").Trim() : "",
                SecurityAnswerThree = _answerBoxes.Count > 2 ? (_answerBoxes[2].Text ?? "").Trim() : "",
                NewPassword = NewPasswordBox.Password,
                ConfirmPassword = ConfirmBox.Password,
            };

            var error = FormValidation.ValidateResetPassword(request);
            if (error != null)
            {
                ShowError(AnswerError, error);
                return;
            }

            SetBusy(true);
            try
            {
                await AppServices.Auth.ResetPasswordBySecurityAsync(request);
                ApplyStep(3);
            }
            catch (DatumException ex)
            {
                // 工单已被这次失败的提交消耗掉，只能回第一步重来（datum_user_auth.go:733 GETDEL）。
                ApplyStep(1);
                ShowError(VerifyError, ex.Message + "（请重新获取验证码后再试）");
                await LoadCaptchaAsync();
            }
            catch (Exception)
            {
                ApplyStep(1);
                ShowError(VerifyError, "网络异常，请检查后端地址与网络连接");
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
            VerifyButton.IsEnabled = !busy;
            ResetButton.IsEnabled = !busy;
            VerifyProgress.IsActive = busy;
            VerifyProgress.Visibility = busy ? Visibility.Visible : Visibility.Collapsed;
            ResetProgress.IsActive = busy;
            ResetProgress.Visibility = busy ? Visibility.Visible : Visibility.Collapsed;
        }
    }
}
