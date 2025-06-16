use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;
use regex::Regex;
use chrono::Local;

/// 默认配置结构体
#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Config {
    pub scan_depth: u32,
    pub timeout: f64,
    pub interval: f64,
    pub proxy: Option<String>,
    pub rename_template: String,
    pub date_format: String,
    pub keep_brackets: bool,
    pub remove_illegal_chars: bool,
    pub save_cover: bool,
    pub tags: Vec<String>,
    pub replace: Vec<[String; 2]>,
    pub output_dir: Option<String>,
    pub cover_dir: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scan_depth: 3,
            timeout: 15.0,
            interval: 1.0,
            proxy: None,
            rename_template: "[rjcode] [title]".to_string(),
            date_format: "%Y-%m-%d".to_string(),
            keep_brackets: true,
            remove_illegal_chars: true,
            save_cover: true,
            tags: vec!["circle".to_string(), "cv".to_string(), "genre".to_string()],
            replace: vec![],
            output_dir: None,
            cover_dir: None,
        }
    }
}

impl Config {
    /// 从 JSON 加载配置，如果不存在则自动写入默认配置
    pub fn load(path: &str) -> Result<Self, String> {
        match fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).map_err(|e| e.to_string()),
            Err(_) => {
                let default_config = Config::default();
                default_config.save(path)?;
                Ok(default_config)
            }
        }
    }

    /// 保存配置到 JSON 文件
    pub fn save(&self, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::File::create(path)
            .and_then(|mut file| file.write_all(json.as_bytes()))
            .map_err(|e| e.to_string())
    }

    /// 验证配置项合法性，返回错误信息列表
    pub fn verify(&self) -> Vec<String> {
        let mut errors = vec![];

        // 1. Proxy 格式验证
        if let Some(proxy) = &self.proxy {
            let proxy_re = Regex::new(r"^(http|https|socks5)?://?[\w\.-]+(:\d+)?$").unwrap();
            if !proxy_re.is_match(proxy) {
                errors.push(format!("代理格式不合法: {}", proxy));
            }
        }

        // 2. rename_template 必须包含 [rjcode]
        if !self.rename_template.contains("[rjcode]") {
            errors.push("rename_template 必须包含 [rjcode]".to_string());
        }

        // 3. 禁止 Windows 非法字符
        let illegal_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
        if self.rename_template.chars().any(|c| illegal_chars.contains(&c)) {
            errors.push("rename_template 中包含 Windows 非法字符".to_string());
        }

        // 4. 日期格式验证
        if let Err(_) = Local::now().format(&self.date_format).to_string().parse::<String>() {
            errors.push(format!("date_format 无法解析: {}", self.date_format));
        }

        // 5. tags 合法性验证
        let valid_tags = ["circle", "cv", "genre"];
        for tag in &self.tags {
            if !valid_tags.contains(&tag.as_str()) {
                errors.push(format!("不支持的标签: {}", tag));
            }
        }

        // 6. replace 验证：必须为 [旧, 新] 字符串数组
        for (i, pair) in self.replace.iter().enumerate() {
            if pair.len() != 2 {
                errors.push(format!("第 {} 项 replace 格式错误，必须为两个字符串", i));
            }
        }

        // 7. 路径字段合法性（如果设置了就检查是否是合法目录）
        for (field, path_opt) in &[("output_dir", &self.output_dir), ("cover_dir", &self.cover_dir)] {
            if let Some(path_str) = path_opt {
                let path = Path::new(path_str);
                if !path.exists() {
                    errors.push(format!("{} 指定的路径不存在: {}", field, path_str));
                } else if !path.is_dir() {
                    errors.push(format!("{} 不是合法目录: {}", field, path_str));
                }
            }
        }

        errors
    }
}
