// 认证相关 API
// 桌面端登录/注册/验证码/找回密码

use crate::api::client::ApiClient;
use crate::api::models::*;
use anyhow::Result;

impl ApiClient {
    /// 桌面端登录（支持验证码）
    pub async fn desktop_login(
        &self,
        username: &str,
        password: &str,
        code: Option<String>,
        uuid: Option<String>,
    ) -> Result<ApiResponse<LoginResponse>> {
        let body = LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
            code,
            uuid,
        };
        self.post_anonymous("/datum/user/login", &body).await
    }

    /// 桌面端获取验证码
    pub async fn get_captcha(&self) -> Result<ApiResponse<CaptchaResponse>> {
        self.get_anonymous("/datum/user/captchaImage", None).await
    }

    /// 桌面端注册（带3个密保问题）
    pub async fn desktop_register(&self, req: &RegisterRequest) -> Result<ApiResponse<serde_json::Value>> {
        self.post_anonymous("/datum/user/register", req).await
    }

    /// 桌面端获取当前用户信息（含 roles/permissions/封禁状态）
    pub async fn get_user_info(&self) -> Result<ApiResponse<UserInfoResponse>> {
        self.get("/datum/user/getInfo", None).await
    }

    /// 获取用户密保问题（找回密码用，匿名）
    pub async fn get_security_questions(&self, username: &str) -> Result<ApiResponse<Vec<SecurityQuestion>>> {
        let params = vec![("userName", username.to_string())];
        self.get_anonymous("/datum/user/securityQuestions", Some(params)).await
    }

    /// 通过密保找回密码（匿名）
    pub async fn reset_password_by_security(&self, data: &serde_json::Value) -> Result<ApiResponse<serde_json::Value>> {
        self.post_anonymous("/datum/user/resetPasswordBySecurity", data).await
    }

    /// 桌面端登出
    pub async fn logout(&self) -> Result<ApiResponse<serde_json::Value>> {
        self.post("/logout", &serde_json::json!({})).await
    }
}