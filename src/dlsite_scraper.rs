// src/scraper/scraper.rs

use std::collections::HashMap;
use thiserror::Error;
use std::path::Path;
use std::fs;
use tokio::fs as async_fs;
use ureq::Error; // Needed for Error
use ureq::Agent;
use std::time::Duration;
use std::io::Read;
use headless_chrome::Browser;
use anyhow::Context;

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
    fn fetch_page_json(&self, url: &str) -> Result<serde_json::Value, ScraperError>;
}

/// 默认实现，用于抓取DLsite页面
pub struct DlsiteScraper {
    headers: HashMap<String, String>,
}

impl DlsiteScraper {
    pub fn new() -> Self {
        let mut headers = HashMap::new();
        headers.insert("User-Agent".into(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114 Safari/537.36".into());
        headers.insert("Accept-Language".into(), "ja-JP".into());

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
        let body = match response.body_mut().read_to_string() {
            Ok(s) => s,
            Err(e) => {
                println!("❌ 读取响应失败: {}", e);
                return Err(ScraperError::HttpRequestError(format!("读取响应失败: {}", e)));
            }
        };

        if body.contains("この作品は存在しません") {
            println!("⚠️ 页面显示作品不存在");
            return Err(ScraperError::NotFound);
        }
        Ok(body)
    }


    fn fetch_page_json(&self, url: &str) -> Result<serde_json::Value, ScraperError> {
        println!("🌐 请求页面: {}", url);

        let mut req = ureq::get(url);
        for (k, v) in &self.headers {
            req = req.header(k, v);
        }

        let mut response = req.call().map_err(|e| {
            println!("❌ 请求失败: {}", e);
            match e {
                ureq::Error::StatusCode(code) if code == 404 || code == 410 => ScraperError::NotFound,
                ureq::Error::StatusCode(code) => {
                    ScraperError::HttpRequestError(format!("状态码错误: {}", code))
                }
                _ => ScraperError::HttpRequestError(format!("{}", e)),
            }
        })?;

        println!("✅ 状态: {}", response.status());

        let body = response.body_mut().read_to_string().map_err(|e| {
            println!("❌ 读取失败: {}", e);
            ScraperError::HttpRequestError(format!("{}", e))
        })?;

        // 尝试解析完整 JSON 字符串
        let json: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
            println!("❌ JSON 解析失败: {}", e);
            ScraperError::HttpRequestError(format!("解析 JSON 失败: {}", e))
        })?;

        if json.is_array() {
            println!("✅ JSON 数组解析成功，共 {} 项", json.as_array().unwrap().len());
            Ok(json)
        } else {
            Err(ScraperError::HttpRequestError(
                "返回的 JSON 不是数组".into(),
            ))
        }
    }

    /// 下载封面图像（jpg），保存为 {目标目录}/cover.jpg
    fn download_cover<'a>(&'a self, rjcode: &'a str, dest_dir: &'a Path)
        -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<()>> + Send + 'a>>
    {
        Box::pin(async move {
            if !rjcode.starts_with("RJ") || rjcode.len() < 5 {
                println!("非法 RJ 码：{}", rjcode);
                return None;
            }


            let numeric = rjcode.get(2..7)?.parse::<u32>().ok()?; // 取 01240
            let incremented = numeric + 1;                        // 加一，得到 01241
            let dir = format!("RJ{:05}000", incremented);          // 拼 RJ0124100
            let url = format!(
                "https://img.dlsite.jp/modpub/images2/work/doujin/{}/{}_img_main.jpg",
                dir, rjcode
            );
            println!("{}",url);

        let client = reqwest::Client::new();
        let resp = match client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                println!("下载失败：{}，错误：{}", url, e);
                return None;
            }
        };

            if !resp.status().is_success() {
                println!("请求失败：{} 状态码 {}", url, resp.status());
                return None;
            }

            let img_bytes = match resp.bytes().await {
                Ok(b) => b,
                Err(e) => {
                    println!("读取响应字节失败: {}", e);
                    return None;
                }
            };

            let save_path = dest_dir.join("cover.jpg");
            if let Err(e) = async_fs::write(&save_path, &img_bytes).await {
                println!("保存图像失败：{}，错误：{}", save_path.display(), e);
                return None;
            }

            Some(())
        })
    }

}


