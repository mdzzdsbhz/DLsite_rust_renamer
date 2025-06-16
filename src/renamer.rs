use std::fs;
use std::path::{Path, PathBuf};
use crate::dlsite_scraper::DlsiteScraper;
use crate::dlsite_scraper::Scraper;
use crate::work_metadata::WorkMetadata;
use crate::dlsite::Dlsite;
use crate::dlsite_scraper::ScraperError;


/// 文件重命名器（泛型，支持任意实现了 Scraper 的抓取器）
pub struct Renamer<S: Scraper> {
    pub scraper: S,
    pub save_cover: bool,
}

impl<S: Scraper> Renamer<S> {
    pub fn new(scraper: S, save_cover: bool) -> Self {
        Self { scraper, save_cover }
    }

    /// 对某个 RJ 目录执行重命名（异步）
    pub async fn rename_folder(&self, rjcode: &str, folder_path: &Path) -> anyhow::Result<()> {
        let scraper = DlsiteScraper::new();
        let dlsite = Dlsite::new(scraper);

        // 抓取作品信息
        let metadata: WorkMetadata = match dlsite.fetch_metadata(rjcode) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("❌ 获取元数据失败: {rjcode}，错误信息: {e}");
                return Ok(()); // 不 panic，跳过此目录
            }
        };

        // 生成新文件夹名
        let new_name = self.generate_name(&metadata);
        let new_path = folder_path.parent().unwrap().join(&new_name);

        // 执行重命名
        if new_path != folder_path {
            fs::rename(folder_path, &new_path)?;
            println!("✅ 已重命名: {} -> {}", rjcode, new_name);
        } else {
            println!("ℹ️ 命名已是最新: {}", new_name);
        }

        // 下载封面（可选）
        if self.save_cover {
            if let Some(e) = self.scraper.download_cover(rjcode, &new_path).await {
                eprintln!("⚠️ 封面下载失败: {:?}", e);
            }
        }

        Ok(())
    }

    /// 使用元数据生成新名称（默认格式）
    fn generate_name(&self, info: &WorkMetadata) -> String {
        let circle = info.circle.as_deref().unwrap_or("Unknown");
        format!("{} [{}] {}", info.rjcode, circle, info.title)
            .replace("/", "／") // 替换非法路径字符
            .replace("\\", "＼")
            .replace(":", "：")
    }
}
