use dioxus::prelude::*;
use super::route::Route;
use crate::threading::command::Command;
use crate::media::data::{MediaType, MediaInfo};
use dioxus::events::MouseEvent;

#[component]
pub fn Home() -> Element {
    let cmd_tx = use_context::<std::sync::mpsc::Sender<Command>>();
    
    let list_signal = use_context::<Signal<Vec<MediaInfo>>>();
    
    let mut current_image_signal = use_context::<Signal<Option<String>>>();
    let current_image = current_image_signal();

    let tx_for_init = cmd_tx.clone();
    use_hook(move || {
        tx_for_init.send(Command::GetAllMedia()).unwrap();
    });

    rsx! {
        div { 
            class: "container",
            style: "display: flex; height: 100vh; color: white;",
            div { 
                style: "width: 300px; background: #222; overflow-y: auto; padding: 10px; border-right: 1px solid #444;",
                h2 { "Ma Bibliothèque" }
                div { 
                    class: "list",
                    for item in list_signal() {
                        div { 
                            style: "padding: 10px; border-bottom: 1px solid #333; cursor: pointer; display: flex; align-items: center;",
                            onclick: {
                                let item_id = item.id;
                                let item_path = item.path.clone();
                                let item_type = item.media_type.clone();
                                let tx_click = cmd_tx.clone();
                                move |_: MouseEvent| {
                                    println!("Click sur ID: {}", item_id);
                                    
                                    if item_type == MediaType::Image {
                                        current_image_signal.set(Some(item_path.clone()));
                                    }
                                    
                                    tx_click.send(Command::Play(item_id)).unwrap();
                                }
                            },
                            
                            span { style: "margin-right: 10px;",
                                match item.media_type {
                                    MediaType::Image => "🖼️",
                                    MediaType::Audio => "🎵",
                                    MediaType::Video => "🎬",
                                }
                            }
                            "{item.title.as_deref().unwrap_or(&item.path)}"
                        }
                    }
                }
            }

            div { 
                style: "flex: 1; padding: 20px; display: flex; justify-content: center; align-items: center; background: #111;",
                
                if let Some(path) = current_image {
                    img {
                        src: "{path}",
                        style: "max-width: 100%; max-height: 90vh; box-shadow: 0 0 20px black;"
                    }
                } else {
                    div { 
                        style: "text-align: center; color: #666;",
                        h1 { "NeoKodi" }
                        p { "Sélectionne un média à gauche pour commencer" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn TV() -> Element {
    rsx! { h1 { "Ma TV" }, p { "Liste des chaînes TV ici..." } }
}

#[component]
pub fn Films() -> Element {
    rsx! {
        h1 { "Mes Films" }
        div { class: "media-grid",
            div { class: "media-card", "Matrix" }
            div { class: "media-card", "Inception" }
            div { class: "media-card", "Interstellar" }
        }
    }
}

#[component]
pub fn Series() -> Element {
    rsx! { 
        h1 { "Mes Séries" }
        div { class: "media-grid",
            div { class: "media-card", "Breaking Bad" }
            div { class: "media-card", "Game of Thrones" }
            div { class: "media-card", "Stranger Things" }
        }
    }
}

#[component]
pub fn Music() -> Element {
    rsx! {
        h1 { "Ma Musique" }
        
        div { class: "audio-list",
            
            div { class: "audio-row",
                div { class: "audio-icon", "🎵" }
                div { class: "audio-info",
                    span { class: "audio-title", "Billie Jean" }
                    span { class: "audio-artist", "Michael Jackson" }
                }
                div { class: "audio-duration", "4:54" }
            }

            div { class: "audio-row",
                div { class: "audio-icon", "🎸" }
                div { class: "audio-info",
                    span { class: "audio-title", "Bohemian Rhapsody" }
                    span { class: "audio-artist", "Queen" }
                }
                div { class: "audio-duration", "5:55" }
            }

            div { class: "audio-row",
                div { class: "audio-icon", "🎹" }
                div { class: "audio-info",
                    span { class: "audio-title", "Imagine" }
                    span { class: "audio-artist", "John Lennon" }
                }
                div { class: "audio-duration", "3:01" }
            }
        }
    }
}

#[component]
pub fn Images() -> Element {
    rsx! { h1 { "Mes Images" }, p { "Galerie d'images..." } }
}

#[component]
pub fn Addons() -> Element {
    rsx! { h1 { "Add-ons" }, p { "Gérer les add-ons ici..." } }
}

#[component]
pub fn Settings() -> Element {
    rsx! { h1 { "Paramètres" }, p { "Configurer l'application..." } }
}

#[component]
pub fn PageNotFound(route: Vec<String>) -> Element {
    rsx! {
        div { 
            style: "padding: 50px; color: white; text-align: center;",
            h1 { "Oups ! Page introuvable" }
            p { "La route que tu cherches n'existe pas : {route:?}" }
            Link { to: Route::Home {}, "Retour à l'accueil" }
        }
    }
}