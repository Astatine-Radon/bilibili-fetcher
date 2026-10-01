//! 通过 bvid 获取视频 CID
use anyhow::{bail, Context, Result};

use crate::client::BiliClient;

pub async fn fetch_cid(client: &BiliClient, bvid: &str) -> Result<String> {
    let url = format!(
        "https://api.bilibili.com/x/web-interface/view?bvid={}",
        bvid
    );

    // 最初版本：view 接口什么都不带
    let resp: serde_json::Value = client.http().get(&url).send().await?.json().await?;

    if resp["code"].as_i64() != Some(0) {
        bail!(
            "获取 CID 失败: {}",
            resp["message"].as_str().unwrap_or("未知错误")
        );
    }

    resp["data"]["cid"]
        .as_u64()
        .map(|c| c.to_string())
        .context("响应中未找到 cid 字段")
}