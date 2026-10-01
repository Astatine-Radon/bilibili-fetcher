# BiliDownloader

一个用 Rust 编写的 B 站视频下载器，带 Web 界面。

粘贴链接 → 自动下载最高画质视频和音频 → 用 ffmpeg 无损合并成 MP4。

## ✨ 特性

- 🌐 **Web 界面**：浏览器操作，实时进度条，下载完直接点按钮保存
- 🎬 **DASH 音视频分离下载**：自动获取最高画质流，ffmpeg 无损合并
- ⚡ **异步架构**：基于 tokio + axum，支持并发处理多个下载任务
- 📦 **单二进制**：前端资源全部内嵌，编译后只需一个 exe + ffmpeg
- 🌙 **深色 UI**：B 站风格配色，护眼

## 📋 环境要求

| 依赖 | 版本 | 说明 |
|---|---|---|
| Rust | 1.75+ | 编译用 |
| ffmpeg | 任意近期版本 | 合并音视频必需 |

## 🚀 快速开始

### 1. 克隆项目

```bash
git clone https://github.com/<你的用户名>/bili-downloader.git
cd bili-downloader
```

### 2. 准备 ffmpeg

从 [ffmpeg 官网](https://ffmpeg.org/download.html) 或 [gyan.dev](https://www.gyan.dev/ffmpeg/builds/) 下载，把 `ffmpeg.exe`（Windows）或 `ffmpeg`（Linux/macOS）放到项目根目录的 `drivers/` 文件夹下：

```
bili-downloader/
├── drivers/
│   └── ffmpeg.exe
├── src/
└── Cargo.toml
```

或者把它放到任意位置，通过环境变量指定：

```bash
# Linux / macOS
export FFMPEG_PATH=/path/to/ffmpeg

# Windows (PowerShell)
$env:FFMPEG_PATH = "D:\ffmpeg\bin\ffmpeg.exe"
```

### 3. 配置 Cookie

打开 `src/config.rs`，找到 `COOKIE` 常量，替换成你自己的 B 站 Cookie：

```rust
pub const COOKIE: &str = "SESSDATA=你的值; buvid3=你的值; bili_jct=你的值";
```

具体怎么获取见下方 [如何获取 Cookie](#-如何获取-cookie)。

### 4. 编译并运行

```bash
cargo run --release
```

看到下面的输出说明启动成功：

```
🎬 B站下载器已启动
👉 请在浏览器打开: http://127.0.0.1:3000
按 Ctrl+C 退出
```

### 5. 使用

浏览器打开 <http://127.0.0.1:3000>，粘贴任意 B 站视频链接（例如 `https://www.bilibili.com/video/BV1xx411c7mD`），点击 **开始下载**。

等待完成后，页面会出现 **⬇ 下载 MP4** 按钮，点击即可保存到本地。

## 🍪 如何获取 Cookie

1. 用浏览器登录 [bilibili.com](https://www.bilibili.com)
2. 按 `F12` 打开开发者工具
3. 切换到 **Application**（应用）标签
4. 左侧选择 **Cookies → https://www.bilibili.com**
5. 找到以下三个字段，复制它们的值：

   | 字段 | 作用 | 是否必须 |
   |---|---|---|
   | `SESSDATA` | 登录凭证 | ✅ 必须 |
   | `buvid3` | 设备指纹 | ✅ 必须 |
   | `bili_jct` | CSRF token | ✅ 必须 |

6. 按下述格式拼成一行，替换 `src/config.rs` 里的 `COOKIE`：

   ```
   SESSDATA=xxxxx; buvid3=xxxxx; bili_jct=xxxxx
   ```

> ⚠️ **注意**：`SESSDATA` 有时效性（一般几个月）。失效后程序会报"获取 CID 失败"，届时按上面步骤重新复制一份即可。

## 📁 项目结构

```
src/
├── main.rs          启动 HTTP 服务
├── config.rs        Cookie / User-Agent / ffmpeg 路径等常量
├── client.rs        reqwest 客户端封装
├── utils.rs         BV 号提取等工具函数
├── wbi.rs           B 站 WBI 签名算法
├── downloader.rs    流式下载
├── merger.rs        ffmpeg 合并
├── pipeline.rs      下载流程编排
├── api/             接口调用
│   ├── mod.rs
│   ├── nav.rs       WBI 密钥获取
│   ├── view.rs      视频 CID 获取
│   └── playurl.rs   音视频流地址解析
├── web/             axum 路由与任务管理
│   ├── mod.rs
│   ├── handlers.rs  HTTP handlers
│   └── tasks.rs     任务状态 + SSE 事件广播
└── static/          前端资源（编译时内嵌进二进制）
    ├── index.html
    ├── style.css
    └── app.js
```

## 🛠️ 技术栈

- **异步运行时**：[tokio](https://tokio.rs)
- **Web 框架**：[axum](https://github.com/tokio-rs/axum)
- **HTTP 客户端**：[reqwest](https://github.com/seanmonstar/reqwest)
- **进度推送**：SSE（Server-Sent Events）
- **前端**：原生 HTML / CSS / JS，无构建步骤

## ❓ 常见问题

### Q：启动后报 "未找到 ffmpeg"

把 `ffmpeg.exe` 放到 `drivers/` 目录，或设置 `FFMPEG_PATH` 环境变量。

### Q：下载报 "HTTP 403 Forbidden"

通常是 Cookie 过期或 B 站风控。按以下顺序排查：

1. 在浏览器里重新登录 B 站，复制一份新的 Cookie 替换 `src/config.rs`
2. 确认 `buvid3` 的值和浏览器里当前一致
3. 换一个视频试试，如果所有视频都失败，基本就是 Cookie 问题
4. 检查 `config.rs` 里的 `COOKIE` 是不是还带着 `填你的` 之类的占位符

### Q：合并报 "Invalid data found when processing input"

说明下载到的文件不是有效视频。用十六进制编辑器打开 `downloads/<任务ID>/video.m4s`，如果开头是 `<html>` 或者一堆 `0`，说明下载请求被 CDN 拒绝了（返回的是错误页而不是视频）。

排查方向：
- 检查请求头是否是 **只有 `Referer` + `Cookie`**（本项目的设计原则，多余的头部反而会被 CDN 判为异常请求）
- 检查 `Referer` 是不是用户输入的完整视频页 URL

### Q：端口 3000 被占用

修改 `src/config.rs` 里的 `BIND_ADDR`，比如改成 `127.0.0.1:8080`。

### Q：想要 4K / 8K 画质

在 `src/api/playurl.rs` 里调整 `qn` 参数。但需要你的账号本身具有该画质权限（大会员）。

### Q：下载到一半中断了怎么办

直接重新提交任务即可，程序会从头开始下载（暂不支持断点续传）。

## 🗺️ Roadmap

- [ ] 首次启动时检测占位符 Cookie，引导用户在浏览器中填写
- [ ] 支持多 P 视频选择
- [ ] 支持选择清晰度（不再写死 1080P）
- [ ] 下载历史记录持久化
- [ ] 断点续传

## ⚠️ 免责声明

本项目仅供 **个人学习和技术研究** 使用，请勿用于任何商业用途或大规模批量下载。

- 下载的视频版权归 **B 站及原视频作者** 所有
- 请遵守 [B 站用户协议](https://www.bilibili.com/protocal/licence.html) 及相关法律法规
- 请勿将下载的内容进行二次上传或传播
- 使用本工具产生的一切后果由使用者自行承担

## 📄 License

[MIT](LICENSE)

## 🙏 致谢

- [bilibili-API-collect](https://github.com/SocialSisterYi/bilibili-API-collect)：社区维护的 B 站 API 文档
- [WBI 签名算法说明](https://github.com/SocialSisterYi/bilibili-API-collect/blob/master/docs/misc/sign/wbi.md)
