// API 客户端核心
// 封装 reqwest，提供统一的鉴权头、超时、错误处理

use anyhow::Result;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::Mutex;

pub static BASE_URL: &str = "http://154.8.139.48:8080";
pub static TOKEN: AtomicBool = AtomicBool::new(false);

#[derive(Clone)]
pub struct ApiClient {
    pub client: reqwest::Client,
    pub base_url: String,
    pub token: Arc<Mutex<Option<String>>>,
}

impl ApiClient {
    /// 创建新的 API 客户端
    pub fn new(base_url: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: base_url.unwrap_or_else(|| BASE_URL.to_string()),
            token: Arc::new(Mutex::new(None)),
        }
    }

    /// 设置 Bearer token
    pub async fn set_token(&self, token: String) {
        let mut t = self.token.lock().await;
        *t = Some(token);
    }

    /// 清除 token（登出）
    pub async fn clear_token(&self) {
        let mut t = self.token.lock().await;
        *t = None;
    }

    /// 获取当前 token
    pub async fn get_token(&self) -> Option<String> {
        let t = self.token.lock().await;
        t.clone()
    }

    /// 构建带默认鉴权的请求头
    fn headers(&self, token: Option<&str>) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json;charset=utf-8".parse().unwrap(),
        );
        if let Some(t) = token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", t).parse().unwrap(),
            );
        }
        headers
    }

    /// GET 请求
    pub async fn get<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        params: Option<Vec<(&str, String)>>,
    ) -> Result<T> {
        let token = self.get_token().await;
        let mut req = self
            .client
            .get(format!("{}{}", self.base_url, path))
            .headers(self.headers(token.as_deref()));

        if let Some(p) = params {
            req = req.query(&p);
        }

        let resp = req.send().await?;
        let text = resp.text().await?;
        Ok(serde_json::from_str(&text)?)
    }

    /// POST 请求
    pub async fn post<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> Result<T> {
        let token = self.get_token().await;
        let resp = self
            .client
            .post(format!("{}{}", self.base_url, path))
            .headers(self.headers(token.as_deref()))
            .json(body)
            .send()
            .await?;
        let text = resp.text().await?;
        Ok(serde_json::from_str(&text)?)
    }

    /// PUT 请求
    pub async fn put<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> Result<T> {
        let token = self.get_token().await;
        let resp = self
            .client
            .put(format!("{}{}", self.base_url, path))
            .headers(self.headers(token.as_deref()))
            .json(body)
            .send()
            .await?;
        let text = resp.text().await?;
        Ok(serde_json::from_str(&text)?)
    }

    /// DELETE 请求
    pub async fn delete<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T> {
        let token = self.get_token().await;
        let resp = self
            .client
            .delete(format!("{}{}", self.base_url, path))
            .headers(self.headers(token.as_deref()))
            .send()
            .await?;
        let text = resp.text().await?;
        Ok(serde_json::from_str(&text)?)
    }

    /// GET 请求（匿名，不带 token）
    pub async fn get_anonymous<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        params: Option<Vec<(&str, String)>>,
    ) -> Result<T> {
        let mut req = self
            .client
            .get(format!("{}{}", self.base_url, path))
            .headers(self.headers(None));

        if let Some(p) = params {
            req = req.query(&p);
        }

        let resp = req.send().await?;
        let text = resp.text().await?;
        Ok(serde_json::from_str(&text)?)
    }

    /// POST 请求（匿名，不带 token）
    pub async fn post_anonymous<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> Result<T> {
        let resp = self
            .client
            .post(format!("{}{}", self.base_url, path))
            .headers(self.headers(None))
            .json(body)
            .send()
            .await?;
        let text = resp.text().await?;
        Ok(serde_json::from_str(&text)?)
    }

    /// 文件上传（multipart）
    pub async fn upload_file<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        file_path: &str,
        field_name: &str,
        extra_fields: Option<Vec<(&str, String)>>,
    ) -> Result<T> {
        let token = self.get_token().await;
        let file_bytes = tokio::fs::read(file_path).await?;
        let file_name = std::path::Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
            .to_string();

        let mut form = reqwest::multipart::Form::new()
            .part(
                field_name.to_string(),
                reqwest::multipart::Part::bytes(file_bytes).file_name(file_name),
            );

        if let Some(fields) = extra_fields {
            for (k, v) in fields {
                form = form.text(k.to_string(), v);
            }
        }

        let resp = self
            .client
            .post(format!("{}{}", self.base_url, path))
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", token.unwrap_or_default()),
            )
            .multipart(form)
            .send()
            .await?;
        let text = resp.text().await?;
        Ok(serde_json::from_str(&text)?)
    }

    /// 文件下载（返回字节流）
    pub async fn download_file(&self, path: &str) -> Result<Vec<u8>> {
        let token = self.get_token().await;
        let resp = self
            .client
            .get(format!("{}{}", self.base_url, path))
            .headers(self.headers(token.as_deref()))
            .send()
            .await?;
        Ok(resp.bytes().await?.to_vec())
    }

    /// 从任意 URL 下载原始字节流（支持绝对 URL 和相对路径，带 Bearer token）
    pub async fn download_raw_url(&self, url: &str) -> Result<Vec<u8>> {
        if url.is_empty() {
            anyhow::bail!("empty url");
        }
        let token = self.get_token().await;
        let full_url = if url.starts_with("http://") || url.starts_with("https://") {
            url.to_string()
        } else {
            format!("{}{}", self.base_url, url)
        };
        let resp = self
            .client
            .get(&full_url)
            .headers(self.headers(token.as_deref()))
            .send()
            .await?;
        Ok(resp.bytes().await?.to_vec())
    }
}