//! 请求 playurl 接口，解析出最高画质的音视频流地址
use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::collections::HashMap;

use crate::api::nav::fetch_wbi_keys;
use crate::client::BiliClient;
use crate::config;
use crate::wbi;

pub struct Streams {
    pub video_url: String,
    pub audio_url: String,
}

pub async fn fetch_streams(client: &BiliClient, bvid: &str, cid: &str) -> Result<Streams> {
    let keys = fetch_wbi_keys(client).await?;

    let mut params = HashMap::new();
    params.insert("bvid".into(), bvid.to_string());
    params.insert("cid".into(), cid.to_string());
    params.insert("qn".into(), "80".into());
    params.insert("fnval".into(), "4048".into());
    params.insert("fourk".into(), "1".into());
    params.insert("try_look".into(), "1".into());

    let signed_query = wbi::wbi_sign(params, &keys.img_key, &keys.sub_key);
    let url = format!(
        "https://api.bilibili.com/x/player/wbi/playurl?{}",
        signed_query
    );

    // 最初版本：Cookie + Referer https://www.bilibili.com/
    let resp: Value = client
        .http()
        .get(&url)
        .header("Cookie", config::COOKIE)
        .header("Referer", "https://www.bilibili.com/")
        .send()
        .await?
        .json()
        .await?;

    if resp["code"].as_i64() != Some(0) {
        bail!(
            "请求 playurl 失败: {}",
            resp["message"].as_str().unwrap_or("未知错误")
        );
    }

    let dash = &resp["data"]["dash"];
    Ok(Streams {
        video_url: pick_best_url(dash, "video")?,
        audio_url: pick_best_url(dash, "audio")?,
    })
}

/// 严格还原最初版本：max_by_key 取 id 最大的
fn pick_best_url(dash: &Value, kind: &str) -> Result<String> {
    dash[kind]
        .as_array()
        .with_context(|| format!("未找到 {} 数组", kind))?
        .iter()
        .max_by_key(|v| v["id"].as_u64().unwrap_or(0))
        .with_context(|| format!("{} 数组为空", kind))?
        ["baseUrl"]
        .as_str()
        .with_context(|| format!("未找到 {} 的 baseUrl", kind))
        .map(str::to_string)
}