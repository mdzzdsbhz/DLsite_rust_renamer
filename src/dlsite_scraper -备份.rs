// src/scraper/scraper.rs

use std::collections::HashMap;
use thiserror::Error;
use std::path::Path;
use std::fs;
use ureq::Error; // Needed for Error
use ureq::Agent;
use std::time::Duration;

#[derive(Debug, Error)]
pub enum ScraperError {
    #[error("HTTP 请求失败: {0}")]
    HttpRequestError(String),
    #[error("作品不存在")]
    NotFound,
    #[error("页面解析失败: {0}")]
    ParseError(String),
}

/// 爬虫 Trait：可实现为各种站点的爬虫
pub trait Scraper {
    fn fetch_page(&self, url: &str) -> Result<String, ScraperError>;
    /// 下载封面图像（jpg），保存为 {目标目录}/cover.jpg
    fn download_cover<'a>(&'a self, rjcode: &'a str, dest_dir: &'a std::path::Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<()>> + Send + 'a>>;
}

/// 默认实现，用于抓取DLsite页面
pub struct DlsiteScraper {
    headers: HashMap<String, String>,
}

impl DlsiteScraper {
    pub fn new() -> Self {
        let mut headers = HashMap::new();
        headers.insert("User-Agent".into(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114 Safari/537.36".into());
        // headers.insert("Accept-Language".into(), "ja-JP".into());

        DlsiteScraper { headers }
    }
}

impl Scraper for DlsiteScraper {
    fn fetch_page(&self, url: &str) -> Result<String, ScraperError> {
        println!("🌐 开始请求页面: {}", url); // 🧪 1. 看是否调用了

        // 创建 Agent（如需代理，可继续添加配置）
        let agent = Agent::new_with_defaults();

        // 构建请求并附加自定义 headers
        let mut request = agent.get(url);
        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        // 🧪 2. 尝试发送请求
        println!("🚀 发送请求中...");
        let mut response = match request.call() {
            Ok(resp) => {
                println!("✅ 收到响应: {} {}", resp.status(), resp.status());
                resp
            }
            Err(ureq::Error::StatusCode(code)) => {
                println!("❌ 状态码错误: {}", code);
                if code == 404 || code == 410 {
                    return Err(ScraperError::NotFound);
                }
                return Err(ScraperError::HttpRequestError(format!("状态码错误: {}", code)));
            }
            Err(e) => {
                println!("❌ 请求失败: {}", e);
                return Err(ScraperError::HttpRequestError(format!("请求失败: {}", e)));
            }
        };

        // 🧪 3. 尝试读取响应内容
        println!("📖 正在读取响应内容...");
        let mut body = String::new();
        if let Err(e) = response.body_mut().read_to_string() {
            println!("❌ 读取响应失败: {}", e);
            return Err(ScraperError::HttpRequestError(format!("读取响应失败: {}", e)));
        }

        println!("📄 页面大小: {} 字节", body.len());

        if body.contains("この作品は存在しません") {
            println!("⚠️ 页面显示作品不存在");
            return Err(ScraperError::NotFound);
        }

        Ok(body)
    }



        /// 下载封面图像（jpg），保存为 {目标目录}/cover.jpg
    fn download_cover<'a>(&'a self, rjcode: &'a str, dest_dir: &'a Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<()>> + Send + 'a>> {
        Box::pin(async move {
            let url = format!("https://img.dlsite.jp/modpub/images2/work/doujin/RJ{}/{}/{}_img_main.jpg",
                &rjcode[2..5], rjcode, rjcode);

            // You may need to use an async HTTP client here, e.g., reqwest, and ensure self.client is defined.
            // let img_bytes = self.client.get(&url).send().await.ok()?.bytes().await.ok()?;
            // For now, this is a placeholder for the actual download logic.
            // Replace the following lines with your actual async HTTP client logic.
            let img_bytes = match reqwest::get(&url).await.ok()?.bytes().await.ok() {
                Some(bytes) => bytes,
                None => return None,
            };
            let save_path = dest_dir.join("cover.jpg");
            fs::write(save_path, &img_bytes).ok()?;
            Some(())
        })
    }

}
