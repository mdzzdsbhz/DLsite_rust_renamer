use crate::dlsite_scraper::ScraperError;
use crate::work_metadata::WorkMetadata;
use crate::dlsite_scraper::Scraper;
use scraper::{Html, Selector};
use serde_json::Value;

pub struct Dlsite<S: Scraper> {
    pub scraper: S,
}

impl<S: Scraper> Dlsite<S> {
    pub fn new(scraper: S) -> Self {
        Dlsite { scraper }
    }

    /// 构造作品网页 URL
    fn compile_work_page_url(rjcode: &str) -> String {
        format!("https://www.dlsite.com/maniax/work/=/product_id/{}.html", rjcode)
    }

    /// 构造 JSON API URL
    fn compile_product_api_url(rjcode: &str) -> String {
        format!(
            "https://www.dlsite.com/maniax/api/=/product.json?workno={}", rjcode
        )
    }

    /// 获取元数据主函数
    pub fn fetch_metadata(&self, rjcode: &str) -> Result<WorkMetadata, ScraperError> {
        let html_url = Self::compile_work_page_url(rjcode);
        println!("🔍 正在解析元数据: {}", html_url);
        let html = self.scraper.fetch_page(&html_url)?;

        let json_url = Self::compile_product_api_url(rjcode);
        println!("🌐 获取 JSON 元数据: {}", json_url);
        let json_data = self.scraper.fetch_page_json(&json_url)?;

        // ⭐ 注意：API 返回的是 { "RJxxxxx": { ... } }
        let work_data = json_data
            .get(rjcode)
            .ok_or_else(|| ScraperError::ParseError("找不到对应的 RJ 编号数据".to_string()))?;

        self.parse_metadata(rjcode, &html, Some(work_data.clone()))
    }

    fn parse_metadata(&self, rjcode: &str, html: &str, json_data: Option<Value>) -> Result<WorkMetadata, ScraperError> {
        let document = Html::parse_document(html);

        let sel_title = Selector::parse("span#work_name").unwrap();
        let sel_circle = Selector::parse("span#maker_name a").unwrap();
        let sel_genres = Selector::parse("span#work_genre span.genre_item").unwrap();
        let sel_tags = Selector::parse("span#work_memo span.genre_item").unwrap();
        let sel_release_date = Selector::parse("th:contains(\"販売日\") + td").ok();
        let sel_voice = Selector::parse("th:contains(\"声優\") + td a").ok();
        let sel_series = Selector::parse("th:contains(\"シリーズ名\") + td a").ok();

        let title = document
            .select(&sel_title)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "(无标题)".to_string());

        let circle = document
            .select(&sel_circle)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string());

        let release_date = sel_release_date.as_ref().and_then(|sel| {
            document
                .select(sel)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
        });

        let tags: Vec<String> = document
            .select(&sel_tags)
            .map(|e| e.text().collect::<String>().trim().to_string())
            .collect();

        let categories: Vec<String> = document
            .select(&sel_genres)
            .map(|e| e.text().collect::<String>().trim().to_string())
            .collect();

        let voice_actor = sel_voice.as_ref().and_then(|sel| {
            let actors: Vec<_> = document
                .select(sel)
                .map(|e| e.text().collect::<String>().trim().to_string())
                .collect();
            if actors.is_empty() {
                None
            } else {
                Some(actors.join(" / "))
            }
        });

        let series = sel_series.as_ref().and_then(|sel| {
            document
                .select(sel)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
        });

        let language = json_data.as_ref()
            .and_then(|json| json.get("language"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut metadata = WorkMetadata::from_fields(
            rjcode,
            &title,
            circle.as_deref(),
            release_date.as_deref(),
            tags.iter().map(String::as_str).collect(),
            voice_actor.as_deref(),
            series.as_deref(),
            categories.iter().map(String::as_str).collect(),
            language.as_deref(), // 使用 JSON 中的语言
        );

        metadata.guess_lang();

        Ok(metadata)
    }
}
