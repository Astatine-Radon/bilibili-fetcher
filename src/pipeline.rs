//!编排 + 推送事件
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;

use crate::api;
use crate::client::BiliClient;
use crate::config;
use crate::downloader;
use crate::merger;
use crate::utils;
use crate::web::tasks::{Task, TaskEvent};

pub async fn run(task: Arc<Task>, client: BiliClient) {
    if let Err(e) = run_inner(&task, &client).await {
        task.emit(TaskEvent::Failed {
            message: format!("{:#}", e),
        })
        .await;
    }
}

async fn run_inner(task: &Arc<Task>, client: &BiliClient) -> Result<()> {
    let raw_url = task.url.clone();
    let bvid = utils::extract_bvid(&raw_url)?;

    let out_dir = PathBuf::from(config::DOWNLOAD_DIR).join(&task.id);
    tokio::fs::create_dir_all(&out_dir).await?;

    let video_path = out_dir.join("video.m4s");
    let audio_path = out_dir.join("audio.m4s");
    let output_path = out_dir.join("output.mp4");

    // 1. CID
    task.status("正在获取视频信息...").await;
    let cid = api::view::fetch_cid(client, &bvid).await?;

    // 2. 流地址
    task.status("正在解析视频流地址...").await;
    let streams = api::playurl::fetch_streams(client, &bvid, &cid).await?;

    // 3. 视频流
    task.status("开始下载视频流...").await;
    {
        let t = Arc::clone(task);
        downloader::download(client, &streams.video_url, &video_path, &raw_url, |d, total| {
            let t = Arc::clone(&t);
            async move { t.progress("video", d, total).await }
        })
        .await?;
    }

    // 4. 音频流
    task.status("开始下载音频流...").await;
    {
        let t = Arc::clone(task);
        downloader::download(client, &streams.audio_url, &audio_path, &raw_url, |d, total| {
            let t = Arc::clone(&t);
            async move { t.progress("audio", d, total).await }
        })
        .await?;
    }

    // 5. 合并
    task.status("正在合并音视频...").await;
    merger::merge(&video_path, &audio_path, &output_path).await?;

    let size = tokio::fs::metadata(&output_path).await?.len();
    *task.file_path.write().await = Some(output_path);

    task.emit(TaskEvent::Done {
        filename: "output.mp4".into(),
        size,
    })
    .await;

    Ok(())
}