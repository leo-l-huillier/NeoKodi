use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

fn history_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("EpiKodi")
        .join("music_download_history.txt")
}

pub fn load() -> HashSet<String> {
    let path = history_path();
    if !path.exists() {
        return HashSet::new();
    }
    fs::read_to_string(&path)
        .unwrap_or_default()
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

pub fn add(url: &str) {
    let path = history_path();
    fs::create_dir_all(path.parent().unwrap()).ok();
    let mut contents = fs::read_to_string(&path).unwrap_or_default();
    contents.push_str(url);
    contents.push('\n');
    fs::write(path, contents).ok();
}

pub fn contains(url: &str) -> bool {
    load().contains(url)
}

pub fn clear() {
    let path = history_path();
    fs::write(path, "").ok();
}