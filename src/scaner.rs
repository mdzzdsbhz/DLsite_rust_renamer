use std::path::{Path, PathBuf};
use regex::Regex;
use walkdir::WalkDir;

/// RJ 编号扫描器接口
pub trait Scaner {
    fn scan(&self, root: &Path) -> Vec<(String, PathBuf)>;
}

/// 实现，带递归深度的 RJ 编号扫描器
pub struct ScanerImpl {
    depth: usize,
    pattern: Regex,
}

impl ScanerImpl {
    pub fn new(depth: usize) -> Self {
        Self {
            depth,
            pattern: Regex::new(r"RJ\d{6,}").unwrap(),
        }
    }
}

impl Scaner for ScanerImpl {
    fn scan(&self, root: &Path) -> Vec<(String, PathBuf)> {
        let mut results = vec![];
        for entry in WalkDir::new(root)
            .max_depth(self.depth)
            .min_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_dir())
        {
            let fname = entry.file_name().to_string_lossy();
            if let Some(mat) = self.pattern.find(&fname) {
                let rjcode = mat.as_str().to_string();
                results.push((rjcode, entry.path().to_path_buf()));
            }
        }
        results
    }
}
