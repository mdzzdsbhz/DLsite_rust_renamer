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

        let work_data = json_data
            .as_array()
            .and_then(|arr| arr.first())
            .ok_or_else(|| {
                eprintln!("⚠️ JSON 数据结构异常，未找到数组或首个元素为空！");
                ScraperError::ParseError(format!("未获取到 {} 的 JSON 数据", rjcode))
            })?;

        let metadata = self.parse_metadata(rjcode, &html, Some(work_data.clone()))?;

        println!("\n✅ 元数据解析完成：");
        println!("{:#?}", metadata);

        Ok(metadata)
    }

    fn parse_metadata(&self, rjcode: &str, html: &str, json_data: Option<Value>) -> Result<WorkMetadata, ScraperError> {
        let document = Html::parse_document(html);

        let sel_circle = Selector::parse("span#maker_name a").unwrap();
        let sel_genres = Selector::parse("span#work_genre span.genre_item").unwrap();
        let sel_tags = Selector::parse("span#work_memo span.genre_item").unwrap();
        let sel_release_date = Selector::parse("th:contains(\"販売日\") + td").ok();
        let sel_voice = Selector::parse("th:contains(\"声優\") + td a").ok();
        let sel_series = Selector::parse("th:contains(\"シリーズ名\") + td a").ok();

        // ⚙️ 工具函数：从 JSON 数组/对象中获取字段

    let title = json_data
        .as_ref()
        .and_then(|json| Self::extract_first_string_field(json, "work_name"))
        .unwrap_or_else(|| "(无标题)".to_string());

    let circle = json_data
        .as_ref()
        .and_then(|json| Self::extract_first_string_field(json, "maker_name"))
        .or_else(|| {
            document
                .select(&sel_circle)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
        });

    let release_date = json_data
        .as_ref()
        .and_then(|json| Self::extract_first_string_field(json, "update_date"))
        .or_else(|| {
            sel_release_date.as_ref().and_then(|sel| {
                document
                    .select(sel)
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
            })
        });

    let tags: Vec<String> = if let Some(json) = json_data.as_ref() {
        if let Some(arr) = Self::extract_first_array_of_strings(json, "genres") {
            arr
        } else {
            document
                .select(&sel_tags)
                .map(|e| e.text().collect::<String>().trim().to_string())
                .collect()
        }
    } else {
        document
            .select(&sel_genres)
            .map(|e| e.text().collect::<String>().trim().to_string())
            .collect()
    };

    let voice_actor = json_data
        .as_ref()
        .and_then(|json| {
            json.get("creaters")
                .and_then(|c| c.get("voice_by"))
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.get(0))
                .and_then(|item| item.get("name"))
                .and_then(|name| name.as_str())
                .map(|s| s.to_string())
        })
        .or_else(|| {
            sel_voice.as_ref().and_then(|sel| {
                let actors: Vec<_> = document
                    .select(sel)
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .collect();
                if actors.is_empty() {
                    None
                } else {
                    Some(actors.join(" / "))
                }
            })
        });

    let series = json_data
        .as_ref()
        .and_then(|json| Self::extract_first_string_field(json, "series"))
        .or_else(|| {
            sel_series.as_ref().and_then(|sel| {
                document
                    .select(sel)
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
            })
        });

    let language = json_data
        .as_ref()
        .and_then(|json| Self::extract_first_string_field(json, "language"));

    // Populate categories from JSON or HTML as needed
    let categories: Vec<String> = json_data
        .as_ref()
        .and_then(|json| Self::extract_first_array_of_strings(json, "categories"))
        .unwrap_or_else(|| Vec::new());

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
    println!("✅ 解析完成：{:#?}", metadata);
    Ok(metadata)
}

// 额外工具函数
fn extract_first_array_of_strings(json: &Value, key: &str) -> Option<Vec<String>> {
        if let Some(arr) = json.as_array() {
            for obj in arr {
                if let Some(array) = obj.get(key).and_then(|v| v.as_array()) {
                    return Some(
                        array
                            .iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect(),
                    );
                }
            }
        } else if let Some(array) = json.get(key).and_then(|v| v.as_array()) {
            return Some(
                array
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect(),
            );
        }
        None
    }

    // ⚙️ 工具函数：从 JSON 数组/对象中获取字段
    fn extract_first_string_field(json: &Value, key: &str) -> Option<String> {
        if let Some(arr) = json.as_array() {
            for obj in arr {
                if let Some(val) = obj.get(key).and_then(|v| v.as_str()) {
                    return Some(val.to_string());
                }
            }
        } else if let Some(val) = json.get(key).and_then(|v| v.as_str()) {
            return Some(val.to_string());
        }
        None
    }
}
