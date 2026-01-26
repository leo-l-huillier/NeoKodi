#![allow(non_snake_case)]

mod constants;
mod threading;
mod media;
mod database;
mod library;
mod scan;
mod gui;
mod config;

use crate::gui::style::GLOBAL_STYLE;
use config::AppConfig;
use threading::media_thread::launch_media_thread;
use threading::command::{Command, Event};
use gui::route::Route;
use crate::media::data::{MediaInfo, MediaType}; 
use std::sync::mpsc;
use std::path::PathBuf; 
use std::time::Duration;
use std::cell::RefCell;
use std::rc::Rc;
use dioxus::prelude::*;
use dioxus::desktop::{Config, WindowBuilder};
use dioxus_router::prelude::*;
use warp::Filter;
use std::thread;
use tokio::sync::broadcast;
use std::sync::OnceLock;

static RELOAD_SIGNAL: OnceLock<broadcast::Sender<()>> = OnceLock::new();

struct Backend {
    tx: mpsc::Sender<Command>,
    rx: RefCell<Option<mpsc::Receiver<Event>>>,
}

fn main() {
    unsafe {
        std::env::set_var("RUST_LOG", "warp=info");
        std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", "--disable-web-security --allow-file-access-from-files --allow-running-insecure-content --autoplay-policy=no-user-gesture-required");
    }

    let (reload_tx, _) = broadcast::channel::<()>(16); // Augmenté à 16 par sécurité
    let _ = RELOAD_SIGNAL.set(reload_tx.clone());

    let reload_tx_server = reload_tx.clone(); 

    // --- LE THREAD SERVEUR ROBUSTE ---
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            loop {
                let app_config = AppConfig::load();
                let server_root = app_config.media_path.clone();
                
                println!("\n========================================");
                println!("🌍 SERVEUR : Démarrage sur {}", server_root);
                println!("========================================\n");

                let mut rx = reload_tx_server.subscribe();

                let media_route = warp::path("media").and(warp::fs::dir(server_root));
                let cors = warp::cors().allow_any_origin().allow_methods(vec!["GET", "HEAD"]);
                let routes = media_route.with(cors); 

                // On lance le serveur
                let (_addr, server) = warp::serve(routes)
                    .bind_with_graceful_shutdown(([127, 0, 0, 1], 3030), async move {
                        // On attend le signal
                        let _ = rx.recv().await;
                        println!("\n🛑 SERVEUR : Signal d'arrêt reçu ! Fermeture en cours...\n");
                    });

                // On attend que le serveur s'arrête proprement
                server.await;
                
                println!("⏳ SERVEUR : Port 3030 en cours de libération...");
                // 👇 ON AUGMENTE LE DÉLAI DE SÉCURITÉ (1 seconde)
                // Windows met parfois du temps à libérer le port TCP.
                tokio::time::sleep(Duration::from_millis(1000)).await;
                println!("✅ SERVEUR : Prêt à redémarrer.");
            }
        });
    });

    let window = WindowBuilder::new().with_title("NeoKodi").with_resizable(true).with_maximized(true);
    let config_dioxus = Config::new().with_window(window).with_custom_head(format!("<style>{}</style>", GLOBAL_STYLE)).with_disable_context_menu(false);

    LaunchBuilder::desktop().with_cfg(config_dioxus).launch(App);
}

fn App() -> Element {
    let backend_channels = use_hook(|| {
        let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();
        let (evt_tx, evt_rx) = mpsc::channel::<Event>();
        
        launch_media_thread(cmd_rx, evt_tx);

        let config = AppConfig::load();
        let root_path = PathBuf::from(config.media_path);
        println!("🚀 UI : Démarrage sur {:?}", root_path);
        
        // 👇 CHANGEMENT ICI :
        // On n'utilise PAS ChangeLibraryPath au démarrage (car ça vide la DB !).
        // On utilise AddSource pour s'assurer que le dossier est bien surveillé.
        let _ = cmd_tx.send(Command::AddSource(root_path.clone(), MediaType::Video));
        let _ = cmd_tx.send(Command::AddSource(root_path.clone(), MediaType::Audio));
        let _ = cmd_tx.send(Command::AddSource(root_path, MediaType::Image));

        // 👇 TRES IMPORTANT :
        // On demande tout de suite au backend : "Envoie-moi ce que tu as déjà en base de données !"
        // Comme ça, l'affichage est instantané, même si le scan n'est pas fini.
        let _ = cmd_tx.send(Command::GetAllMedia());

        Rc::new(Backend { tx: cmd_tx, rx: RefCell::new(Some(evt_rx)) })
    });

    use_context_provider(|| backend_channels.tx.clone());
    use_context_provider(|| Signal::new(Option::<(String, String)>::None));
    let mut media_list = use_context_provider(|| Signal::new(Vec::<MediaInfo>::new()));
    
    // Récupération sécurisée du signal
    if let Some(tx) = RELOAD_SIGNAL.get() {
        use_context_provider(|| tx.clone());
    } else {
        println!("⚠️ Signal de reload non initialisé (Normal au premier rendu)");
    }
    
    let current_config = AppConfig::load();
    use_context_provider(|| Signal::new(current_config.media_path));

    use_coroutine(|_: UnboundedReceiver<()>| {
        let backend = backend_channels.clone();
        async move {
            let mut rx_opt = backend.rx.borrow_mut();
            if let Some(rx) = rx_opt.take() {
                drop(rx_opt);
                loop {
                    if let Ok(msg) = rx.try_recv() {
                        if let Event::MediaList(list) = msg { 
                            // On ne log que si la liste n'est pas vide pour éviter le spam
                            if !list.is_empty() {
                                println!("📦 UI REÇUE : {} médias", list.len());
                            }
                            media_list.set(list); 
                        }
                    }
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }
        }
    });

    rsx! { Router::<Route> {} }
}