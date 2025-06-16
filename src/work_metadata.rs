// src/work_metadata/work_metadata.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkMetadata {
    pub rjcode: String,
    pub title: String,
    pub circle: Option<String>,
    pub release_date: Option<String>,
    pub tags: Vec<String>,
    pub voice_actor: Option<String>,
    pub series: Option<String>,
    pub categories: Vec<String>,
    pub age_rating: Option<String>,
    pub lang: Option<String>, // 推测语言
}

impl WorkMetadata {
    /// 创建一个新作品元数据，至少包含rjcode
    pub fn new(rjcode: &str) -> Self {
        WorkMetadata {
            rjcode: rjcode.to_string(),
            title: String::new(),
            circle: None,
            release_date: None,
            tags: Vec::new(),
            voice_actor: None,
            series: None,
            categories: Vec::new(),
            age_rating: None,
            lang: None,
        }
    }

    /// 从 JSON 字符串解析
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    /// 转换为 JSON 字符串
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// 猜测作品的语言，使用标签字段
    pub fn guess_lang(&mut self) {
        let lower_tags: Vec<String> = self.tags.iter().map(|t| t.to_lowercase()).collect();

        if lower_tags.iter().any(|t| t.contains("chinese") || t.contains("中文")) {
            self.lang = Some("zh".to_string());
        } else if lower_tags.iter().any(|t| t.contains("english")) {
            self.lang = Some("en".to_string());
        } else {
            self.lang = Some("ja".to_string()); // 默认日文
        }
    }

    /// 直接构造 WorkMetadata 从结构化字段（可用在 parse_metadata 中）
    pub fn from_fields(
        rjcode: &str,
        title: &str,
        circle: Option<&str>,
        release_date: Option<&str>,
        tags: Vec<&str>,
        voice_actor: Option<&str>,
        series: Option<&str>,
        categories: Vec<&str>,
        age_rating: Option<&str>,
    ) -> Self {
        let mut meta = WorkMetadata {
            rjcode: rjcode.to_string(),
            title: title.to_string(),
            circle: circle.map(str::to_string),
            release_date: release_date.map(str::to_string),
            tags: tags.into_iter().map(str::to_string).collect(),
            voice_actor: voice_actor.map(str::to_string),
            series: series.map(str::to_string),
            categories: categories.into_iter().map(str::to_string).collect(),
            age_rating: age_rating.map(str::to_string),
            lang: None,
        };
        meta.guess_lang();
        meta
    }
}
