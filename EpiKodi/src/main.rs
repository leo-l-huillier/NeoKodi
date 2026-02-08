#![allow(non_snake_case)]

mod constants;
mod threading;
mod database;
mod media;
mod library;
mod scan;
mod config;
mod gui;
mod iptv;
mod logger; 
mod plugin;

use crate::logger::logger::Logger;
use crate::constants::LOG_FILE;

use crate::gui::style::GLOBAL_STYLE;
use crate::config::AppConfig;
use crate::gui::init::{App, RELOAD_SIGNAL}; 

use std::thread;
use std::time::Duration;
use std::path::Path;

use dioxus::prelude::*;
use dioxus::desktop::{Config, WindowBuilder};

use warp::Filter;
use tokio::sync::broadcast;

fn main() {

    let logger = Logger::new(LOG_FILE);

    unsafe {
        std::env::set_var("RUST_LOG", "warp=info");
        std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", "--disable-web-security --allow-file-access-from-files --allow-running-insecure-content --autoplay-policy=no-user-gesture-required");
    }

    let (reload_tx, _) = broadcast::channel::<()>(16);
    let _ = RELOAD_SIGNAL.set(reload_tx.clone());
    let reload_tx_server = reload_tx.clone(); 

    // --- THREAD SERVEUR ---
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            loop {
                let app_config = AppConfig::load();
                let server_root = app_config.media_path.clone();
                logger.info(&format!("🌍 SERVEUR : Démarrage sur {}", server_root));

                let mut rx = reload_tx_server.subscribe();
                let base_route = warp::path("media")
                    .and(warp::fs::dir(app_config.media_path.clone()));
                let mut drives_filter: Option<warp::filters::BoxedFilter<(warp::fs::File,)>> = None;

                for letter in b'a'..=b'z' {
                    let char_letter = letter as char;
                    let drive_path = format!("{}:\\", char_letter);

                    if Path::new(&drive_path).exists() {
                        let this_drive = warp::path("drives")
                            .and(warp::path(char_letter.to_string()))
                            .and(warp::fs::dir(drive_path));

                        drives_filter = match drives_filter {
                            Some(prev) => Some(prev.or(this_drive).unify().boxed()),
                            None => Some(this_drive.boxed()),
                        };
                    }
                }

                let final_drives_route = drives_filter.unwrap_or_else(|| {
                    warp::path("impossible_fallback_route")
                        .and(warp::fs::dir("."))
                        .boxed()
                });
                let cors = warp::cors().allow_any_origin().allow_methods(vec!["GET", "HEAD"]);
                let routes = base_route
                    .or(final_drives_route)
                    .unify() 
                    .with(cors);
                let (_addr, server) = warp::serve(routes)
                    .bind_with_graceful_shutdown(([127, 0, 0, 1], 3030), async move {
                        let _ = rx.recv().await;
                    });

                server.await;
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        });
    });

    // --- FENÊTRE ---
    let window = WindowBuilder::new().with_title("NeoKodi").with_resizable(true).with_maximized(true);
    let config_dioxus = Config::new().with_window(window).with_custom_head(format!("<style>{}</style>", GLOBAL_STYLE));

    LaunchBuilder::new().with_cfg(config_dioxus).launch(App);
}