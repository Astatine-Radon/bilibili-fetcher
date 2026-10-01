//! 通用工具函数
use anyhow::{Context, Result};

pub fn extract_bvid(text: &str) -> Result<String> {
    let re = regex::Regex::new(r"BV[0-9A-Za-z]{10}").unwrap();
    re.find(text)
        .map(|m| m.as_str().to_string())
        .context("未找到有效的 BV 号")
}