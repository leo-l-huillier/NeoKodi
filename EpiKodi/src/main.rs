// src/main.rs

// Tes modules existants (Logique métier)
mod media;
mod database;
mod threading;

// Le nouveau module UI
mod front;

use dioxus::prelude::*;
use dioxus::desktop::{Config, WindowBuilder};
use dioxus_router::prelude::*;

// On utilise uniquement la Route définie dans le dossier front
use front::route::Route;

// Imports de tes threads (si tu les utilises ici, sinon tu peux nettoyer)
// use threading::media_thread::launch_media_thread;
// use threading::command::{Command, Event};

fn main() {
    // 1. Définir la configuration de la fenêtre
    let config = Config::new()
        .with_window(
            WindowBuilder::new()
                .with_title("NeoKodi")
                .with_resizable(true)
        );

    // 2. Lancer l'application
    LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(|| rsx! { Router::<Route> {} });
}