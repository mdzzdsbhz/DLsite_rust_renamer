use std::path::PathBuf;
use eframe::egui::{self, CentralPanel, ScrollArea, TextEdit, TopBottomPanel};
use crate::config::Config;
use crate::renamer::Renamer;
use crate::scaner::{Scaner, ScanerImpl};
use crate::fonts;
use crate::dlsite_scraper;
use std::sync::{Arc, Mutex};
use crate::dlsite_scraper::DlsiteScraper;

pub struct MyApp {
    config_path: PathBuf,
    config: Config,
    folder_path: Option<PathBuf>,
    scan_result: Vec<(String, PathBuf)>,
    logs: Arc<Mutex<Vec<String>>>,
}

impl MyApp {
    pub fn new(config_path: PathBuf) -> Self {
        let config = Config::load(config_path.to_str().unwrap())
            .unwrap_or_else(|_| Config::default());
        Self {
            config_path,
            config,
            folder_path: None,
            scan_result: vec![],
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn log(&self, msg: &str) {
        if let Ok(mut logs) = self.logs.lock() {
            logs.push(msg.to_string());
        }
    }

    fn auto_scan_and_rename(&mut self) {
        if let Some(ref path) = self.folder_path {
            let mut target_dirs = Vec::new();
            let file_name = path.file_name().unwrap().to_string_lossy();

            if file_name.starts_with("RJ") {
                target_dirs.push((file_name.to_string(), path.clone()));
            } else {
                let scaner = ScanerImpl::new(self.config.scan_depth as usize);
                target_dirs = scaner.scan(path);
            }

            self.scan_result = target_dirs.clone();
            self.log(format!("🔍 共扫描到 {} 个文件夹", target_dirs.len()).as_str());

            for (rjcode, path) in target_dirs {
                let scraper = DlsiteScraper::new();
                let renamer = Renamer::new(scraper, true);
                let logs = Arc::clone(&self.logs);

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().expect("无法创建 Tokio runtime");
                    rt.block_on(async move {
                        renamer.rename_folder(&rjcode, &path).await;
                        if let Ok(mut logs) = logs.lock() {
                            logs.push(format!("✔️ {} 重命名完成", rjcode));
                        }
                    });
                });
            }
        } else {
            self.log("⚠️ 请先选择一个文件夹");
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        fonts::setup_custom_fonts(ctx);

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("选择目录").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.folder_path = Some(path.clone());
                        self.log(format!("📁 已选择目录: {}", path.display()).as_str());
                        self.auto_scan_and_rename();
                    }
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            if let Some(folder) = &self.folder_path {
                ui.label(format!("当前目录: {}", folder.display()));
            }

            ui.separator();
            ui.label("📜 日志输出：");
            ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                if let Ok(logs) = self.logs.lock() {
                    for line in logs.iter() {
                        ui.label(line);
                    }
                }
            });

            ui.separator();
            ui.label("⚙️ 当前配置预览（只读）：");
            ui.add_enabled_ui(false, |ui| {
                ui.add(TextEdit::multiline(&mut serde_json::to_string_pretty(&self.config).unwrap())
                    .desired_rows(10));
            });
        });
    }
}
