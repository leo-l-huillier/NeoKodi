pub mod downloader;
pub mod history;

use crate::music_download::downloader::{download_all, DownloadTask};
use crate::constants::{MUSIC_OUTPUT_DIR, MUSIC_FORMAT, MUSIC_MAX_PARALLEL_DOWNLOADS};
use crate::constants::LOG_FILE;
use crate::logger::logger::Logger;

pub struct MusicDownloader {
}

impl MusicDownloader {
    pub fn new() -> Self {
        Self {
        }
    }

    /// Download one or more URLs, grouped into an optional album folder
    pub async fn download(&self, urls: Vec<String>, album: Option<String>) {
        let logger = Logger::new(LOG_FILE);
        logger.debug(&format!(
            "MusicDownloader: starting {} download(s)", urls.len()
        ));

        let tasks: Vec<DownloadTask> = urls
            .into_iter()
            .map(|url| DownloadTask {
                url,
                output: MUSIC_OUTPUT_DIR.to_string(),
                format: MUSIC_FORMAT.to_string(),
                album: album.clone(),
            })
            .collect();

        download_all(tasks, MUSIC_MAX_PARALLEL_DOWNLOADS).await;
    }

    /// Search YouTube and download top N results
    pub async fn search_and_download(
        &self,
        query: &str,
        count: u32,
        album: Option<String>,
    ) {
        let logger = Logger::new(LOG_FILE);
        logger.debug(&format!("MusicDownloader: searching '{}'", query));

        let target = format!("ytsearch{}:{}", count, query);

        let tasks = vec![DownloadTask {
            url: target,
            output: MUSIC_OUTPUT_DIR.to_string(),
            format: MUSIC_FORMAT.to_string(),
            album,
        }];

        download_all(tasks, MUSIC_MAX_PARALLEL_DOWNLOADS).await;
    }

    /// Check if a URL was already downloaded
    pub fn already_downloaded(&self, url: &str) -> bool {
        history::contains(url)
    }

    /// Get full download history
    pub fn get_history(&self) -> Vec<String> {
        history::load().into_iter().collect()
    }

    /// Clear download history
    pub fn clear_history(&self) {
        history::clear();
    }
}