use dioxus::prelude::*;
use super::route::Route;
use crate::threading::command::Command;
use crate::media::data::{MediaType, MediaInfo};
use std::fs; 
use base64::{Engine as _, engine::general_purpose};
use std::path::PathBuf;
use crate::library::sources::LibraryConfig;
use crate::constants::SOURCE_FILE;
use urlencoding::encode;
use rand::Rng;
use std::time::Duration;

// 👇 STRUCTURE POUR LES PLUGINS
#[derive(Clone, PartialEq)]
pub struct PluginSearchResult {
    pub text: String,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum PlayMode {
    StopAtEnd,
    Sequential,
    Random,
    Loop,
}

impl PlayMode {
    fn next(&self) -> Self {
        match self {
            PlayMode::StopAtEnd => PlayMode::Sequential,
            PlayMode::Sequential => PlayMode::Random,
            PlayMode::Random => PlayMode::Loop,
            PlayMode::Loop => PlayMode::StopAtEnd,
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            PlayMode::StopAtEnd => "🛑 Stop",
            PlayMode::Sequential => "➡️ Suite",
            PlayMode::Random => "🔀 Hasard",
            PlayMode::Loop => "🔁 Boucle",
        }
    }
    
    fn color(&self) -> &'static str {
        match self {
            PlayMode::StopAtEnd => "#7f8c8d",
            PlayMode::Sequential => "#3498db",
            PlayMode::Random => "#9b59b6",
            PlayMode::Loop => "#e67e22",
        }
    }
}

fn make_url(full_path: &str, _root_path: &str) -> String {
    let clean_path = full_path.replace("\\", "/");
    
    if let Some(colon_idx) = clean_path.find(':') {
        if colon_idx == 1 { 
            let drive_letter = &clean_path[0..1].to_lowercase();
            let path_after_drive = &clean_path[3..];

            let encoded_parts: Vec<_> = path_after_drive.split('/')
                .map(|part| encode(part))
                .collect();
            
            return format!("http://127.0.0.1:3030/drives/{}/{}", drive_letter, encoded_parts.join("/"));
        }
    }

    format!("http://127.0.0.1:3030/media/{}", encode(&clean_path))
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
                    Link { to: Route::Settings {}, class: "media-card", div { class: "card-icon", "⚙️" } div { class: "card-text", "Paramètres" } }
                }
            }
        }
    }
}

#[component]
pub fn Music() -> Element {
    let cmd_tx = use_context::<std::sync::mpsc::Sender<Command>>();
    let list_signal = use_context::<Signal<Vec<MediaInfo>>>();
    let root_path_signal = use_context::<Signal<String>>();
    let root_path = root_path_signal();
    let plugin_result = use_context::<Signal<PluginSearchResult>>();
    
    let mut current_audio = use_signal(|| Option::<MediaInfo>::None);
    let mut play_mode = use_signal(|| PlayMode::Sequential);
    let mut search_text = use_signal(|| String::new());
    
    let mut queue = use_signal(|| Vec::<MediaInfo>::new());

    let tx_init = cmd_tx.clone();
    use_hook(move || { if list_signal().is_empty() { tx_init.send(Command::GetAllMedia()).unwrap(); } });

    let css_marquee = "
        @keyframes scroll-text { 0% { transform: translateX(100%); } 100% { transform: translateX(-100%); } }
        .marquee-container { overflow: hidden; white-space: nowrap; width: 100%; position: relative; }
        .marquee-text { display: inline-block; animation: scroll-text 15s linear infinite; padding-left: 100%; }
        .audio-row:active { background-color: #333 !important; transform: scale(0.99); transition: transform 0.1s; }
        /* Style pour le bouton d'ajout */
        .add-queue-btn { opacity: 0.5; transition: opacity 0.2s; }
        .add-queue-btn:hover { opacity: 1; transform: scale(1.1); }
    ";

    rsx! {
        style { "{css_marquee}" }
        
        div { class: "container", style: "padding-bottom: 100px;",
            
            div { class: "top-bar", 
                style: "display: flex; align-items: center; justify-content: space-between; position: relative; height: 60px; padding: 0 20px;",
                div { style: "z-index: 2;", Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" } }
                div { class: "page-title", style: "position: absolute; left: 50%; transform: translateX(-50%);", "Musique" } 
                div { style: "z-index: 2;",
                    input {
                        r#type: "text", placeholder: "🔍 Titre...",
                        style: "padding: 8px; border-radius: 5px; border: none; background: #333; color: white; width: 250px;",
                        oninput: move |evt| search_text.set(evt.value()),
                    }
                }
            }
            
            div { class: "audio-list",
                for item in list_signal().iter()
                    .filter(|i| i.media_type == MediaType::Audio)
                    .filter(|i| {
                        let query = search_text().to_lowercase();
                        if query.is_empty() { return true; }
                        i.title.as_deref().unwrap_or(&i.path).to_lowercase().contains(&query)
                    })
                {
                    div { class: "audio-row",
                        style: "cursor: pointer; transition: background 0.2s; user-select: none; display: flex; align-items: center; justify-content: space-between; padding-right: 15px;",
                        
                        onclick: { 
                            let track = item.clone(); 
                            let i = item.id; 
                            let tx = cmd_tx.clone();
                            let mut res = plugin_result.clone(); 
                            move |_| { 
                                res.set(PluginSearchResult { text: String::from("...") });
                                current_audio.set(Some(track.clone())); 
                                tx.send(Command::Play(i)).unwrap(); 
                                tx.send(Command::GetArtistMetadataFromPlugin(track.path.clone())).unwrap();
                            } 
                        },
                        
                        div { style: "display: flex; align-items: center; flex: 1;",
                            div { class: "audio-icon", 
                                if current_audio().as_ref().map(|c| c.id) == Some(item.id) { "🔊" } else { "🎵" }
                            }
                            div { class: "audio-info", 
                                div { 
                                    class: "audio-title", 
                                    style: if current_audio().as_ref().map(|c| c.id) == Some(item.id) { "color: #1db954; font-weight: bold;" } else { "" },
                                    "{item.title.as_deref().unwrap_or(&item.path)}" 
                                } 
                                div { class: "audio-artist", "Artiste inconnu" } 
                            }
                        },

                        button {
                            class: "add-queue-btn",
                            style: "background: transparent; border: 1px solid #555; color: white; border-radius: 50%; width: 30px; height: 30px; cursor: pointer; display: flex; align-items: center; justify-content: center;",
                            title: "Ajouter à la file d'attente",
                            
                            onclick: {
                                let track = item.clone();
                                move |evt: Event<MouseData>| {
                                    evt.stop_propagation();
                                    queue.write().push(track.clone());
                                }
                            },
                            "➕"
                        }
                    }
                }
            }

            if let Some(track) = current_audio() {
                div { 
                    style: "position: fixed; bottom: 0; left: 0; width: 100%; height: 90px; background: #181818; border-top: 1px solid #282828; display: flex; align-items: center; justify-content: space-between; padding: 0 20px; z-index: 1000; box-shadow: 0 -5px 15px rgba(0,0,0,0.5);",
                    
                    div { style: "width: 25%; overflow: hidden;",
                        div { class: "marquee-container",
                            div { class: "marquee-text", style: "font-weight: bold; font-size: 1.1rem;",
                                "{track.title.as_deref().unwrap_or(&track.path)}"
                            }
                        }
                        if !queue().is_empty() {
                            div { style: "color: #3498db; font-size: 0.8rem; margin-top: 4px; font-weight: bold;", 
                                "⏭️ En attente : {queue().len()} titre(s)" 
                            }
                        } else {
                            div { style: "color: #b3b3b3; font-size: 0.9rem; margin-top: 4px;", "{plugin_result.read().text}" }
                        }
                    },

                    div { style: "flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center;",
                        audio { 
                            controls: true, 
                            autoplay: true, 
                            style: "width: 100%; max-width: 500px; height: 40px; outline: none;",
                            src: "{make_url(&track.path, &root_path)}",
                            
                            r#loop: play_mode() == PlayMode::Loop,
                            
                            onended: move |_| {
                                if !queue().is_empty() {
                                    let next_song = queue.write().remove(0);
                                    current_audio.set(Some(next_song.clone()));
                                    cmd_tx.send(Command::Play(next_song.id)).unwrap();
                                    cmd_tx.send(Command::GetArtistMetadataFromPlugin(next_song.path)).unwrap();
                                    return;
                                }

                                let mode = play_mode();
                                let list = list_signal();
                                let audios: Vec<&MediaInfo> = list.iter().filter(|i| i.media_type == MediaType::Audio).collect();
                                
                                match mode {
                                    PlayMode::StopAtEnd => current_audio.set(None),
                                    
                                    PlayMode::Loop => {},
                                    
                                    PlayMode::Sequential => {
                                        if let Some(idx) = audios.iter().position(|x| x.id == track.id) {
                                            if idx + 1 < audios.len() {
                                                let next = audios[idx + 1].clone();
                                                current_audio.set(Some(next.clone()));
                                                cmd_tx.send(Command::Play(next.id)).unwrap();
                                                cmd_tx.send(Command::GetArtistMetadataFromPlugin(next.path)).unwrap();
                                            } else {
                                                current_audio.set(None);
                                            }
                                        }
                                    },
                                    PlayMode::Random => {
                                        if !audios.is_empty() {
                                            let mut rng = rand::thread_rng();
                                            let random_idx = rng.gen_range(0..audios.len());
                                            let next = audios[random_idx].clone();
                                            current_audio.set(Some(next.clone()));
                                            cmd_tx.send(Command::Play(next.id)).unwrap();
                                            cmd_tx.send(Command::GetArtistMetadataFromPlugin(next.path)).unwrap();
                                        }
                                    }
                                }
                            }
                        }
                    },

                    div { style: "width: 25%; display: flex; justify-content: flex-end; align-items: center; gap: 10px;",
                        if !queue().is_empty() {
                            button {
                                style: "background: transparent; border: 1px solid #e74c3c; color: #e74c3c; padding: 5px 10px; border-radius: 5px; cursor: pointer; font-size: 0.8rem;",
                                onclick: move |_| queue.write().clear(),
                                "🗑️"
                            }
                        }

                        button {
                            style: "background: transparent; border: 1px solid {play_mode().color()}; color: {play_mode().color()}; padding: 8px 15px; border-radius: 20px; cursor: pointer; font-weight: bold; transition: all 0.2s;",
                            onclick: move |_| play_mode.set(play_mode().next()),
                            "{play_mode().icon()}"
                        }

                        button {
                            style: "background: transparent; border: none; color: #fff; font-size: 1.5rem; cursor: pointer; margin-left: 10px;",
                            onclick: move |_| current_audio.set(None),
                            "❌"
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
    let mut list_signal = use_context::<Signal<Vec<MediaInfo>>>();
    let root_path_signal = use_context::<Signal<String>>();
    let root_path = root_path_signal();
    
    let mut current_video = use_signal(|| Option::<String>::None);
    let mut search_text = use_signal(|| String::new());
    
    // États pour le feedback visuel (Netflix style)
    let mut show_seek_back = use_signal(|| false);
    let mut show_seek_forward = use_signal(|| false);
    
    let tx_init = cmd_tx.clone(); 
    use_hook(move || { 
        if list_signal().is_empty() { 
            tx_init.send(Command::GetAllMedia()).unwrap(); 
        } 
    });

    // CSS pour l'animation d'apparition/disparition du feedback
    let css_anim = "
        @keyframes fadeOut {
            0% { opacity: 1; transform: scale(1); }
            100% { opacity: 0; transform: scale(1.5); }
        }
        .seek-feedback {
            position: absolute;
            top: 50%;
            transform: translateY(-50%);
            font-size: 2rem;
            font-weight: bold;
            color: white;
            background: rgba(0,0,0,0.5);
            padding: 20px;
            border-radius: 50%;
            pointer-events: none;
            z-index: 30;
            animation: fadeOut 0.8s forwards;
        }
    ";

    rsx! {
        style { "{css_anim}" }

        div { class: "container",
            if let Some(path) = current_video() {
                div { 
                    style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: black; z-index: 999; display: flex; flex-direction: column;",
                    
                    div { style: "height: 60px; padding: 10px; z-index: 1000; position: relative;",
                        button { class: "btn-nav", onclick: move |_| current_video.set(None), "⬅ Retour" }
                    }

                    div { style: "flex: 1; min-height: 0; display: flex; align-items: center; justify-content: center; position: relative; background: black;",
                        {
                            let url = make_url(&path, &root_path);
                            let current_media = list_signal().iter().find(|m| m.path == path).cloned();
                            let start_time = current_media.as_ref().map(|m| m.last_position).unwrap_or(0.0);
                            let media_id = current_media.as_ref().map(|m| m.id).unwrap_or(0);
                            let tx = cmd_tx.clone();

                            rsx! {
                                input {
                                    id: "spy-input",
                                    r#type: "hidden",
                                    value: "",
                                    oninput: move |evt| {
                                        let val = evt.value();
                                        let parts: Vec<&str> = val.split('|').collect();
                                        if parts.len() == 2 {
                                            if let (Ok(time), Ok(duration)) = (parts[0].parse::<f32>(), parts[1].parse::<f32>()) {
                                                if media_id > 0 { tx.send(Command::UpdateProgress(media_id, time, duration)).unwrap(); }
                                                list_signal.write().iter_mut().find(|m| m.id == media_id).map(|m| {
                                                    m.last_position = time;
                                                    m.duration = Some(duration);
                                                });
                                            }
                                        }
                                    }
                                }

                                video { 
                                    id: "main-player", 
                                    key: "{url}", 
                                    src: "{url}", 
                                    controls: true, 
                                    autoplay: true, 
                                    style: "max-width: 100%; max-height: 100%; width: 100%;",
                                }
                                
                                // === ZONES TACTILES INVISIBLES ===

                                // ZONE GAUCHE (-10s)
                                div {
                                    style: "position: absolute; top: 0; left: 0; width: 30%; height: 80%; z-index: 10; cursor: pointer;",
                                    ondblclick: move |_| {
                                        let mut eval = eval(r#"var v=document.getElementById('main-player'); if(v) v.currentTime -= 10;"#);
                                        spawn(async move { eval.recv().await; });
                                        show_seek_back.set(false);
                                        spawn(async move { show_seek_back.set(true); });
                                    },
                                    onclick: move |_| {
                                        let mut eval = eval(r#"var v=document.getElementById('main-player'); if(v) { if(v.paused) v.play(); else v.pause(); }"#);
                                        spawn(async move { eval.recv().await; });
                                    }
                                }

                                // ZONE DROITE (+10s)
                                div {
                                    style: "position: absolute; top: 0; right: 0; width: 30%; height: 80%; z-index: 10; cursor: pointer;",
                                    ondblclick: move |_| {
                                        let mut eval = eval(r#"var v=document.getElementById('main-player'); if(v) v.currentTime += 10;"#);
                                        spawn(async move { eval.recv().await; });
                                        show_seek_forward.set(false);
                                        spawn(async move { show_seek_forward.set(true); });
                                    },
                                    onclick: move |_| {
                                        let mut eval = eval(r#"var v=document.getElementById('main-player'); if(v) { if(v.paused) v.play(); else v.pause(); }"#);
                                        spawn(async move { eval.recv().await; });
                                    }
                                }

                                // === FEEDBACK VISUEL ===
                                if show_seek_back() {
                                    div { 
                                        class: "seek-feedback", 
                                        style: "left: 15%;", 
                                        onanimationend: move |_| show_seek_back.set(false), 
                                        "⏪ -10s" 
                                    }
                                }
                                if show_seek_forward() {
                                    div { 
                                        class: "seek-feedback", 
                                        style: "right: 15%;", 
                                        onanimationend: move |_| show_seek_forward.set(false),
                                        "+10s ⏩" 
                                    }
                                }

                                script { "
                                    var v = document.getElementById('main-player');
                                    var spy = document.getElementById('spy-input');
                                    if (v && {start_time} > 0) {{ v.currentTime = {start_time}; }}
                                    if (v && spy) {{
                                        v.ontimeupdate = function() {{
                                            var total = v.duration || 0; 
                                            spy.value = v.currentTime + '|' + total;
                                            spy.dispatchEvent(new Event('input', {{ bubbles: true }}));
                                        }};
                                    }}
                                " }
                            }
                        }
                    }
                }
            } 
            // ==========================
            // GRILLE DES VIDÉOS (LISTE)
            // ==========================
            else {
                div { class: "top-bar", 
                    style: "display: flex; align-items: center; justify-content: space-between; position: relative; height: 60px; padding: 0 20px;",

                    div { style: "z-index: 2;",
                        Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" }
                    }

                    div { 
                        class: "page-title", 
                        style: "position: absolute; left: 50%; transform: translateX(-50%); width: auto; white-space: nowrap;",
                        "Vidéos" 
                    }

                    input {
                        r#type: "text",
                        placeholder: "🔍 Rechercher un film...",
                        style: "padding: 8px; border-radius: 5px; border: none; background: #333; color: white; width: 250px;",
                        oninput: move |evt| search_text.set(evt.value()),
                    }
                }
                
                div { class: "media-grid",
                    for item in list_signal().iter()
                        .filter(|i| i.media_type == MediaType::Video)
                        .filter(|i| {
                            let query = search_text().to_lowercase();
                            if query.is_empty() { return true; }
                            let name = i.title.as_deref().unwrap_or(&i.path).to_lowercase();
                            name.contains(&query)
                        }) 
                    {
                        {
                            let (progress_percent, has_started) = match item.duration {
                                Some(total) if total > 0.0 => ((item.last_position / total) * 100.0, item.last_position > 5.0),
                                _ => (0.0, item.last_position > 5.0),
                            };

                            rsx! {
                                div { 
                                    class: "media-card",
                                    style: "position: relative; overflow: hidden;", 
                                    onclick: { 
                                        let p=item.path.clone(); 
                                        let i=item.id; 
                                        let tx=cmd_tx.clone(); 
                                        move |_| { current_video.set(Some(p.clone())); tx.send(Command::Play(i)).unwrap(); } 
                                    },

                                    div { class: "card-icon", "🎬" }

                                    if progress_percent > 0.0 {
                                        div { 
                                            style: "position: absolute; bottom: 0; left: 0; width: 100%; height: 6px; background: rgba(0,0,0,0.6); z-index: 10;",
                                            div { style: "height: 100%; background: #e50914; width: {progress_percent}%; transition: width 0.3s;" }
                                        }
                                    } else if has_started {
                                        div { 
                                            style: "position: absolute; bottom: 0; left: 0; width: 100%; height: 6px; background: rgba(0,0,0,0.6); z-index: 10;",
                                            div { style: "height: 100%; background: #3498db; width: 100%;" } 
                                        }
                                    }

                                    div { class: "card-text", style: "overflow: hidden; text-overflow: ellipsis; white-space: nowrap; width: 100%;", 
                                        "{item.title.as_deref().unwrap_or(&item.path)}" 
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

// --- IMAGES ---
#[component]
pub fn Images() -> Element {
    let cmd_tx = use_context::<std::sync::mpsc::Sender<Command>>();
    let list_signal = use_context::<Signal<Vec<MediaInfo>>>();
    let mut current_image = use_signal(|| Option::<String>::None);
    
    let mut search_text = use_signal(|| String::new());

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
            } 
            else {
                div { class: "top-bar", 
                    style: "display: flex; align-items: center; justify-content: space-between; position: relative; height: 60px; padding: 0 20px;",

                    div { style: "z-index: 2;",
                        Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" }
                    }

                    div { 
                        class: "page-title", 
                        style: "position: absolute; left: 50%; transform: translateX(-50%); width: auto; white-space: nowrap;",
                        "Images" 
                    } 

                    div { style: "z-index: 2;",
                        input {
                            r#type: "text",
                            placeholder: "🔍 Rechercher...",
                            style: "padding: 8px; border-radius: 5px; border: none; background: #333; color: white; width: 250px;",
                            oninput: move |evt| search_text.set(evt.value()),
                        }
                    }
                }

                div { class: "media-grid",
                    for item in list_signal().iter()
                        .filter(|i| i.media_type == MediaType::Image)
                        .filter(|i| {
                            let query = search_text().to_lowercase();
                            if query.is_empty() { return true; }
                            let name = i.title.as_deref().unwrap_or(&i.path).to_lowercase();
                            name.contains(&query)
                        })
                    {
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
                div { class: "page-title", "PLUGINS" } 
            }
            
            div { style: "display: flex; flex-direction: column; align-items: center; gap: 30px; margin-top: 50px;",
                h2 { "MusicBrainz" }

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
                            cmd_tx.send(Command::GetArtistMetadataFromPlugin(search_text())).unwrap();
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

// --- AUTRES PAGES ---
#[component] pub fn Iptv() -> Element { rsx! { div { class: "container", div { class: "top-bar", Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" }, div { class: "page-title", "Séries" } } } } }
#[component] pub fn Series() -> Element { rsx! { div { class: "container", div { class: "top-bar", Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" }, div { class: "page-title", "Séries" } } } } }
#[component] pub fn PageNotFound(route: Vec<String>) -> Element { rsx! { div { class: "container", h1 { "404 - Page non trouvée" }, Link { to: Route::Home {}, class: "btn-nav", "Retour Accueil" } } } }

// --- PARAMÈTRES ---
#[component] 
pub fn Settings() -> Element { 
    let cmd_tx = use_context::<std::sync::mpsc::Sender<Command>>();
    
    let mut scan_message = use_signal(|| String::new());

    let mut sources_signal = use_signal(|| {
        let config = LibraryConfig::load(SOURCE_FILE);
        let mut paths = Vec::new();
        for s in config.video_sources { paths.push(s.path.to_string_lossy().to_string()); }
        for s in config.music_sources { paths.push(s.path.to_string_lossy().to_string()); }
        for s in config.image_sources { paths.push(s.path.to_string_lossy().to_string()); }
        paths.sort();
        paths.dedup();
        paths
    });

    rsx! { 
        div { class: "container", 
            div { class: "top-bar", 
                Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" }, 
                div { class: "page-title", "Paramètres" } 
            }
  
            div { style: "display: flex; flex-direction: column; align-items: center; gap: 30px; margin-top: 50px; max-width: 800px; margin-left: auto; margin-right: auto; padding-bottom: 50px;",
                
                div { style: "text-align: center; width: 100%;",
                    h2 { "Gestion des Sources" }
                    p { style: "color: #aaa; margin-bottom: 20px;", "Gérez ici les dossiers que NeoKodi doit scanner." }
                }

                div { style: "width: 100%; display: flex; flex-direction: column; gap: 10px;",
                    if sources_signal().is_empty() {
                        div { style: "text-align: center; font-style: italic; color: #666; padding: 20px;", "Aucune source configurée." }
                    }

                    for path in sources_signal().iter() {
                        div { 
                            style: "background: #1e1e1e; padding: 15px; border-radius: 8px; border: 1px solid #333; display: flex; justify-content: space-between; align-items: center;",
                            
                            div { style: "font-family: monospace; color: #007acc; font-size: 1.1rem;", "📂 {path}" }
                            
                            button {
                                class: "btn-nav",
                                style: "position: relative; transform: none; top: auto; left: auto; background: #c0392b; padding: 8px 15px; font-size: 0.9rem;",
                                onclick: {
                                    let p = path.clone();
                                    let tx = cmd_tx.clone();
                                    move |_| {
                                        let path_buf = PathBuf::from(&p);
                                        tx.send(Command::RemoveSource(path_buf.clone(), MediaType::Video)).unwrap();
                                        tx.send(Command::RemoveSource(path_buf.clone(), MediaType::Audio)).unwrap();
                                        tx.send(Command::RemoveSource(path_buf.clone(), MediaType::Image)).unwrap();
                                        
                                        sources_signal.write().retain(|x| x != &p);
                                    }
                                },
                                "🗑️"
                            }
                        }
                    }
                }

                button { 
                    class: "btn-nav", 
                    style: "position: relative; transform: none; top: auto; left: auto; font-size: 1.1rem; padding: 15px 30px; background-color: #27ae60;",
                    onclick: {
                        let tx = cmd_tx.clone(); 
                            move |_| {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                let path_str = path.to_string_lossy().to_string();
                                if !sources_signal().contains(&path_str) {
                                    tx.send(Command::AddSource(path.clone(), MediaType::Video)).unwrap();
                                    tx.send(Command::AddSource(path.clone(), MediaType::Audio)).unwrap();
                                    tx.send(Command::AddSource(path.clone(), MediaType::Image)).unwrap();
                                    tx.send(Command::Reload()).unwrap(); 
                                    sources_signal.write().push(path_str);
                                }
                            }
                        }
                    },
                    "➕ Ajouter un dossier"
                }

                div { style: "width: 100%; height: 1px; background: #333; margin: 20px 0;" }

                div { style: "text-align: center; width: 100%;",
                    h2 { "Maintenance" }
                    p { style: "color: #aaa; margin-bottom: 20px;", "Si vos fichiers n'apparaissent pas, forcez une relecture complète." }
                    div { style: "display: flex; flex-direction: column; align-items: center; gap: 10px;",
                        button {
                            class: "btn-nav",
                            style: "position: relative; transform: none; top: auto; left: auto; font-size: 1.1rem; padding: 15px 30px; background-color: #2980b9;",
                            onclick: {
                                let tx = cmd_tx.clone();
                                move |_| {
                                    scan_message.set("⏳ Analyse des fichiers en cours...".to_string());
                                    tx.send(Command::Reload()).unwrap();
                                    spawn(async move {
                                        tokio::time::sleep(Duration::from_secs(3)).await;
                                        scan_message.set(String::new());
                                    });
                                }
                            },
                            "🔄 Forcer le re-scan complet"
                        }
                        // Message de confirmation
                        if !scan_message().is_empty() {
                            div { style: "color: #2ecc71; font-weight: bold; margin-top: 10px;", "{scan_message}" }
                        }
                    }
                }
            }
        } 
    } 
}