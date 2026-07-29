//! FLV 直链下载器 — 用于 FLV 流（某些平台的流不适合 FFmpeg 处理）
//!
//! 参考 StreamCap 的 direct_downloader.py

use reqwest::Client;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::{info, warn};

pub struct DirectDownloader {
    recording_id: String,
    stream_url: String,
    output_path: PathBuf,
    should_stop: bool,
    user_agent: String,
    proxy_url: Option<String>,
}

impl DirectDownloader {
    pub fn new(
        recording_id: &str,
        stream_url: &str,
        output_path: PathBuf,
        user_agent: &str,
        proxy_url: Option<&str>,
    ) -> Self {
        Self {
            recording_id: recording_id.to_string(),
            stream_url: stream_url.to_string(),
            output_path,
            should_stop: false,
            user_agent: user_agent.to_string(),
            proxy_url: proxy_url.map(|s| s.to_string()),
        }
    }

    pub fn request_stop(&mut self) {
        self.should_stop = true;
    }

    /// 启动直链下载
    pub async fn start_download(&mut self) -> Result<(), String> {
        let mut client_builder = Client::builder()
            .user_agent(&self.user_agent)
            .timeout(std::time::Duration::from_secs(30));

        // 代理设置
        if let Some(ref proxy) = self.proxy_url {
            if !proxy.is_empty() {
                let proxy = reqwest::Proxy::all(proxy)
                    .map_err(|e| format!("设置代理失败: {}", e))?;
                client_builder = client_builder.proxy(proxy);
            }
        }

        let client = client_builder
            .build()
            .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

        info!("开始直链下载: {} -> {:?}", self.stream_url, self.output_path);

        let response = client
            .get(&self.stream_url)
            .send()
            .await
            .map_err(|e| format!("下载请求失败: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("下载请求返回错误: {}", response.status()));
        }

        let mut file = File::create(&self.output_path)
            .await
            .map_err(|e| format!("创建输出文件失败: {}", e))?;

        let mut stream = response.bytes_stream();
        use futures_util::StreamExt; // Need futures_util for .next()

        while let Some(chunk_result) = stream.next().await {
            if self.should_stop {
                info!("收到停止信号，结束直链下载");
                break;
            }

            match chunk_result {
                Ok(chunk) => {
                    file.write_all(&chunk)
                        .await
                        .map_err(|e| format!("写入文件失败: {}", e))?;
                }
                Err(e) => {
                    warn!("下载数据块失败: {}", e);
                    return Err(format!("下载失败: {}", e));
                }
            }
        }

        file.flush().await.map_err(|e| format!("刷新文件失败: {}", e))?;
        info!("直链下载完成: {:?}", self.output_path);
        Ok(())
    }
}
