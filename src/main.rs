mod config;
mod renamer;
mod scaner;
mod dlsite_scraper;
mod ui;
mod fonts;
mod work_metadata;
mod dlsite;
mod cached_scraper_db;
mod ui_logger;

use std::panic;
use std::fs::OpenOptions;
use std::io::Write;

use std::path::PathBuf;
use eframe::NativeOptions;
use ui::MyApp;


fn setup_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        let msg = format!("⚠️ Panic 发生了：{}\n", panic_info);
        let _ = std::fs::write("panic.log", &msg); // 写入 panic.log
        eprintln!("{}", msg); // 如果你是命令行启动，打印到控制台
    }));
}


fn main() -> eframe::Result<()> {
    setup_panic_hook(); // <<<< 加上这行
    // 默认 config.json 路径（可根据实际情况修改）
    let config_path = PathBuf::from("config.json");

    // 创建 GUI 应用
    let app = MyApp::new(config_path);
    let native_options = NativeOptions {
        ..Default::default()
    };

    // 启动 egui 桌面应用
    eframe::run_native(
        "DLsite Doujin Renamer (Rust版)",
        native_options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
}
