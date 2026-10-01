//! 主入口：串联整个下载流程
mod api;
mod client;
mod config;
mod downloader;
mod merger;
mod pipeline;
mod utils;
mod wbi;
mod web;

use anyhow::Result;
use client::BiliClient;

#[tokio::main]
async fn main() -> Result<()> {
    let client = BiliClient::new()?;
    let app = web::router(client);
    let listener = tokio::net::TcpListener::bind(config::BIND_ADDR).await?;

    println!("🎬 B站下载器已启动");
    println!("👉 请在浏览器打开: http://{}", config::BIND_ADDR);
    println!("按 Ctrl+C 退出\n");

    axum::serve(listener, app).await?;
    Ok(())
}