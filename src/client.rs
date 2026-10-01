//! B 站 HTTP 客户端
use anyhow::{Context, Result};
use reqwest::{Client, Response};

use crate::config;

#[derive(Clone)]
pub struct BiliClient {
    http: Client,
}

impl BiliClient {
    pub fn new() -> Result<Self> {
        let http = Client::builder()
            .user_agent(config::USER_AGENT)
            .build()
            .context("创建 HTTP 客户端失败")?;
        Ok(Self { http })
    }

    /// 暴露底层 Client，让 API 层自己精确控制请求头
    pub fn http(&self) -> &Client {
        &self.http
    }

    /// 下载音视频流：严格还原最初版本 —— 只带 Referer + Cookie
    pub async fn get_stream(&self, url: &str, referer: &str) -> Result<Response> {
        self.http
            .get(url)
            .header("Referer", referer)
            .header("Cookie", config::COOKIE)
            .send()
            .await
            .with_context(|| format!("下载请求失败: {}", url))?
            .error_for_status()
            .with_context(|| format!("HTTP 错误: {}", url))
    }
}