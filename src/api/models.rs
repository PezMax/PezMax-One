// 通用 API 响应模型
// 对应后端 AjaxResult / TableDataInfo 统一格式
// 后端返回 camelCase JSON，所有数据模型使用 rename_all = "camelCase"

use serde::{Deserialize, Deserializer, Serialize};

/// null JSON 值 → 对应类型的 Default（如 String → ""，i64 → 0）
fn null_to_default<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Option::<T>::deserialize(d).map(|v| v.unwrap_or_default())
}

/// 后端统一响应格式 (AjaxResult)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
pub struct ApiResponse<T> {
    pub code: i32,
    pub msg: String,
    #[serde(default)]
    pub data: Option<T>,
}

/// 分页响应格式 (TableDataInfo)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
pub struct PageResponse<T> {
    pub code: i32,
    pub msg: String,
    #[serde(default)]
    pub rows: Vec<T>,
    #[serde(default)]
    pub total: i64,
}

/// 分页请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageParams {
    #[serde(default = "default_page_num")]
    pub page_num: i32,
    #[serde(default = "default_page_size")]
    pub page_size: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by_column: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_asc: Option<String>,
}

fn default_page_num() -> i32 { 1 }
fn default_page_size() -> i32 { 20 }

impl Default for PageParams {
    fn default() -> Self {
        Self {
            page_num: 1,
            page_size: 20,
            order_by_column: None,
            is_asc: None,
        }
    }
}

/// 登录请求体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
}

/// 登录响应 (桌面端)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    #[serde(default)]
    pub user_id: i64,
    #[serde(default)]
    pub user_name: String,
    #[serde(default)]
    pub nick_name: String,
    #[serde(default)]
    pub avatar: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub phonenumber: String,
    #[serde(default)]
    pub sex: String,
    #[serde(default)]
    pub status: String,
}

/// getInfo 响应结构（用户信息嵌套在 data.user 中）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfoResponse {
    pub user: UserInfo,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub is_default_modify_pwd: bool,
    #[serde(default)]
    pub is_password_expired: bool,
}

/// 验证码响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptchaResponse {
    #[serde(default = "default_true")]
    pub captcha_enabled: bool,
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub img: String, // base64
}

fn default_true() -> bool { true }

/// 注册请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub nickname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    pub security_questions: Vec<SecurityQuestion>,
}

/// 密保问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityQuestion {
    pub question: String,
    pub answer: String,
}

/// 试卷文件
/// 后端返回: fileId, fileName, fileFormat, fileSize, fileUrl, fileSubject,
///            fileYear(整数), fileSchool, createBy, createTime, fileCover
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaperFile {
    #[serde(default, deserialize_with = "null_to_default")]
    pub file_id: i64,
    #[serde(default, deserialize_with = "null_to_default")]
    pub file_name: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub file_format: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub file_size: i64,
    #[serde(default, deserialize_with = "null_to_default")]
    pub file_url: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub file_subject: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub file_year: i64,
    #[serde(rename = "fileSchool", default, deserialize_with = "null_to_default")]
    pub school_name: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub create_by: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub create_time: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub file_cover: String,
    /// 0=未审核, 1=通过, 2=未通过, 3=被举报；null 视为可见
    #[serde(default)]
    pub file_status: Option<i64>,
}

/// 文件树节点
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileTreeNode {
    pub id: i64,
    pub label: String,
    #[serde(default)]
    pub children: Vec<FileTreeNode>,
    #[serde(rename = "type", default)]
    pub node_type: String,
}

/// 书签
/// 后端返回: id, title, description, resourceType, coverImage, createBy, createTime, collection
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
    #[serde(default, deserialize_with = "null_to_default")]
    pub id: i64,
    #[serde(default, deserialize_with = "null_to_default")]
    pub title: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub description: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub resource_type: String,
    #[serde(rename = "coverImage", default, deserialize_with = "null_to_default")]
    pub cover_url: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub create_by: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub create_time: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub collection: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub status: i64,
}

/// 通知
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    #[serde(default, deserialize_with = "null_to_default")]
    pub notify_id: i64,
    #[serde(default, deserialize_with = "null_to_default")]
    pub title: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub content: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub notify_type: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub status: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub create_time: String,
}

/// 下载记录
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadRecord {
    #[serde(default, deserialize_with = "null_to_default")]
    pub download_id: i64,
    #[serde(default, deserialize_with = "null_to_default")]
    pub file_id: i64,
    #[serde(default, deserialize_with = "null_to_default")]
    pub file_name: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub user_id: i64,
    #[serde(default, deserialize_with = "null_to_default")]
    pub download_time: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub file_format: String,
}

/// 收藏记录
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteRecord {
    #[serde(default, deserialize_with = "null_to_default")]
    pub file_id: i64,
    #[serde(default, deserialize_with = "null_to_default")]
    pub file_name: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub file_subject: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub create_time: String,
}

/// 用户统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStats {
    #[serde(default, deserialize_with = "null_to_default")]
    pub favorite_count: i64,
    #[serde(default, deserialize_with = "null_to_default")]
    pub download_count: i64,
    #[serde(default, deserialize_with = "null_to_default")]
    pub upload_count: i64,
}

/// 举报
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    #[serde(default, deserialize_with = "null_to_default")]
    pub report_id: i64,
    #[serde(default, deserialize_with = "null_to_default")]
    pub report_type: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub content: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub status: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub create_time: String,
}

/// 排行榜用户项
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRankItem {
    #[serde(default, deserialize_with = "null_to_default")]
    pub user_id: i64,
    #[serde(default, deserialize_with = "null_to_default")]
    pub user_name: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub nick_name: String,
    #[serde(default, deserialize_with = "null_to_default")]
    #[serde(alias = "count")]
    pub upload_count: i64,
    #[serde(default, deserialize_with = "null_to_default")]
    pub download_count: i64,
    #[serde(default, deserialize_with = "null_to_default")]
    pub avatar: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub years: i64,
}

impl UserRankItem {
    pub fn display_name(&self) -> &str {
        if !self.nick_name.is_empty() { &self.nick_name } else { &self.user_name }
    }
}

/// 收藏/下载/举报的通用操作响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResponse {
    pub code: i32,
    pub msg: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}