#![allow(non_snake_case)]

mod threading;
mod constants;
mod media;
mod database;
mod library;
mod scan;
mod gui;

use threading::media_thread::launch_media_thread;
use threading::command::{Command, Event};
use gui::route::Route;
use crate::media::data::MediaType;

use std::sync::mpsc;
use std::time::Duration;
use std::cell::RefCell;
use std::rc::Rc;
use dioxus::prelude::*;
use dioxus::desktop::{Config, WindowBuilder};
use dioxus_router::prelude::*;
use crate::media::data::MediaInfo;

fn main() {
    let config = Config::new()
        .with_window(WindowBuilder::new()
            .with_title("NeoKodi")
            .with_resizable(true)
        );

    LaunchBuilder::desktop().with_cfg(config).launch(App);
}

struct Backend {
    tx: mpsc::Sender<Command>,
    rx: RefCell<Option<mpsc::Receiver<Event>>>,
}

fn App() -> Element {
    
    let backend_channels = use_hook(|| {
        println!("--- Initialisation du Thread Média ---");
        let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();
        let (evt_tx, evt_rx) = mpsc::channel::<Event>();
        
        launch_media_thread(cmd_rx, evt_tx);

        Rc::new(Backend {
            tx: cmd_tx,
            rx: RefCell::new(Some(evt_rx)),
        })
    });

    use_context_provider(|| backend_channels.tx.clone());
    let mut current_image = use_context_provider(|| Signal::new(Option::<String>::None));
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
                            Event::MediaList(list) => {
                                println!("GUI: Liste reçue avec {} éléments", list.len());
                                media_list.set(list);
                            }
                            
                            Event::Info(info) => {
                                if info.media_type == MediaType::Image {
                                    current_image.set(Some(info.path));
                                }
                            }
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
