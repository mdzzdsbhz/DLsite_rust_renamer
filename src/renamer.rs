use std::fs;
use std::path::{Path, PathBuf};
use crate::dlsite_scraper::DlsiteScraper;
use crate::dlsite_scraper::Scraper;
use crate::work_metadata::WorkMetadata;
use crate::dlsite::Dlsite;
use crate::dlsite_scraper::ScraperError;
use crate::config::Config; // Add this line to import Config


/// 文件重命名器（泛型，支持任意实现了 Scraper 的抓取器）
pub struct Renamer<S: Scraper> {
    pub scraper: S,
    pub save_cover: bool,
    pub config: Config,
}

impl<S: Scraper> Renamer<S> {
    pub fn new(scraper: S, save_cover: bool, config: Config) -> Self {
        Self { scraper, save_cover, config }
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
        let mut name = self.config.rename_template.clone();

        // 定义模板变量及其实际值
        let circle = info.circle.as_ref().unwrap_or(&"Unknown".to_string()).clone();
        let cv = info.voice_actor.as_ref().unwrap_or(&"Unknown".to_string()).clone();
        let lang = info.lang.as_ref().unwrap_or(&"Unknown".to_string()).clone();
        let release_date = info.release_date.as_ref().unwrap_or(&"Unknown".to_string()).clone();
        let series = info.series.as_ref().unwrap_or(&"Unknown".to_string()).clone();
        let age_rating = info.age_rating.as_ref().unwrap_or(&"Unknown".to_string()).clone();
        let genre = info.tags.join(",");
        let categories = info.categories.join(",");

        let replacements = [
            ("[rjcode]", &info.rjcode),
            ("[title]", &info.title),
            ("[circle]", &circle),
            ("[cv]", &cv),
            ("[genre]", &genre),
            ("[lang]", &lang),
            ("[release_date]", &release_date),
            ("[series]", &series),
            ("[categories]", &categories),
            ("[age_rating]", &age_rating),
        ];

        // 替换模板中的变量
        for (key, value) in replacements {
            name = name.replace(key, value);
        }

        // 替换非法字符
        if self.config.remove_illegal_chars {
            let illegal_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
            name = name
                .chars()
                .map(|c| if illegal_chars.contains(&c) { ' ' } else { c })
                .collect();
        }

        // 保留或去除括号
        if !self.config.keep_brackets {
            name = name.replace('[', "").replace(']', "");
        }

        name.trim().to_string()
    }
}
