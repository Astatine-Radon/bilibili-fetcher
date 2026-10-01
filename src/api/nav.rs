//! 获取 WBI 签名所需的 img_key / sub_key
use anyhow::{Context, Result};

use crate::client::BiliClient;
use crate::config;

const NAV_URL: &str = "https://api.bilibili.com/x/web-interface/nav";

pub struct WbiKeys {
    pub img_key: String,
    pub sub_key: String,
}

pub async fn fetch_wbi_keys(client: &BiliClient) -> Result<WbiKeys> {
    // 最初版本：只有 Cookie
    let resp: serde_json::Value = client
        .http()
        .get(NAV_URL)
        .header("Cookie", config::COOKIE)
        .send()
        .await?
        .json()
        .await?;

    let img_url = resp["data"]["wbi_img"]["img_url"]
        .as_str()
        .context("未找到 img_url，可能 Cookie 已失效")?;
    let sub_url = resp["data"]["wbi_img"]["sub_url"]
        .as_str()
        .context("未找到 sub_url")?;

    Ok(WbiKeys {
        img_key: extract_key(img_url).context("提取 img_key 失败")?,
        sub_key: extract_key(sub_url).context("提取 sub_key 失败")?,
    })
}

fn extract_key(url: &str) -> Option<String> {
    url.rsplit('/')
        .next()
        .and_then(|s| s.split('.').next())
        .map(str::to_string)
}