use dioxus::prelude::*;

use std::sync::{mpsc, OnceLock};
use tokio::sync::broadcast;

use crate::threading::media_thread::launch_media_thread;
use crate::threading::command::{Command, Event};

use crate::media::data::{MediaInfo, MediaType};
use crate::gui::pages::PluginSearchResult;
use crate::gui::route::Route;

use crate::config::AppConfig;
use std::cell::RefCell;
use std::rc::Rc;
use std::path::PathBuf;
use std::time::Duration;
use crate::iptv::parser::TVChannel;


pub static RELOAD_SIGNAL: OnceLock<broadcast::Sender<()>> = OnceLock::new();

struct Backend {
    tx: mpsc::Sender<Command>,
    rx: RefCell<Option<mpsc::Receiver<Event>>>,
}

// === COMPOSANT RACINE (DÉPLACÉ) ===
pub fn App() -> Element {
    let backend_channels = use_hook(|| {
        let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();
        let (evt_tx, evt_rx) = mpsc::channel::<Event>();
        
        launch_media_thread(cmd_rx, evt_tx);

        let config = AppConfig::load();
        let root_path = PathBuf::from(config.media_path);
        
        // Initialisation scan
        let _ = cmd_tx.send(Command::AddSource(root_path.clone(), MediaType::Video));
        let _ = cmd_tx.send(Command::AddSource(root_path.clone(), MediaType::Audio));
        let _ = cmd_tx.send(Command::AddSource(root_path, MediaType::Image));
        let _ = cmd_tx.send(Command::GetAllMedia());
        let _ = cmd_tx.send(Command::GetAllPlaylists());
        let _ = cmd_tx.send(Command::GetPluginHistory);
        
        Rc::new(Backend { tx: cmd_tx, rx: RefCell::new(Some(evt_rx)) })
    });

    use_context_provider(|| backend_channels.tx.clone());
    
    // États globaux
    let mut media_list = use_context_provider(|| Signal::new(Vec::<MediaInfo>::new()));

    // Pour recevoir le contenu d'une playlist
    let mut playlists = use_context_provider(|| Signal::new(Vec::<(i64, String)>::new()));
    let mut loaded_ids = use_context_provider(|| Signal::new(Vec::<i64>::new()));
    
    // Plugin Result
    let mut plugin_result = use_context_provider(|| Signal::new(Vec::<String>::new()));
    
    let mut root_path_signal = use_context_provider(|| Signal::new(String::new()));
    let current_config = AppConfig::load();
    root_path_signal.set(current_config.media_path);

    if let Some(tx) = RELOAD_SIGNAL.get() {
        use_context_provider(|| tx.clone());
    }

    let mut iptv_loading = use_context_provider(|| Signal::new(false)); 
    let mut iptv_channels = use_context_provider(|| Signal::new(Vec::<TVChannel>::new()));

    // BOUCLE D'ÉVÉNEMENTS
    use_coroutine(|_: UnboundedReceiver<()>| {
        let backend = backend_channels.clone();
        async move {
            let mut rx_opt = backend.rx.borrow_mut();
            if let Some(rx) = rx_opt.take() {
                drop(rx_opt);
                loop {
                    while let Ok(msg) = rx.try_recv() {
                        match msg {
                            Event::MediaList(list) => { media_list.set(list); },

                            Event::PlaylistList(list) => { playlists.set(list); },
                            Event::IDList(ids) => { loaded_ids.set(ids); },
                            
                            Event::PluginDataReceived(info) => {
                                plugin_result.write().insert(0, info);
                            },

                            Event::NowPlaying(id) => println!("▶️ Lecture ID: {}", id),
                            Event::Info(info) => println!("ℹ️ Info: {:?}", info.title),
                            Event::M3UList(channels) => {
                                iptv_channels.set(channels);
                                iptv_loading.set(false);
                            },
                            _ => {} 
                        }
                    }
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }
        }
    });

    rsx! { Router::<Route> {} }
}