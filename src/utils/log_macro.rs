// src/utils/log_macro.rs 或你自定义模块路径
use std::any::Any;
use std::sync::Mutex;

#[macro_export]
macro_rules! log_to_ui {
    // 带 logs 参数
    ($logs:expr, $($arg:tt)+) => {{
        let log_ref = &*$logs;
        let any_ref = log_ref as &dyn std::any::Any;

        if let Some(logs_mutex) = $crate::utils::log_macro::as_mutex(any_ref) {
            if let Ok(mut logs) = logs_mutex.lock() {
                logs.push(format!($($arg)+));
            }
        } else {
            println!($($arg)+);
        }
    }};

    // 无 logs 参数，直接输出
    ($($arg:tt)+) => {{
        println!($($arg)+);
    }};
}

/// 安全地尝试将 Any 引用转为 Mutex<Vec<String>> 引用
pub fn as_mutex<'a>(t: &'a dyn Any) -> Option<&'a Mutex<Vec<String>>> {
    t.downcast_ref::<Mutex<Vec<String>>>()
}
