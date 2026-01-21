use dioxus::prelude::*;
use super::route::Route;
use crate::threading::command::Command;
use crate::media::data::{MediaType, MediaInfo};
use crate::constants::MEDIA_ROOT;
use std::fs; 
use base64::{Engine as _, engine::general_purpose};

// --- ACCUEIL ---
#[component]
pub fn Home() -> Element {
    rsx! {
        div { class: "container", style: "justify-content: center; align-items: center;",
            h1 { style: "font-size: 4rem; margin-bottom: 50px; color: #007acc; text-transform: uppercase; letter-spacing: 5px;", "NeoKodi" }
            div { style: "display: grid; grid-template-columns: repeat(3, 1fr); gap: 30px;",
                Link { to: Route::Films {}, class: "media-card", div { class: "card-icon", "🎬" } div { class: "card-text", "Films" } }
                Link { to: Route::Images {}, class: "media-card", div { class: "card-icon", "🖼️" } div { class: "card-text", "Images" } }
                Link { to: Route::Music {}, class: "media-card", div { class: "card-icon", "🎵" } div { class: "card-text", "Musique" } }
                Link { to: Route::Series {}, class: "media-card", div { class: "card-icon", "🍿" } div { class: "card-text", "Séries" } }
                Link { to: Route::TV {}, class: "media-card", div { class: "card-icon", "📺" } div { class: "card-text", "TV" } }
                Link { to: Route::Settings {}, class: "media-card", div { class: "card-icon", "⚙️" } div { class: "card-text", "Paramètres" } }
            }
        }
    }
}

// --- MUSIQUE ---
#[component]
pub fn Music() -> Element {
    let cmd_tx = use_context::<std::sync::mpsc::Sender<Command>>();
    let list_signal = use_context::<Signal<Vec<MediaInfo>>>();
    let mut current_audio = use_signal(|| Option::<String>::None);
    let tx_init = cmd_tx.clone();
    use_hook(move || { if list_signal().is_empty() { tx_init.send(Command::GetAllMedia()).unwrap(); } });

    rsx! {
        div { class: "container",
            if let Some(path) = current_audio() {
                div { style: "flex: 1; display: flex; flex-direction: column; justify-content: center; align-items: center;",
                    div { style: "font-size: 5rem; margin-bottom: 20px;", "🎵" }
                    h2 { "Lecture en cours" }
                    div { style: "color: #007acc; font-family: monospace; margin-bottom: 30px;", "{path}" }
                    audio { controls: true, autoplay: true, style: "width: 80%; max-width: 600px;",
                        onended: move |_| current_audio.set(None),
                        src: {
                            let clean = path.replace("\\", "/").replace(" ", "%20");
                            format!("http://127.0.0.1:3030/{}", clean.split("/media/").last().unwrap_or(&clean))
                        }
                    }
                    button { class: "btn-nav", style: "margin-top: 30px;", onclick: move |_| current_audio.set(None), "❌ Arrêter" }
                }
            } else {
                div { class: "top-bar", Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" }, div { class: "page-title", "Musique" } }
                div { class: "audio-list",
                    for item in list_signal().iter().filter(|i| i.media_type == MediaType::Audio) {
                        div { class: "audio-row",
                            onclick: { let p=item.path.clone(); let i=item.id; let tx=cmd_tx.clone(); move |_| { current_audio.set(Some(p.clone())); tx.send(Command::Play(i)).unwrap(); } },
                            div { class: "audio-icon", "🎵" }
                            div { class: "audio-info", div { class: "audio-title", "{item.title.as_deref().unwrap_or(&item.path)}" } div { class: "audio-artist", "Artiste inconnu" } }
                        }
                    }
                }
            }
        }
    }
}

// --- FILMS ---
#[component]
pub fn Films() -> Element {
    let cmd_tx = use_context::<std::sync::mpsc::Sender<Command>>();
    let list_signal = use_context::<Signal<Vec<MediaInfo>>>();
    let mut current_video = use_signal(|| Option::<String>::None);
    let tx_init = cmd_tx.clone(); 
    use_hook(move || { if list_signal().is_empty() { tx_init.send(Command::GetAllMedia()).unwrap(); } });

    rsx! {
        div { class: "container",
            if let Some(path) = current_video() {
                // LECTEUR VIDEO
                // On met un fond noir qui prend TOUT l'écran
                // MAIS on centre le contenu (la vidéo)
                div { style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: black; z-index: 999; display: flex; align-items: center; justify-content: center;",
                    
                    button { class: "btn-nav", style: "position: absolute; top: 20px; left: 20px; z-index: 1000;", onclick: move |_| current_video.set(None), "⬅ Retour" }
                    
                    {
                        let clean = path.replace("\\", "/").replace(" ", "%20");
                        let url = format!("http://127.0.0.1:3030/{}", clean.split("/media/").last().unwrap_or(&clean));
                        
                        // 👇 LA SOLUTION EST ICI : max-height: 85vh
                        // La vidéo ne fera jamais plus de 85% de la hauteur de l'écran.
                        // Comme elle est centrée verticalement par le parent (align-items: center),
                        // la barre de contrôle sera remontée bien au-dessus de la barre des tâches.
                        rsx! { video { key: "{url}", src: "{url}", controls: true, autoplay: true, style: "width: 100%; max-height: 85vh; background: black;" } }
                    }
                }
            } else {
                div { class: "top-bar", Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" }, div { class: "page-title", "Films" } }
                div { class: "media-grid",
                    for item in list_signal().iter().filter(|i| i.media_type == MediaType::Video) {
                        div { class: "media-card",
                            onclick: { let p=item.path.clone(); let i=item.id; let tx=cmd_tx.clone(); move |_| { current_video.set(Some(p.clone())); tx.send(Command::Play(i)).unwrap(); } },
                            div { class: "card-icon", "🎬" }
                            div { class: "card-text", style: "overflow: hidden; text-overflow: ellipsis; white-space: nowrap; width: 100%;", "{item.title.as_deref().unwrap_or(&item.path)}" }
                        }
                    }
                }
            }
        }
    }
}

// --- IMAGES ---
#[component]
pub fn Images() -> Element {
    let cmd_tx = use_context::<std::sync::mpsc::Sender<Command>>();
    let list_signal = use_context::<Signal<Vec<MediaInfo>>>();
    let mut current_image = use_signal(|| Option::<String>::None);
    let tx_init = cmd_tx.clone();
    use_hook(move || { if list_signal().is_empty() { tx_init.send(Command::GetAllMedia()).unwrap(); } });

    rsx! {
        div { class: "container",
            if let Some(data) = current_image() {
                // Même logique pour l'image : on centre et on limite la hauteur
                div { style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: black; z-index: 999; display: flex; align-items: center; justify-content: center;",
                    button { class: "btn-nav", style: "position: absolute; top: 20px; left: 20px;", onclick: move |_| current_image.set(None), "Fermer" }
                    img { src: "{data}", style: "max-width: 100%; max-height: 85vh;" }
                }
            } else {
                div { class: "top-bar", Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" }, div { class: "page-title", "Images" } }
                div { class: "media-grid",
                    for item in list_signal().iter().filter(|i| i.media_type == MediaType::Image) {
                        div { class: "media-card",
                            onclick: {
                                let p=item.path.clone(); let i=item.id; let tx=cmd_tx.clone();
                                move |_| { 
                                    if let Ok(bytes) = fs::read(&p) {
                                        let b64 = general_purpose::STANDARD.encode(&bytes);
                                        current_image.set(Some(format!("data:image/png;base64,{}", b64)));
                                    }
                                    tx.send(Command::Play(i)).unwrap();
                                }
                            },
                            div { class: "card-icon", "🖼️" }
                        }
                    }
                }
            }
        }
    }
}

// --- AUTRES PAGES ---
#[component] pub fn TV() -> Element { 
    rsx! { div { class: "container", div { class: "top-bar", Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" }, div { class: "page-title", "TV" } } } } 
}
#[component] pub fn Series() -> Element { 
    rsx! { div { class: "container", div { class: "top-bar", Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" }, div { class: "page-title", "Séries" } } } } 
}
#[component] pub fn Addons() -> Element { 
    rsx! { div { class: "container", div { class: "top-bar", Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" }, div { class: "page-title", "Add-ons" } } } } 
}
#[component] pub fn Settings() -> Element { 
    rsx! { div { class: "container", div { class: "top-bar", Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" }, div { class: "page-title", "Paramètres" } } } } 
}
#[component] pub fn PageNotFound(route: Vec<String>) -> Element { 
    rsx! { div { class: "container", h1 { "404" }, Link { to: Route::Home {}, class: "btn-nav", "Retour Accueil" } } } 
}