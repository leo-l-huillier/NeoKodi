use dioxus::prelude::*;
use super::route::Route;
use crate::threading::command::Command;
use crate::media::data::{MediaType, MediaInfo};
use crate::config::AppConfig;
use std::fs; 
use base64::{Engine as _, engine::general_purpose};
use std::path::{Path, PathBuf};
use tokio::sync::broadcast; // 👈 Nécessaire pour le redémarrage serveur

// --- FONCTION UTILITAIRE URL ---
// Transforme un chemin local (C:\Films\Avatar.mp4) en URL Serveur (http://127.0.0.1:3030/media/Avatar.mp4)
fn make_url(full_path: &str, root_path: &str) -> String {
    let path_obj = Path::new(full_path);
    let root_obj = Path::new(root_path);

    match path_obj.strip_prefix(root_obj) {
        Ok(relative) => {
            let relative_str = relative.to_string_lossy().replace("\\", "/");
            let clean_url = relative_str.replace(" ", "%20").replace("#", "%23");
            format!("http://127.0.0.1:3030/media/{}", clean_url)
        },
        Err(_) => {
            // Fallback si le chemin ne correspond pas (rare)
            format!("http://127.0.0.1:3030/media/{}", full_path.replace("\\", "/").replace(" ", "%20"))
        }
    }
}

// --- ACCUEIL ---
#[component]
pub fn Home() -> Element {
    rsx! {
        div { class: "container",
            // Titre centré avec un peu de marge
            div { style: "display: flex; flex-direction: column; align-items: center; justify-content: center; padding-top: 50px;",
                h1 { style: "font-size: 4rem; margin-bottom: 50px; color: #007acc; text-transform: uppercase; letter-spacing: 5px;", "NeoKodi" }
                
                div { class: "media-grid", style: "width: 100%; max-width: 900px;",
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
}

// --- MUSIQUE ---
#[component]
pub fn Music() -> Element {
    let cmd_tx = use_context::<std::sync::mpsc::Sender<Command>>();
    let list_signal = use_context::<Signal<Vec<MediaInfo>>>();
    let root_path_signal = use_context::<Signal<String>>();
    let root_path = root_path_signal();
    
    let mut current_audio = use_signal(|| Option::<String>::None);
    let tx_init = cmd_tx.clone();
    
    // Si la liste est vide, on force un rafraîchissement (utile si on arrive direct sur la page)
    use_hook(move || { if list_signal().is_empty() { tx_init.send(Command::GetAllMedia()).unwrap(); } });

    rsx! {
        div { class: "container",
            if let Some(path) = current_audio() {
                // Lecteur Audio Plein Écran (Overlay)
                div { style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: #121212; z-index: 999; display: flex; flex-direction: column;",
                    div { style: "flex: 1; display: flex; flex-direction: column; justify-content: center; align-items: center;",
                        div { style: "font-size: 5rem; margin-bottom: 20px;", "🎵" }
                        h2 { "Lecture en cours" }
                        audio { controls: true, autoplay: true, style: "width: 80%; max-width: 600px;",
                            onended: move |_| current_audio.set(None),
                            src: "{make_url(&path, &root_path)}"
                        }
                        button { 
                            class: "btn-nav", 
                            // On force le style ici pour écraser les positions absolues du CSS global
                            style: "position: relative; transform: none; top: auto; left: auto; background-color: #d32f2f; border-color: #b71c1c; font-size: 1.2rem; padding: 15px 40px;", 
                            onclick: move |_| current_audio.set(None), 
                            "⏹️ Arrêter la lecture" 
                        }
                    }
                }
            } else {
                // Vue Liste
                div { class: "top-bar", 
                    Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" }, 
                    div { class: "page-title", "Musique" } 
                }
                
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
    let root_path_signal = use_context::<Signal<String>>();
    let root_path = root_path_signal();
    
    let mut current_video = use_signal(|| Option::<String>::None);
    let tx_init = cmd_tx.clone(); 
    use_hook(move || { if list_signal().is_empty() { tx_init.send(Command::GetAllMedia()).unwrap(); } });

    rsx! {
        div { class: "container",
            if let Some(path) = current_video() {
                // Lecteur Vidéo Plein Écran
                div { style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: black; z-index: 999; display: flex; flex-direction: column;",
                    div { style: "height: 60px; padding: 10px;",
                        button { class: "btn-nav", style: "position: relative; top: 0; left: 0; transform: none;", onclick: move |_| current_video.set(None), "⬅ Retour" }
                    }
                    div { style: "flex: 1; min-height: 0; display: flex; align-items: center; justify-content: center;",
                        {
                            let url = make_url(&path, &root_path);
                            rsx! { video { key: "{url}", src: "{url}", controls: true, autoplay: true, style: "max-width: 100%; max-height: 100%; width: 100%;" } }
                        }
                    }
                }
            } else {
                // Vue Grille avec Barre Fixe (Sticky)
                div { class: "top-bar", 
                    Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" }, 
                    div { class: "page-title", "Films" } 
                }
                
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
                // Visionneuse Image Plein Écran
                div { style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: black; z-index: 999; display: flex; flex-direction: column;",
                    div { style: "height: 60px; padding: 10px;",
                        button { class: "btn-nav", style: "position: relative; top: 0; left: 0; transform: none;", onclick: move |_| current_image.set(None), "Fermer" }
                    }
                    div { style: "flex: 1; min-height: 0; display: flex; align-items: center; justify-content: center;",
                         img { src: "{data}", style: "max-width: 100%; max-height: 100%; object-fit: contain;" }
                    }
                }
            } else {
                // Vue Grille
                div { class: "top-bar", 
                    Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" }, 
                    div { class: "page-title", "Images" } 
                }
                
                div { class: "media-grid",
                    for item in list_signal().iter().filter(|i| i.media_type == MediaType::Image) {
                        div { class: "media-card",
                            onclick: {
                                let p=item.path.clone(); let i=item.id; let tx=cmd_tx.clone();
                                move |_| { 
                                    // Pour les images locales, on charge en base64 pour éviter les soucis de serveur lors d'un hot-reload
                                    if let Ok(bytes) = fs::read(&p) {
                                        let b64 = general_purpose::STANDARD.encode(&bytes);
                                        current_image.set(Some(format!("data:image/png;base64,{}", b64)));
                                    }
                                    tx.send(Command::Play(i)).unwrap();
                                }
                            },
                            div { class: "card-icon", "🖼️" }
                            div { class: "card-text", style: "overflow: hidden; text-overflow: ellipsis; white-space: nowrap; width: 100%;", "{item.title.as_deref().unwrap_or(&item.path)}" }
                        }
                    }
                }
            }
        }
    }
}

// --- AUTRES PAGES (Placeholders) ---
#[component] pub fn TV() -> Element { rsx! { div { class: "container", div { class: "top-bar", Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" }, div { class: "page-title", "TV" } } } } }
#[component] pub fn Series() -> Element { rsx! { div { class: "container", div { class: "top-bar", Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" }, div { class: "page-title", "Séries" } } } } }
#[component] pub fn Addons() -> Element { rsx! { div { class: "container", div { class: "top-bar", Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" }, div { class: "page-title", "Add-ons" } } } } }
#[component] pub fn PageNotFound(route: Vec<String>) -> Element { rsx! { div { class: "container", h1 { "404 - Page non trouvée" }, Link { to: Route::Home {}, class: "btn-nav", "Retour Accueil" } } } }


// --- PARAMÈTRES (AVEC HOT RELOAD) ---
#[component] 
pub fn Settings() -> Element { 
    let mut root_path_signal = use_context::<Signal<String>>();
    let cmd_tx = use_context::<std::sync::mpsc::Sender<Command>>();
    
    // On récupère le signal de redémarrage injecté dans main.rs
    let reload_tx = use_context::<broadcast::Sender<()>>();

    rsx! { 
        div { class: "container", 
            div { class: "top-bar", 
                Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" }, 
                div { class: "page-title", "Paramètres" } 
            }
            
            div { style: "display: flex; flex-direction: column; align-items: center; gap: 30px; margin-top: 50px;",
                div { style: "text-align: center;",
                    h2 { "Dossier Média Actuel" }
                    div { style: "background: #1e1e1e; padding: 20px; border-radius: 8px; font-family: monospace; color: #007acc; border: 1px solid #333; margin-top: 10px;",
                        "{root_path_signal}"
                    }
                }

                button { 
                    class: "btn-nav", 
                    style: "position: relative; transform: none; top: auto; left: auto; font-size: 1.2rem; padding: 15px 30px;",
                    onclick: move |_| {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            let path_str = path.to_string_lossy().to_string();
                            
                            // 1. Sauvegarde Config
                            let mut config = AppConfig::load();
                            config.media_path = path_str.clone();
                            config.save();

                            // 2. Mise à jour Visuelle
                            root_path_signal.set(path_str.clone());

                            // 3. Backend : Nettoyage + Scan
                            let p = PathBuf::from(path_str);
                            let _ = cmd_tx.send(Command::ChangeLibraryPath(p));

                            // 4. Serveur : Redémarrage immédiat (Hot Reload)
                            println!("⚡ SIGNAL DE REDÉMARRAGE SERVEUR ENVOYÉ !");
                            let _ = reload_tx.send(()); 
                        }
                    },
                    "📂 Changer le dossier racine"
                }

                div { style: "color: #666; font-size: 0.9rem; margin-top: 20px; max-width: 400px; text-align: center;",
                    "Note : Le changement de dossier redémarre automatiquement le serveur interne. La liste des médias se mettra à jour progressivement."
                }
            }
        } 
    } 
}