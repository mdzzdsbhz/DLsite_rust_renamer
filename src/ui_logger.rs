use std::sync::{Arc, Mutex};
use log::{Record, Level, Metadata, SetLoggerError, LevelFilter};
use once_cell::sync::OnceCell;

/// 全局日志记录器（egui UI 可读取）
static LOGGER: UiLogger = UiLogger;
static LOGS: OnceCell<Arc<Mutex<Vec<String>>>> = OnceCell::new();

pub fn init(logs: Arc<Mutex<Vec<String>>>) -> Result<(), SetLoggerError> {
    LOGS.set(logs).ok();
    log::set_logger(&LOGGER)?;
    log::set_max_level(LevelFilter::Info); // 你也可以改成 Debug
    Ok(())
}

struct UiLogger;

impl log::Log for UiLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            if let Some(logs) = LOGS.get() {
                if let Ok(mut vec) = logs.lock() {
                    vec.push(format!("[{}] {}", record.level(), record.args()));
                }
            }
        }
    }

    fn flush(&self) {}
}
