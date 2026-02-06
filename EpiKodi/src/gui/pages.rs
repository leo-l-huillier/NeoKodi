use dioxus::prelude::*;
use super::route::Route;
use crate::threading::command::Command;
use crate::media::data::{MediaType, MediaInfo};
use crate::config::AppConfig;
use std::fs; 
use base64::{Engine as _, engine::general_purpose};
use std::path::{Path, PathBuf};
use tokio::sync::broadcast;
use dioxus::desktop::use_eval;
// 👇 IMPORT NECESSAIRE POUR LA TV (Tri et Structures)
use crate::iptv::parser::{TVChannel, ContentType};

// 👇 STRUCTURE POUR LES PLUGINS
#[derive(Clone, PartialEq)]
pub struct PluginSearchResult {
    pub text: String,
}

// --- FONCTION UTILITAIRE URL ---
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
            format!("http://127.0.0.1:3030/media/{}", full_path.replace("\\", "/").replace(" ", "%20"))
        }
    }
}

// --- ACCUEIL ---
#[component]
pub fn Home() -> Element {
    rsx! {
        div { class: "container",
            div { style: "display: flex; flex-direction: column; align-items: center; justify-content: center; padding-top: 50px;",
                h1 { style: "font-size: 4rem; margin-bottom: 50px; color: #007acc; text-transform: uppercase; letter-spacing: 5px;", "NeoKodi" }
                
                div { class: "media-grid", style: "width: 100%; max-width: 900px;",
                    Link { to: Route::Videos {}, class: "media-card", div { class: "card-icon", "🎬" } div { class: "card-text", "Vidéos" } }
                    Link { to: Route::Images {}, class: "media-card", div { class: "card-icon", "🖼️" } div { class: "card-text", "Images" } }
                    Link { to: Route::Music {}, class: "media-card", div { class: "card-icon", "🎵" } div { class: "card-text", "Musique" } }
                    Link { to: Route::Plugins {}, class: "media-card", div { class: "card-icon", "🧩" } div { class: "card-text", "Plugins" } }
                    Link { to: Route::Iptv {}, class: "media-card", div { class: "card-icon", "📺" } div { class: "card-text", "TV & VOD" } }
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
    
    use_hook(move || { if list_signal().is_empty() { tx_init.send(Command::GetAllMedia()).unwrap(); } });

    rsx! {
        div { class: "container",
            if let Some(path) = current_audio() {
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
                            style: "position: relative; transform: none; top: auto; left: auto; background-color: #d32f2f; border-color: #b71c1c; font-size: 1.2rem; padding: 15px 40px;", 
                            onclick: move |_| current_audio.set(None), 
                            "⏹️ Arrêter la lecture" 
                        }
                    }
                }
            } else {
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

// --- FILMS (VIDEOS) ---
#[component]
pub fn Videos() -> Element {
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
                div { class: "top-bar", 
                    Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" }, 
                    div { class: "page-title", "Vidéos" } 
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
                div { style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: black; z-index: 999; display: flex; flex-direction: column;",
                    div { style: "height: 60px; padding: 10px;",
                        button { class: "btn-nav", style: "position: relative; top: 0; left: 0; transform: none;", onclick: move |_| current_image.set(None), "Fermer" }
                    }
                    div { style: "flex: 1; min-height: 0; display: flex; align-items: center; justify-content: center;",
                         img { src: "{data}", style: "max-width: 100%; max-height: 100%; object-fit: contain;" }
                    }
                }
            } else {
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

// --- PLUGINS (ADDONS) ---
#[component] 
pub fn Plugins() -> Element { 
    let cmd_tx = use_context::<std::sync::mpsc::Sender<Command>>();
    let plugin_result = use_context::<Signal<PluginSearchResult>>();
    let mut search_text = use_signal(|| String::from("Nirvana"));

    rsx! { 
        div { class: "container", 
            div { class: "top-bar", 
                Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" }, 
                div { class: "page-title", "Add-ons" } 
            }
            
            div { style: "display: flex; flex-direction: column; align-items: center; gap: 30px; margin-top: 50px;",
                h2 { "Test Plugin MusicBrainz" }

                div { style: "display: flex; gap: 10px;",
                    input {
                        style: "padding: 10px; border-radius: 4px; border: 1px solid #333; background: #1e1e1e; color: white; width: 300px;",
                        value: "{search_text}",
                        oninput: move |evt| search_text.set(evt.value())
                    }
                    button { 
                        class: "btn-nav", 
                        style: "position: relative; transform: none; top: auto; left: auto;",
                        onclick: move |_| {
                            cmd_tx.send(Command::GetArtistInfo(search_text())).unwrap();
                        },
                        "🔍 Rechercher"
                    }
                }

                div { style: "background: #1e1e1e; padding: 20px; border-radius: 8px; border: 1px solid #333; max-width: 600px; width: 80%; min-height: 100px;",
                    h3 { style: "margin-top: 0; color: #aaa; font-size: 1rem;", "Résultat du plugin :" }
                    pre { style: "color: #007acc; white-space: pre-wrap; font-family: monospace; font-size: 1.1rem;",
                        "{plugin_result().text}"
                    }
                }
            }
        } 
    } 
}

// --- IPTV (LIVE TV & VOD) ---
#[component]
pub fn Iptv() -> Element {
    let cmd_tx = use_context::<std::sync::mpsc::Sender<Command>>();
    let channels = use_context::<Signal<Vec<TVChannel>>>();
    let mut is_loading = use_context::<Signal<bool>>();
    let mut m3u_url = use_signal(|| String::from("https://iptv-org.github.io/iptv/countries/fr.m3u"));
    
    let mut current_stream = use_signal(|| Option::<String>::None);
    let mut active_tab = use_signal(|| ContentType::Live);
    
    // 👇 Utilise le use_eval du prélude (standard dans les versions récentes)
    let eval = use_eval();

    use_effect(move || {
        if let Some(url) = current_stream() {
            // 👇 CORRECTION : On utilise {0} pour injecter 'url' proprement sans duplication d'argument
            let js_code = format!(r#"
                console.log("🚀 Lecture: " + "{0}");
                setTimeout(() => {{
                    var video = document.getElementById('iptv-player');
                    
                    if (!video) {{
                        console.error("❌ ERREUR : Balise vidéo introuvable !");
                        return;
                    }}

                    var url = "{0}"; 

                    if (Hls.isSupported()) {{
                        console.log("✅ HLS détecté");
                        var hls = new Hls();
                        hls.loadSource(url);
                        hls.attachMedia(video);
                        
                        hls.on(Hls.Events.MANIFEST_PARSED, function() {{
                            video.play().catch(e => console.error("❌ Err Play:", e));
                        }});
                        
                        hls.on(Hls.Events.ERROR, function (event, data) {{
                             if (data.fatal) {{ console.error("⚠️ HLS Fatal Error:", data); }}
                        }});
                    }}
                    else if (video.canPlayType('application/vnd.apple.mpegurl')) {{
                        console.log("🍎 Natif");
                        video.src = url;
                        video.addEventListener('loadedmetadata', function() {{
                            video.play().catch(e => console.error("❌ Err Play:", e));
                        }});
                    }}
                }}, 500);
            "#, url); 

            let _ = eval(&js_code);
        }
    });

    rsx! {
        div { class: "container",
            if let Some(_) = current_stream() {
                // --- LECTEUR VIDÉO ---
                div { style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: black; z-index: 999; display: flex; flex-direction: column;",
                    div { style: "height: 60px; padding: 10px; background: rgba(0,0,0,0.5); display: flex; align-items: center;",
                        button { 
                            class: "btn-nav", 
                            style: "position: relative; transform: none;", 
                            onclick: move |_| current_stream.set(None), 
                            "⬅ Retour aux chaînes" 
                        }
                    }
                    div { style: "flex: 1; display: flex; align-items: center; justify-content: center;",
                        // 👇 Muted + Autoplay pour garantir le lancement
                        video { 
                            id: "iptv-player", 
                            controls: true, 
                            autoplay: true, 
                            "muted": "true",       
                            "playsinline": "true", 
                            style: "width: 100%; height: 100%; object-fit: contain;" 
                        }
                    }
                }
            } else {
                // --- Interface Principale ---
                div { class: "top-bar",
                    Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" },
                    div { class: "page-title", "TV & VOD" }
                }

                div { style: "padding: 20px; display: flex; flex-direction: column; gap: 20px;",
                    
                    if is_loading() {
                         div { style: "margin-top: 100px; text-align: center; width: 100%; display: flex; flex-direction: column; align-items: center;",
                            h2 { "Téléchargement..." }
                            div { class: "loading-container", div { class: "loading-bar" } }
                        }
                    } else if channels().is_empty() {
                         div { style: "text-align: center;",
                            input { 
                                style: "padding: 10px; width: 300px; margin-right: 10px;", 
                                value: "{m3u_url}", 
                                oninput: move |e| m3u_url.set(e.value()) 
                            }
                            button { 
                                onclick: move |_| { is_loading.set(true); cmd_tx.send(Command::LoadM3U(m3u_url())).unwrap(); },
                                "Charger"
                            }
                         }
                    } else {
                        // Onglets
                        div { style: "display: flex; gap: 10px; justify-content: center; margin-bottom: 20px;",
                            button {
                                style: if *active_tab.read() == ContentType::Live { "background: #007acc; color: white; padding: 10px 20px; border: none; border-radius: 4px;" } else { "background: #333; color: #aaa; padding: 10px 20px; border: none; border-radius: 4px;" },
                                onclick: move |_| active_tab.set(ContentType::Live),
                                "📺 Chaînes TV"
                            }
                            button {
                                style: if *active_tab.read() == ContentType::Movie { "background: #007acc; color: white; padding: 10px 20px; border: none; border-radius: 4px;" } else { "background: #333; color: #aaa; padding: 10px 20px; border: none; border-radius: 4px;" },
                                onclick: move |_| active_tab.set(ContentType::Movie),
                                "🎬 Films"
                            }
                            button {
                                style: if *active_tab.read() == ContentType::Series { "background: #007acc; color: white; padding: 10px 20px; border: none; border-radius: 4px;" } else { "background: #333; color: #aaa; padding: 10px 20px; border: none; border-radius: 4px;" },
                                onclick: move |_| active_tab.set(ContentType::Series),
                                "🍿 Séries"
                            }
                        }

                        // Grille
                        div { class: "media-grid",
                            for channel in channels().iter().filter(|c| c.content_type == *active_tab.read()) {
                                div { class: "media-card",
                                    onclick: { 
                                        let url = channel.url.clone();
                                        move |_| current_stream.set(Some(url.clone())) 
                                    },
                                    div { class: "card-icon", 
                                        match channel.content_type {
                                            ContentType::Live => "📺",
                                            ContentType::Movie => "🎬",
                                            ContentType::Series => "🍿",
                                        }
                                    }
                                    div { class: "card-text", "{channel.title}" }
                                    if let Some(g) = &channel.group {
                                        div { style: "font-size: 0.7rem; color: #666;", "{g}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// --- AUTRES PAGES ---
#[component] pub fn Series() -> Element { rsx! { div { class: "container", div { class: "top-bar", Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" }, div { class: "page-title", "Séries" } } } } }
#[component] pub fn PageNotFound(route: Vec<String>) -> Element { rsx! { div { class: "container", h1 { "404 - Page non trouvée" }, Link { to: Route::Home {}, class: "btn-nav", "Retour Accueil" } } } }

// --- PARAMÈTRES ---
#[component] 
pub fn Settings() -> Element { 
    let mut root_path_signal = use_context::<Signal<String>>();
    let cmd_tx = use_context::<std::sync::mpsc::Sender<Command>>();
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
                            let mut config = AppConfig::load();
                            config.media_path = path_str.clone();
                            config.save();
                            root_path_signal.set(path_str.clone());
                            let p = PathBuf::from(path_str);
                            let _ = cmd_tx.send(Command::ChangeLibraryPath(p));
                            println!("⚡ SIGNAL DE REDÉMARRAGE SERVEUR ENVOYÉ !");
                            let _ = reload_tx.send(()); 
                        }
                    },
                    "📂 Changer le dossier racine"
                }
            }
        } 
    } 
}