// src/cached_scraper_db.rs
use crate::dlsite_scraper::{Scraper, ScraperError}; // 你已有的模块
use rusqlite::{params, Connection};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::work_metadata::WorkMetadata;
use crate::dlsite::Dlsite;
use crate::log_to_ui;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkMeta {
    pub rjcode: String,
    pub title: String,
    pub maker: String,
    pub date: Option<String>,
    pub cover_url: Option<String>,
}

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("数据库错误: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("序列化错误: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("爬虫错误: {0}")]
    Scraper(#[from] ScraperError),
}

pub struct CachedScraperDb {
    conn: Connection,
}

impl CachedScraperDb {
    pub fn new(db_path: &str) -> Result<Self, CacheError> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS work_cache (
                rjcode TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                fetched_at INTEGER NOT NULL
            );",
        )?;
        Ok(Self { conn })
    }

        /// 获取或爬取数据（主接口）
    pub fn get_or_fetch(
        &self,
        scraper: impl Scraper,
        rjcode: &str,
        logs: &std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    ) -> Result<WorkMeta, CacheError> {
        if let Some(cached) = self.get(rjcode)? {
            log_to_ui!(logs,"🧊 缓存命中：{}", rjcode);
            return Ok(cached);
        }
        log_to_ui!(logs,"🔥 未命中缓存，开始爬取：{}", rjcode);

        // ✅ 使用 Dlsite 封装抓取逻辑
        let dlsite = Dlsite::new(scraper);
        let metadata: WorkMetadata = dlsite.fetch_metadata(rjcode)?;

        // ✅ 将 WorkMetadata 映射为 WorkMeta
        let meta = WorkMeta {
            rjcode: rjcode.to_string(),
            title: metadata.title,
            maker: metadata.circle.unwrap_or_default(),
            date: metadata.release_date,
            cover_url: None, // WorkMetadata 没有 cover_url 字段
        };

        self.insert(&meta)?;
        Ok(meta)
    }

    fn get(&self, rjcode: &str) -> Result<Option<WorkMeta>, CacheError> {
        let mut stmt = self
            .conn
            .prepare("SELECT data FROM work_cache WHERE rjcode = ?1")?;
        let row: Option<String> = stmt
            .query_row(params![rjcode], |row| row.get(0))
            .optional()?;

        match row {
            Some(json) => Ok(Some(serde_json::from_str(&json)?)),
            None => Ok(None),
        }
    }

    fn insert(&self, meta: &WorkMeta) -> Result<(), CacheError> {
        let json = serde_json::to_string(meta)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO work_cache (rjcode, data, fetched_at) VALUES (?1, ?2, strftime('%s','now'))",
            params![meta.rjcode, json],
        )?;
        Ok(())
    }
}

// 简化的提取工具函数
fn extract_between<'a>(html: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let s = html.find(start)? + start.len();
    let e = html[s..].find(end)? + s;
    Some(&html[s..e])
}
