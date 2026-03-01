mod config;
mod downloader;
mod history;

use clap::{Parser, Subcommand};
use colored::Colorize;
use downloader::DownloadTask;

#[derive(Parser)]
#[command(name = "music-dl", about = "A simple async music downloader")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Download one or more URLs
    Get {
        #[arg(required = true)]
        urls: Vec<String>,

        #[arg(short, long)]
        output: Option<String>,

        #[arg(short, long)]
        format: Option<String>,

        /// Put all downloads into a folder with this album name
        #[arg(short, long)]
        album: Option<String>,
    },

    /// Search YouTube and download the top result
    Search {
        /// Search query (e.g: lofi hip hop)
        #[arg(required = true)]
        query: Vec<String>,

        /// How many results to download
        #[arg(short, long, default_value = "1")]
        count: u32,

        #[arg(short, long)]
        output: Option<String>,

        #[arg(short, long)]
        format: Option<String>,
    },

    /// Show and edit your config
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Show download history
    History,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Print current config
    Show,
    /// Set a config value (e.g: config set format flac)
    Set {
        key: String,
        value: String,
    },
}

// #[tokio::main] turns our main function into an async function
// managed by the tokio runtime
#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let cfg = config::load();

    println!("{}", "🎵 music-dl".cyan().bold());

    match cli.command {
        Commands::Get { urls, output, format, album } => {
            let output = output.unwrap_or(cfg.output_dir.clone());
            let format = format.unwrap_or(cfg.format.clone());

            std::fs::create_dir_all(&output).expect("Could not create output dir");

            let tasks: Vec<DownloadTask> = urls
                    .into_iter()
                    .map(|url| DownloadTask {
                        url,
                        output: output.clone(),
                        format: format.clone(),
                        album: album.clone(),
                    })
                    .collect();

            downloader::download_all(tasks, cfg.max_parallel_downloads).await;
        }

        Commands::Search { query, count, output, format } => {
            let output = output.unwrap_or(cfg.output_dir.clone());
            let format = format.unwrap_or(cfg.format.clone());
            let query_str = query.join(" ");

            std::fs::create_dir_all(&output).expect("Could not create output dir");

            // ytsearchN:query tells yt-dlp to search YouTube for N results
            let target = format!("ytsearch{}:{}", count, query_str);

            println!("  Searching: {}", query_str.yellow());

            let tasks = vec![DownloadTask {
                url: target,
                output,
                format,
                album: None, // search doesn't support album grouping yet
            }];

            downloader::download_all(tasks, cfg.max_parallel_downloads).await;
        }

        Commands::Config { action } => match action {
            ConfigAction::Show => {
                let path = config::config_path();
                println!("  Config file: {}", path.display().to_string().yellow());
                println!("  output_dir:            {}", cfg.output_dir.yellow());
                println!("  format:                {}", cfg.format.yellow());
                println!("  max_parallel_downloads:{}", cfg.max_parallel_downloads.to_string().yellow());
            }
            ConfigAction::Set { key, value } => {
                let mut cfg = cfg;
                match key.as_str() {
                    "output_dir" => cfg.output_dir = value,
                    "format"     => cfg.format = value,
                    "max_parallel_downloads" => {
                        cfg.max_parallel_downloads = value.parse()
                            .expect("max_parallel_downloads must be a number");
                    }
                    _ => {
                        eprintln!("{} Unknown key: {}", "Error:".red(), key);
                        eprintln!("Valid keys: output_dir, format, max_parallel_downloads");
                        return;
                    }
                }
                config::save(&cfg);
                println!("{}", "✅ Config saved!".green());
            }
        },

        Commands::History => {
            let history = history::load();
            if history.is_empty() {
                println!("  No downloads yet.");
            } else {
                println!("  {} downloaded URLs:", history.len());
                for url in &history {
                    println!("    - {}", url.yellow());
                }
            }
        }
    }
}