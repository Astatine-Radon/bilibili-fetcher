use anyhow::{Context, Result};
use futures_util::StreamExt;
use std::future::Future;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::client::BiliClient;

pub async fn download<F, Fut>(
    client: &BiliClient,
    url: &str,
    path: &Path,
    referer: &str,
    on_progress: F,
) -> Result<u64>
where
    F: Fn(u64, Option<u64>) -> Fut,
    Fut: Future<Output = ()>,
{
    // ★ 关键：走 client.get_stream，只带 Referer + Cookie
    let resp = client.get_stream(url, referer).await?;
    let total = resp.content_length();

    let mut file = File::create(path)
        .await
        .with_context(|| format!("创建文件失败: {}", path.display()))?;
    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = 0;
    let mut last_reported: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;

        if downloaded - last_reported >= 200 * 1024 {
            on_progress(downloaded, total).await;
            last_reported = downloaded;
        }
    }
    file.flush().await?;
    on_progress(downloaded, total).await;
    Ok(downloaded)
}