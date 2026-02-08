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

use crate::gui::style::GLOBAL_STYLE;
use crate::config::AppConfig;

use crate::gui::init::{App, RELOAD_SIGNAL}; 

use std::thread;
use std::time::Duration;

use dioxus::prelude::*;
use dioxus::desktop::{Config, WindowBuilder};

use warp::Filter;
use tokio::sync::broadcast;

fn main() {
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
                println!("🌍 SERVEUR : Démarrage sur {}", server_root);

                let mut rx = reload_tx_server.subscribe();
                let media_route = warp::path("media").and(warp::fs::dir(server_root));
                let cors = warp::cors().allow_any_origin().allow_methods(vec!["GET", "HEAD"]);
                
                let (_addr, server) = warp::serve(media_route.with(cors))
                    .bind_with_graceful_shutdown(([127, 0, 0, 1], 3030), async move {
                        let _ = rx.recv().await;
                    });

                server.await;
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        });
    });

    // --- FENÊTRE ---
    let window = WindowBuilder::new()
        .with_title("NeoKodi")
        .with_resizable(true)
        .with_maximized(true);

    let config_dioxus = Config::new()
        .with_window(window)
        .with_custom_head(format!(
            r#"
            <style>{}</style>
            <script src="https://cdn.jsdelivr.net/npm/hls.js@1"></script>
            "#, 
            GLOBAL_STYLE
        ))
        .with_disable_context_menu(false);

    LaunchBuilder::new()
        .with_cfg(config_dioxus)
        .launch(App);
}