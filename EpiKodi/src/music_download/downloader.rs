use crate::music_download::history;
use crate::constants::LOG_FILE;
use crate::logger::logger::Logger;
use id3::{Tag, TagLike, Version};
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Semaphore;

pub struct DownloadTask {
    pub url: String,
    pub output: String,
    pub format: String,
    pub album: Option<String>,
}

// At the top of downloader.rs
fn ytdlp_cmd() -> &'static str {
    if cfg!(target_os = "windows") {
        "./yt-dlp.exe"
    } else {
        "yt-dlp"
    }
}

pub async fn download_all(tasks: Vec<DownloadTask>, max_parallel: usize) {
    let semaphore = Arc::new(Semaphore::new(max_parallel));
    let mut handles = vec![];
    let total = tasks.len();

    for (i, task) in tasks.into_iter().enumerate() {
        let sem = Arc::clone(&semaphore);
        let index = i + 1;

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            download_one(task, index, total).await;
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

async fn download_one(task: DownloadTask, index: usize, total: usize) {
    let logger = Logger::new(LOG_FILE);

    if history::contains(&task.url) {
        logger.debug(&format!(
            "[{}/{}] Skipping already downloaded: {}", index, total, task.url
        ));
        return;
    }

    logger.debug(&format!(
        "[{}/{}] Downloading: {}", index, total, task.url
    ));

    let output_template = match &task.album {
        Some(album) => format!("{}/{}/%(title)s.%(ext)s", task.output, album),
        None        => format!("{}/%(album,Unknown Album)s/%(title)s.%(ext)s", task.output),
    };

    let output = Command::new(ytdlp_cmd())
        .args([
            "-x",
            "--audio-format", &task.format,
            "--audio-quality", "0",
            "--print", "after_move:filepath",
            "-o", &output_template,
            &task.url,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .await
        .expect("Failed to run yt-dlp — is it installed?");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let file_path = stdout.lines().last().unwrap_or("").trim();

        logger.debug(&format!("[{}/{}] Done: {}", index, total, file_path));

        if task.format == "mp3" {
            tag_file(file_path, &task.url, task.album.as_deref());
        }

        history::add(&task.url);
    } else {
        logger.error(&format!("[{}/{}] Failed: {}", index, total, task.url));
    }
}

fn tag_file(file_path: &str, source_url: &str, album: Option<&str>) {
    let path = Path::new(file_path);
    if !path.exists() { return; }

    let mut tag = Tag::read_from_path(path).unwrap_or_default();

    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown");

    tag.set_title(title);

    if let Some(album_name) = album {
        tag.set_album(album_name);
    }

    tag.add_comment(id3::frame::Comment {
        lang: "eng".to_string(),
        description: "source".to_string(),
        text: source_url.to_string(),
    });

    if let Err(e) = tag.write_to_path(path, Version::Id3v24) {
        let logger = Logger::new(LOG_FILE);
        logger.error(&format!("Could not write ID3 tags to {}: {}", file_path, e));
    }
}