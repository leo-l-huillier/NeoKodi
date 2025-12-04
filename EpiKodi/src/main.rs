

mod media;
mod database;
mod threading;

use threading::media_thread::launch_media_thread;
use threading::command::Command;
use threading::command::Event;

use std::time::Duration;
use std::thread::sleep;
use std::sync::mpsc;
const GLOBAL_STYLE: &str = r#"
    body {
        margin: 0;
        background-color: #0d0d0d;
        color: white;
        font-family: Arial, sans-serif;
    }

    .container {
        display: flex;
        height: 100vh;
    }

    .sidebar {
        width: 250px;
        background: #1a1a1a;
        display: flex;
        flex-direction: column;
        padding: 20px;
        gap: 20px;
    }

    .sidebar button {
        background: #2b2b2b;
        border: none;
        padding: 12px;
        color: white;
        font-size: 18px;
        border-radius: 6px;
        cursor: pointer;
    }

    .sidebar button:hover {
        background: #3a3a3a;
    }

    .content {
        flex: 1;
        padding: 30px;
        font-size: 24px;
    }
    "#;
/*
fn main() {

    // ========== MEDIA THREADING ===========

    let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();
    let (evt_tx, evt_rx) = mpsc::channel::<Event>();
    launch_media_thread(cmd_rx, evt_tx);

    // ========== GUI ===========

    let mut i = 0;
    loop {
        // Simulate GUI

        println!("GUI working...");
        sleep(Duration::from_secs(1));
        if i==2 {
            cmd_tx.send(Command::Info(3)).unwrap();
            if let Ok(event) = evt_rx.try_recv() {
                match event {
                    Event::Finished(id) => println!("Media finished item {id}"),
                    Event::NowPlaying(msg) => println!("MEDIA says: {msg}"),
                    Event::Data(info) => println!("MEDIA info: {info}"),
            }
        }
        }
        if i==5 {
            cmd_tx.send(Command::Pause(3)).unwrap();
        }
        if i==8 {
            cmd_tx.send(Command::Resume(3)).unwrap();
        }
        if i==10 {
            break;
        }
        i += 1;
    }
}
*/

use dioxus::prelude::*;
use std::thread::Scope;
use dioxus_desktop::launch;
use dioxus_router::prelude::*;

fn main() {
    launch(Home);
}

pub fn Home() -> Element {
    rsx! {
        style { "{GLOBAL_STYLE}" }

        div { class: "container",
            // -------- MENU LATERAL --------
            nav { class: "sidebar",
                button { "Films" }
                button { "Series" }
                button { "Musique" }
                button { "Images" }
                button { "Add-ons" }
                button { "Paramètres" }
            }

            // -------- ZONE CONTENU --------
            main { class: "content",
                h1 { "Welcome to NeoKodi" }
                p { "Sélectionne un élément dans le menu." }
            }
        }
    }
}



