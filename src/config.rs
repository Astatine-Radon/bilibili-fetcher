//! 全局配置

pub const COOKIE: &str = "SESSDATA=your_own;\
buvid3=your_own; bili_jct=your_own";

pub const USER_AGENT: &str = "your_user_agent";

pub const BIND_ADDR: &str = "127.0.0.1:3000";
pub const DOWNLOAD_DIR: &str = "downloads";

pub fn ffmpeg_path() -> std::path::PathBuf {
    if let Ok(p) = std::env::var("FFMPEG_PATH") {
        return std::path::PathBuf::from(p);
    }
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("drivers/ffmpeg.exe")
}