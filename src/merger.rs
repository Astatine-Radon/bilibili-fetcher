//! 使用 ffmpeg 无损合并音视频
use anyhow::{bail, Result};
use std::path::Path;

use crate::config;

pub async fn merge(video: &Path, audio: &Path, output: &Path) -> Result<()> {
    let ffmpeg = config::ffmpeg_path();
    if !ffmpeg.exists() {
        bail!(
            "未找到 ffmpeg，请放到 {} 或设置环境变量 FFMPEG_PATH",
            ffmpeg.display()
        );
    }

    let video = video.to_path_buf();
    let audio = audio.to_path_buf();
    let output = output.to_path_buf();

    tokio::task::spawn_blocking(move || {
        let status = std::process::Command::new(&ffmpeg)
            .args([
                "-i", video.to_str().unwrap(),
                "-i", audio.to_str().unwrap(),
                "-c", "copy",
                output.to_str().unwrap(),
                "-y",
                "-loglevel", "error",
            ])
            .status()?;
        if !status.success() {
            anyhow::bail!("ffmpeg 合并失败");
        }
        Ok::<(), anyhow::Error>(())
    })
    .await??;

    Ok(())
}