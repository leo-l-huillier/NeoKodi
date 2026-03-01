use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

// Returns path to ~/.local/share/music-dl/history.txt
fn history_path() -> PathBuf {
    let base = dirs::data_local_dir()
        .expect("Could not find data directory");
    base.join("music-dl").join("history.txt")
}

// Load the set of already-downloaded URLs from disk
pub fn load() -> HashSet<String> {
    let path = history_path();

    if !path.exists() {
        return HashSet::new(); // empty set on first run
    }

    let contents = fs::read_to_string(&path)
        .expect("Could not read history file");

    // Each line is one URL
    contents.lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect() // collect into a HashSet
}

// Add a URL to the history file
pub fn add(url: &str) {
    let path = history_path();

    fs::create_dir_all(path.parent().unwrap())
        .expect("Could not create data directory");

    // Append the URL as a new line
    let mut contents = fs::read_to_string(&path).unwrap_or_default();
    contents.push_str(url);
    contents.push('\n');

    fs::write(&path, contents)
        .expect("Could not write history file");
}

pub fn contains(url: &str) -> bool {
    load().contains(url)
}