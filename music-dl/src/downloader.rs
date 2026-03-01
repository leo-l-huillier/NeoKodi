use crate::history;
use colored::Colorize;
use id3::{Tag, TagLike, Version};
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command; // tokio's async version of std::process::Command
use tokio::sync::Semaphore; // limits how many downloads run at once
use std::sync::Arc;

pub struct DownloadTask {
    pub url: String,
    pub output: String,
    pub format: String,
    pub album: Option<String>, // None means "let yt-dlp figure it out"
}

// Arc = "Atomically Reference Counted" — lets us safely share the semaphore
// across async tasks running in parallel
pub async fn download_all(tasks: Vec<DownloadTask>, max_parallel: usize) {
    // A semaphore with N permits means at most N tasks run at the same time
    let semaphore = Arc::new(Semaphore::new(max_parallel));

    // This will hold all our async task handles
    let mut handles = vec![];

    let total = tasks.len();

    for (i, task) in tasks.into_iter().enumerate() {
        let sem = Arc::clone(&semaphore);
        let index = i + 1;

        // tokio::spawn launches this block as an async task
        // It runs concurrently with other spawned tasks
        let handle = tokio::spawn(async move {
            // Acquire a permit — if all N permits are taken, this waits
            let _permit = sem.acquire().await.unwrap();
            // _permit is automatically released when it goes out of scope

            download_one(task, index, total).await;
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
}

async fn download_one(task: DownloadTask, index: usize, total: usize) {
    let prefix = format!("[{}/{}]", index, total).cyan().to_string();

    // Check history before downloading
    if history::contains(&task.url) {
        println!("{} {} {}", prefix, "⏭  Already downloaded, skipping:".yellow(), task.url);
        return;
    }

    println!("{} {} {}", prefix, "⬇  Downloading:".cyan(), task.url);

    // Use tokio's async Command so we don't block the thread while waiting
   // If the user gave us an album name, use it directly.
// Otherwise fall back to yt-dlp's %(album)s metadata.
let output_template = match &task.album {
    Some(album) => format!("{}/{}/%(title)s.%(ext)s", task.output, album),
    None        => format!("{}/%(album,Unknown Album)s/%(title)s.%(ext)s", task.output),
};

let output = Command::new("yt-dlp")
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
    .expect("Failed to run yt-dlp");

    if output.status.success() {
        // The last line of stdout is the file path (from --print after_move:filepath)
        let stdout = String::from_utf8_lossy(&output.stdout);
        let file_path = stdout.lines().last().unwrap_or("").trim();

        println!("{} {} {}", prefix, "✅ Done:".green(), file_path);

        // Tag the file with metadata
        if task.format == "mp3" {
            tag_file(file_path, &task.url, task.album.as_deref());
        }

        // Save to history so we skip it next time
        history::add(&task.url);
    } else {
        eprintln!("{} {}", prefix, "❌ Failed".red());
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
        eprintln!("  Warning: could not write tags: {}", e);
    }
}