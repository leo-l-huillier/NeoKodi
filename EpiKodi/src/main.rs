#![allow(non_snake_case)]

mod constants;
mod threading;
mod media;
mod database;
mod library;
mod scan;
mod gui;

// 👇 On importe ton style
use crate::gui::style::GLOBAL_STYLE;
use crate::constants::MEDIA_ROOT;
use threading::media_thread::launch_media_thread;
use threading::command::{Command, Event};
use gui::route::Route;
use crate::media::data::{MediaInfo};
use std::sync::mpsc;
use std::time::Duration;
use std::cell::RefCell;
use std::rc::Rc;
use dioxus::prelude::*;
use dioxus::desktop::{Config, WindowBuilder};
use dioxus_router::prelude::*;
use warp::Filter;
use std::thread;

fn main() {
    unsafe {
        std::env::set_var(
            "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", 
            "--disable-web-security --allow-file-access-from-files --allow-running-insecure-content --autoplay-policy=no-user-gesture-required"
        );
    }

    // SERVEUR
    thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let media_folder = warp::fs::dir(MEDIA_ROOT);
            let cors = warp::cors().allow_any_origin().allow_methods(vec!["GET"]);
            let routes = media_folder.with(cors);
            warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
        });
    });

    // FENÊTRE
    let window = WindowBuilder::new()
        .with_title("NeoKodi")
        .with_resizable(true)
        .with_maximized(true); 

    let config = Config::new()
        .with_window(window)
        // Injection du CSS depuis style.rs
        .with_custom_head(format!("<style>{}</style>", GLOBAL_STYLE))
        .with_disable_context_menu(false);

    LaunchBuilder::desktop().with_cfg(config).launch(App);
}

struct Backend {
    tx: mpsc::Sender<Command>,
    rx: RefCell<Option<mpsc::Receiver<Event>>>,
}

fn App() -> Element {
    let backend_channels = use_hook(|| {
        let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();
        let (evt_tx, evt_rx) = mpsc::channel::<Event>();
        launch_media_thread(cmd_rx, evt_tx);
        Rc::new(Backend { tx: cmd_tx, rx: RefCell::new(Some(evt_rx)) })
    });

    use_context_provider(|| backend_channels.tx.clone());
    use_context_provider(|| Signal::new(Option::<(String, String)>::None));
    let mut media_list = use_context_provider(|| Signal::new(Vec::<MediaInfo>::new()));

    use_coroutine(move |_: UnboundedReceiver<()>| {
        let backend = backend_channels.clone();
        async move {
            let mut rx_opt = backend.rx.borrow_mut();
            if let Some(rx) = rx_opt.take() {
                drop(rx_opt);
                loop {
                    if let Ok(msg) = rx.try_recv() {
                        match msg {
                            Event::MediaList(list) => { media_list.set(list); }
                            _ => {} 
                        }
                    }
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }
        }
    });

    // 👇 C'EST ICI LE SECRET : AUCUN HTML, JUSTE LE ROUTEUR
    // Si tu mets des div ou une sidebar ici, elle apparaîtra en double.
    rsx! { Router::<Route> {} }
}