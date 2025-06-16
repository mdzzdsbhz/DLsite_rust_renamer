use std::fs;
use std::path::{Path, PathBuf};
use crate::dlsite_scraper::{DlsiteScraper, Scraper, ScraperError};
use crate::work_metadata::WorkMetadata;
use crate::dlsite::Dlsite;
use crate::config::Config;
use crate::ui_logger;

/// 文件重命名器（泛型，支持任意实现了 Scraper 的抓取器）
pub struct Renamer<S: Scraper> {
    pub scraper: S,
    pub save_cover: bool,
    pub config: Config,
    pub metadata: Option<WorkMetadata>, // ✅ 可选元数据缓存
}

impl<S: Scraper> Renamer<S> {
    /// 默认构造方式：需要爬虫获取元数据
    pub fn new(scraper: S, save_cover: bool, config: Config) -> Self {
        Self {
            scraper,
            save_cover,
            config,
            metadata: None,
        }
    }

    /// 新增构造方式：已获取元数据，避免重复抓取
    pub fn new_from_metadata(scraper: S, save_cover: bool, config: Config, metadata: WorkMetadata) -> Self {
        Self {
            scraper,
            save_cover,
            config,
            metadata: Some(metadata),
        }
    }

    /// 对某个 RJ 目录执行重命名（异步）
    pub async fn rename_folder(&self, rjcode: &str, folder_path: &Path) -> anyhow::Result<()> {

        let metadata = if let Some(ref cached) = self.metadata {
            cached.clone()
        } else {
            let dlsite = Dlsite::new(DlsiteScraper::new());
            match dlsite.fetch_metadata(rjcode) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("❌ 获取元数据失败: {rjcode}，错误信息: {e}");
                    return Ok(());
                }
            }
        };

        // 生成新文件夹名
        let new_name = self.generate_name(&metadata);
        let new_path = folder_path.parent().unwrap().join(&new_name);

        // 执行重命名
        if new_path != folder_path {
            fs::rename(folder_path, &new_path)?;

            log::info!("✅ 已重命名: {} -> {}", rjcode, new_name);
        } else {
            log::info!("ℹ️ 命名已是最新: {}", new_name);
        }

        // 下载封面（可选）
        if self.save_cover {
            if let Some(e) = self.scraper.download_cover(rjcode, &new_path).await {
                log::info!("⚠️ 封面下载失败: {:?}", e);
            }
        }

        Ok(())
    }

    /// 使用元数据生成新名称（默认格式）
    fn generate_name(&self, info: &WorkMetadata) -> String {
        let mut name = self.config.rename_template.clone();

        let circle = info.circle.as_ref().unwrap_or(&"".to_string()).clone();
        let cv = info.voice_actor.as_ref().unwrap_or(&"".to_string()).clone();
        let lang = info.lang.as_ref().unwrap_or(&"".to_string()).clone();
        let release_date = info.release_date.as_ref().unwrap_or(&"".to_string()).clone();
        let series = info.series.as_ref().unwrap_or(&"".to_string()).clone();
        let age_rating = info.age_rating.as_ref().unwrap_or(&"".to_string()).clone();
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

        for (key, value) in replacements {
            name = name.replace(key, value);
        }

        if self.config.remove_illegal_chars {
            let illegal_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
            name = name
                .chars()
                .map(|c| if illegal_chars.contains(&c) { ' ' } else { c })
                .collect();
        }

        if !self.config.keep_brackets {
            name = name.replace('[', "").replace(']', "");
        }

        name.trim().to_string()
    }
}
