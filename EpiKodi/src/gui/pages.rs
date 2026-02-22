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
    // 1. On normalise les slashs (Windows utilise \, le web utilise /)
    let clean_path = full_path.replace("\\", "/");

    // 2. On gère les lettres de lecteur (Ex: "C:/Films/Vacances.mp4")
    if let Some(colon_idx) = clean_path.find(':') {
        if colon_idx == 1 { 
            // On récupère la lettre "c"
            let drive_letter = &clean_path[0..1].to_lowercase();
            
            // On prend tout ce qu'il y a après "C:/"
            // Attention : on vérifie qu'on ne dépasse pas la taille de la string
            let path_after_drive = if clean_path.len() > 3 {
                &clean_path[3..]
            } else {
                ""
            };

            // 3. LA CLÉ DU SUCCÈS : On encode chaque dossier séparément
            // Si on encode tout d'un coup, les "/" deviennent "%2F" et le serveur ne trouve pas le dossier.
            let encoded_parts: Vec<String> = path_after_drive.split('/')
                .map(|part| urlencoding::encode(part).to_string())
                .collect();
            
            // On reconstruit l'URL propre
            let url = format!("http://127.0.0.1:3030/drives/{}/{}", drive_letter, encoded_parts.join("/"));
            
            // Petit log pour vérifier
            println!("🔗 URL FIXÉE : {} -> {}", full_path, url);
            return url;
        }
    }

    // Cas de secours (si le chemin n'a pas de lettre de lecteur, rare sous Windows absolu)
    let encoded_path = urlencoding::encode(&clean_path).to_string();
    format!("http://127.0.0.1:3030/media/{}", encoded_path)
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
    let plugin_history = use_context::<Signal<Vec<String>>>();
    
    // Contextes Playlist
    let mut all_playlists = use_context::<Signal<Vec<(i64, String)>>>(); // (ID, Nom)
    let mut loaded_ids_signal = use_context::<Signal<Vec<i64>>>(); // IDs de la playlist active

    // États Lecture
    let mut current_audio = use_signal(|| Option::<MediaInfo>::None);
    let mut play_mode = use_signal(|| PlayMode::Sequential);
    let mut search_text = use_signal(|| String::new());
    let mut queue = use_signal(|| Vec::<MediaInfo>::new());

    // États UI
    let mut show_queue_popup = use_signal(|| false);
    let mut show_playlist_manager = use_signal(|| false);
    let mut new_playlist_name = use_signal(|| String::new());
    
    // 👇 NOUVEAU : Pour savoir si on est en mode "Vue Playlist"
    let mut active_playlist_name = use_signal(|| Option::<String>::None);
    
    // État pour le menu "Ajouter à..." (ID du média en cours d'ajout)
    let mut adding_media_to_playlist = use_signal(|| Option::<i64>::None);

    let tx_init = cmd_tx.clone();
    use_hook(move || { 
        if list_signal().is_empty() { tx_init.send(Command::GetAllMedia()).unwrap(); } 
        tx_init.send(Command::GetAllPlaylists()).unwrap();
        tx_init.send(Command::GetPluginHistory).unwrap();

        // 🚀 AUTO-RECHERCHE CORRIGÉE
        let list = list_signal();
        // On précise le type explicitement pour aider Rust
        let mut searched_artists = std::collections::HashSet::<String>::new();
        
        // On itère directement sur 'list' (pas de parenthèses ici !)
        for item in list.iter().filter(|i| i.media_type == MediaType::Audio) {
            if let Some(ref artist) = item.artist {
                if !searched_artists.contains(artist) {
                    let _ = tx_init.send(Command::GetArtistMetadataFromPlugin(artist.clone()));
                    searched_artists.insert(artist.clone());
                }
            }
        }
    });

    // NOTE : J'ai supprimé le use_effect qui remplissait la queue automatiquement.
    // Maintenant, cliquer sur une playlist ne fait que FILTRER la vue.
    // Si tu veux tout jouer, tu cliques sur le premier titre affiché.

    let css_music = "
        @keyframes scroll-text { 0% { transform: translateX(100%); } 100% { transform: translateX(-100%); } }
        .marquee-container { overflow: hidden; white-space: nowrap; width: 100%; position: relative; }
        .marquee-text { display: inline-block; animation: scroll-text 15s linear infinite; padding-left: 100%; }
        
        .audio-row:active { background-color: #333 !important; transform: scale(0.99); transition: transform 0.1s; }
        .add-queue-btn { opacity: 0.5; transition: opacity 0.2s; }
        .add-queue-btn:hover { opacity: 1; transform: scale(1.1); }
        
        .queue-popup::-webkit-scrollbar { width: 6px; }
        .queue-popup::-webkit-scrollbar-thumb { background: #555; border-radius: 3px; }
        .queue-popup::-webkit-scrollbar-track { background: #222; }

        .playlist-overlay {
            position: fixed; top: 0; left: 0; width: 100%; height: 100%;
            background: rgba(0,0,0,0.8); 
            z-index: 3000;
            display: flex; justify-content: center; align-items: center;
        }
        .playlist-modal {
            background: #1e1e1e; padding: 25px; border-radius: 12px; width: 400px;
            border: 1px solid #333; box-shadow: 0 10px 25px rgba(0,0,0,0.5);
        }
        .pl-item {
            display: flex; justify-content: space-between; align-items: center;
            padding: 10px; border-bottom: 1px solid #333; cursor: pointer;
        }
        .pl-item:hover { background: #2d2d2d; }
        .pl-option { padding: 8px 10px; cursor: pointer; color: white; }
        .pl-option:hover { background: #333; }
    ";

    rsx! {
        style { "{css_music}" }
        
        div { class: "container", style: "padding-bottom: 100px;",
            
            // --- TOP BAR ---
            div { class: "top-bar", 
                style: "display: flex; align-items: center; justify-content: space-between; position: relative; height: 60px; padding: 0 20px; z-index: 500; background: #121212;",
                
                div { style: "z-index: 2;", Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" } }
                
                // Bouton Gestion Playlists
                div { style: "z-index: 2; position: absolute; left: 140px;",
                    button { 
                        class: "btn-nav", 
                        style: "position: relative; transform: none; top: auto; left: auto; background: #8e44ad;",
                        onclick: move |_| show_playlist_manager.set(true),
                        "📂 Playlists"
                    }
                }

                // TITRE CENTRAL DYNAMIQUE
                div { 
                    style: "position: absolute; left: 50%; transform: translateX(-50%); display: flex; align-items: center; gap: 10px; z-index: 2;",
                    
                    if let Some(name) = active_playlist_name() {
                        // MODE PLAYLIST ACTIVE
                        span { style: "color: #2ecc71; font-weight: bold; font-size: 1.2rem;", "📂 {name}" }
                        button {
                            style: "background: none; border: none; color: #e74c3c; cursor: pointer; font-size: 1.2rem; padding: 0 5px;",
                            title: "Fermer la playlist",
                            onclick: move |_| {
                                active_playlist_name.set(None); // On quitte le mode playlist
                                loaded_ids_signal.write().clear(); // On vide le filtre
                            },
                            "✖"
                        }
                    } else {
                        // MODE NORMAL
                        span { style: "font-weight: bold; font-size: 1.2rem;", "Musique" }
                    }
                } 
                
                div { style: "z-index: 2;",
                    input {
                        r#type: "text", placeholder: "🔍 Titre...",
                        style: "padding: 8px; border-radius: 5px; border: none; background: #333; color: white; width: 250px;",
                        oninput: move |evt| search_text.set(evt.value()),
                    }
                }
            }
            
            // --- LISTE PRINCIPALE (AVEC FILTRE PLAYLIST) ---
            div { class: "audio-list",
                {list_signal().iter()
                    .filter(|i| i.media_type == MediaType::Audio)
                    // 1. Filtre Recherche
                    .filter(|i| {
                        let query = search_text().to_lowercase();
                        if query.is_empty() { return true; }
                        i.title.as_deref().unwrap_or(&i.path).to_lowercase().contains(&query)
                    })
                    // 2. Filtre Playlist (Si active)
                    .filter(|i| {
                        if active_playlist_name().is_some() {
                            loaded_ids_signal().contains(&i.id)
                        } else {
                            true
                        }
                    })
                    // 3. Mapping pour l'affichage
                    .map(|item| {
                        // Calcul du Z-Index dynamique
                        let is_menu_open = adding_media_to_playlist() == Some(item.id);
                        let z_index = if is_menu_open { 100 } else { 0 };
                        
                        rsx! {
                            div { 
                                class: "audio-row",
                                style: "cursor: pointer; transition: background 0.2s; user-select: none; display: flex; align-items: center; justify-content: space-between; padding-right: 15px; position: relative; z-index: {z_index}; overflow: visible;", 
                                
                                // Clic principal : Jouer
                                onclick: { 
                                    let track = item.clone();
                                    let i = item.id; 
                                    let tx = cmd_tx.clone();
                                    let mut history = plugin_history.clone(); // ✅ On utilise le nouveau nom
                                    move |_| { 
                                        history.write().insert(0, "...".to_string()); 
                                        
                                        current_audio.set(Some(track.clone())); 
                                        tx.send(Command::Play(i)).unwrap(); 
                                        
                                        if let Some(ref artist) = track.artist {
                                            let _ = tx.send(Command::GetArtistMetadataFromPlugin(artist.clone()));
                                        }
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
                                        div { 
                                            class: "audio-artist", 
                                            "{item.artist.as_deref().unwrap_or(\"Artiste inconnu\")}" 
                                        }
                                    }
                                },

                                div { style: "display: flex; gap: 10px;",
                                    // Bouton Disquette (Ajouter à playlist)
                                    button {
                                        class: "add-queue-btn",
                                        style: "background: transparent; border: 1px solid #555; color: #f1c40f; border-radius: 50%; width: 30px; height: 30px; cursor: pointer; display: flex; align-items: center; justify-content: center;",
                                        title: "Sauvegarder dans une playlist",
                                        onclick: {
                                            let id = item.id;
                                            move |evt: Event<MouseData>| {
                                                evt.stop_propagation(); 
                                                if adding_media_to_playlist() == Some(id) {
                                                    adding_media_to_playlist.set(None);
                                                } else {
                                                    adding_media_to_playlist.set(Some(id));
                                                }
                                            }
                                        },
                                        "💾"
                                    }

                                    // Bouton Plus (Ajouter à file d'attente)
                                    button {
                                        class: "add-queue-btn",
                                        style: "background: transparent; border: 1px solid #555; color: white; border-radius: 50%; width: 30px; height: 30px; cursor: pointer; display: flex; align-items: center; justify-content: center;",
                                        title: "Ajouter à la file d'attente actuelle",
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

                                // --- POPUP MENU DÉROULANT ---
                                if is_menu_open {
                                    div {
                                        style: "position: absolute; right: 50px; top: 40px; background: #222; border: 1px solid #444; border-radius: 5px; z-index: 200; min-width: 200px; box-shadow: 0 5px 15px rgba(0,0,0,0.8);",
                                        onclick: |evt: Event<MouseData>| evt.stop_propagation(),
                                        
                                        // Zone Création
                                        div {
                                            style: "padding: 8px; border-bottom: 1px solid #444; background: #2a2a2a;",
                                            input {
                                                r#type: "text",
                                                value: "{new_playlist_name}",
                                                placeholder: "➕ Créer & Ajouter...",
                                                style: "width: 100%; padding: 5px; border: 1px solid #555; border-radius: 3px; background: #111; color: white; font-size: 0.8rem;",
                                                oninput: move |evt| new_playlist_name.set(evt.value()),
                                            }
                                            if !new_playlist_name().is_empty() {
                                                div {
                                                    style: "margin-top: 5px; cursor: pointer; background: #27ae60; color: white; padding: 4px; text-align: center; border-radius: 3px; font-size: 0.8rem;",
                                                    onclick: {
                                                        let mid = item.id;
                                                        let tx = cmd_tx.clone();
                                                        move |_| {
                                                            tx.send(Command::AddPlaylist(new_playlist_name())).unwrap();
                                                            tx.send(Command::GetAllPlaylists()).unwrap();
                                                            // Idéalement on ajoute direct, mais ici on crée d'abord
                                                            new_playlist_name.set(String::new());
                                                        }
                                                    },
                                                    "Créer"
                                                }
                                            }
                                        }

                                        div { style: "padding: 5px 10px; color: #888; font-size: 0.7rem; text-transform: uppercase;", "Playlists existantes" }
                                        
                                        // Liste Playlists
                                        div { style: "max-height: 150px; overflow-y: auto;",
                                            for (pl_id, pl_name) in all_playlists() {
                                                div {
                                                    class: "pl-option",
                                                    onclick: {
                                                        let pid = pl_id;
                                                        let mid = item.id;
                                                        let tx = cmd_tx.clone();
                                                        move |_| {
                                                            tx.send(Command::AddMediaToPlaylist(mid, pid)).unwrap();
                                                            adding_media_to_playlist.set(None);
                                                        }
                                                    },
                                                    "📂 {pl_name}"
                                                }
                                            }
                                            if all_playlists().is_empty() {
                                                div { style: "padding: 10px; color: #666; font-style: italic; font-size: 0.8rem;", "Aucune playlist." }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    })
                }
            }

            // --- GESTIONNAIRE DE PLAYLISTS (MODAL) ---
            if show_playlist_manager() {
                div { class: "playlist-overlay", onclick: move |_| show_playlist_manager.set(false),
                    div { class: "playlist-modal", onclick: |evt| evt.stop_propagation(),
                        h2 { style: "margin-top: 0; color: white;", "Mes Playlists" }
                        
                        // Créer nouvelle (Gros bouton)
                        div { style: "display: flex; gap: 10px; margin-bottom: 20px;",
                            input {
                                r#type: "text",
                                value: "{new_playlist_name}",
                                placeholder: "Nom de la playlist...",
                                style: "flex: 1; padding: 8px; border-radius: 5px; border: 1px solid #444; background: #2d2d2d; color: white;",
                                oninput: move |evt| new_playlist_name.set(evt.value()),
                            }
                            button {
                                class: "btn-nav",
                                style: "position: relative; transform: none; top: auto; left: auto; background: #27ae60;",
                                onclick: {
                                    let tx = cmd_tx.clone();
                                    move |_| {
                                        if !new_playlist_name().is_empty() {
                                            tx.send(Command::AddPlaylist(new_playlist_name())).unwrap();
                                            tx.send(Command::GetAllPlaylists()).unwrap();
                                            new_playlist_name.set(String::new());
                                        }
                                    }
                                },
                                "Créer"
                            }
                        }

                        // Liste pour Ouvrir/Supprimer
                        div { style: "max-height: 300px; overflow-y: auto;",
                            for (id, name) in all_playlists() {
                                div { class: "pl-item",
                                    
                                    // 👇 C'EST CE BOUTON QUI DECONNE ACTUELLEMENT
                                    div { 
                                        style: "flex: 1;",
                                        onclick: {
                                            let pid = id;
                                            let pname = name.clone(); // Clone pour le signal
                                            let tx = cmd_tx.clone();
                                            move |_| {
                                                println!("🖱️ [FRONT] Clic sur ouvrir playlist ID: {}", pid); // MOCHARD FRONTEND
                                                
                                                // 1. Envoyer l'ordre au backend
                                                match tx.send(Command::GetMediaFromPlaylist(pid)) {
                                                    Ok(_) => println!("✅ [FRONT] Commande envoyée !"),
                                                    Err(e) => println!("❌ [FRONT] Erreur envoi commande : {}", e),
                                                }

                                                // 2. Activer le mode visuel
                                                active_playlist_name.set(Some(pname.clone()));
                                                
                                                // 3. Fermer la fenêtre
                                                show_playlist_manager.set(false);
                                            }
                                        },
                                        "📂 {name}" 
                                    },
                                    
                                    // SUPPRIMER (Celui-ci marchait peut-être déjà, mais on le garde propre)
                                    button {
                                        style: "background: none; border: none; color: #c0392b; cursor: pointer; font-weight: bold;",
                                        onclick: {
                                            let pid = id;
                                            let tx = cmd_tx.clone();
                                            move |evt: Event<MouseData>| {
                                                evt.stop_propagation();
                                                tx.send(Command::DeletePlaylist(pid)).unwrap();
                                                tx.send(Command::GetAllPlaylists()).unwrap();
                                            }
                                        },
                                        "🗑️"
                                    }
                                }
                            }
                            if all_playlists().is_empty() {
                                div { style: "color: #777; text-align: center; padding: 20px;", "Aucune playlist créée." }
                            }
                        }

                        div { style: "margin-top: 20px; text-align: right;",
                            button { 
                                class: "btn-nav", 
                                style: "position: relative; transform: none; top: auto; left: auto; background: #444;",
                                onclick: move |_| show_playlist_manager.set(false),
                                "Fermer" 
                            }
                        }
                    }
                }
            }

            // --- LECTEUR AUDIO (FIXED BOTTOM) ---
            if let Some(track) = current_audio() {
                div { 
                    style: "position: fixed; bottom: 0; left: 0; width: 100%; height: 90px; background: #181818; border-top: 1px solid #282828; display: flex; align-items: center; justify-content: space-between; padding: 0 20px; z-index: 1000; box-shadow: 0 -5px 15px rgba(0,0,0,0.5);",
                    
                    // Partie Gauche (Infos)
                    div { style: "width: 25%; position: relative;",
                        div { class: "marquee-container",
                            div { class: "marquee-text", style: "font-weight: bold; font-size: 1.1rem;",
                                "{track.title.as_deref().unwrap_or(&track.path)}"
                            }
                        }
                        if !queue().is_empty() {
                            div { 
                                style: "display: inline-block; cursor: help; position: relative;",
                                onmouseenter: move |_| show_queue_popup.set(true),
                                onmouseleave: move |_| show_queue_popup.set(false),
                                div { style: "color: #3498db; font-size: 0.8rem; margin-top: 4px; font-weight: bold;", "⏭️ En attente : {queue().len()} titre(s)" }
                                if show_queue_popup() {
                                    div {
                                        class: "queue-popup",
                                        style: "position: absolute; bottom: 130%; left: 0; width: 300px; max-height: 400px; overflow-y: auto; background: #282828; border: 1px solid #444; border-radius: 8px; box-shadow: 0 5px 20px rgba(0,0,0,0.8); padding: 10px; z-index: 2000;",
                                        h4 { style: "margin: 0 0 10px 0; color: #fff; border-bottom: 1px solid #444; padding-bottom: 5px;", "File d'attente" }
                                        for (idx, song) in queue().iter().enumerate() {
                                            div { 
                                                style: "padding: 8px; border-bottom: 1px solid #333; font-size: 0.9rem; color: #ccc; display: flex; gap: 10px;",
                                                span { style: "color: #888; font-family: monospace;", "{idx + 1}." }
                                                span { style: "white-space: nowrap; overflow: hidden; text-overflow: ellipsis;", "{song.title.as_deref().unwrap_or(&song.path)}" }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            div { 
                                    style: "color: #b3b3b3; font-size: 0.9rem; margin-top: 4px;", 
                                    // On affiche le premier élément de l'historique (le plus récent)
                                    "{plugin_history().first().cloned().unwrap_or_default()}" 
                            }
                        }
                    },

                    // Partie Centrale (Player HTML5)
                    div { style: "flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center;",
                        audio { 
                            controls: true, autoplay: true, style: "width: 100%; max-width: 500px; height: 40px; outline: none;",
                            src: "{make_url(&track.path, &root_path)}",
                            r#loop: play_mode() == PlayMode::Loop,
                            
                            // Logique de fin de piste
                            onended: move |_| {
                                // 1. Priorité à la File d'attente
                                if !queue().is_empty() {
                                    let next_song = queue.write().remove(0);
                                    current_audio.set(Some(next_song.clone()));
                                    cmd_tx.send(Command::Play(next_song.id)).unwrap();
                                    cmd_tx.send(Command::GetArtistMetadataFromPlugin(next_song.path)).unwrap();
                                    return;
                                }
                                
                                // 2. Sinon gestion du mode lecture (Sequential, Random...)
                                // Note: On lit dans la liste filtrée ACTUELLE (donc dans la playlist si active)
                                let mode = play_mode();
                                let list = list_signal();
                                
                                // On filtre selon ce qui est affiché à l'écran
                                let audios: Vec<&MediaInfo> = list.iter()
                                    .filter(|i| i.media_type == MediaType::Audio)
                                    // On applique le même filtre que l'affichage pour la suite logique
                                    .filter(|i| {
                                        if active_playlist_name().is_some() {
                                            loaded_ids_signal().contains(&i.id)
                                        } else { true }
                                    })
                                    .collect();

                                match mode {
                                    PlayMode::StopAtEnd => current_audio.set(None),
                                    PlayMode::Loop => {}, // Géré par l'attribut loop HTML
                                    PlayMode::Sequential => {
                                        if let Some(idx) = audios.iter().position(|x| x.id == track.id) {
                                            if idx + 1 < audios.len() {
                                                let next = audios[idx + 1].clone();
                                                current_audio.set(Some(next.clone()));
                                                cmd_tx.send(Command::Play(next.id)).unwrap();
                                            } else { current_audio.set(None); }
                                        }
                                    },
                                    PlayMode::Random => {
                                        if !audios.is_empty() {
                                            let mut rng = rand::thread_rng();
                                            let random_idx = rng.gen_range(0..audios.len());
                                            let next = audios[random_idx].clone();
                                            current_audio.set(Some(next.clone()));
                                            cmd_tx.send(Command::Play(next.id)).unwrap();
                                        }
                                    }
                                }
                            },
                            
                            // Script pour update la barre de progression DB
                            script { "
                                var v = document.getElementById('main-player'); // Assure-toi que l'ID est bon ou utilise 'this'
                                // ... ton script existant ...
                            " }
                        }
                    },

                    // Partie Droite (Contrôles)
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
    
    let plugin_history = use_context::<Signal<Vec<String>>>();
    
    // 👇 On stocke l'objet COMPLET pour garder l'ID sous la main
    let mut playing_video = use_signal(|| Option::<MediaInfo>::None);
    let mut selected_media = use_signal(|| Option::<MediaInfo>::None);
    let mut search_text = use_signal(|| String::new());

    let mut ui_visible = use_signal(|| true);
    let mut activity_id = use_signal(|| 0);
    
    // ⏱️ NOUVEAU : Signaux pour récupérer le temps en direct du lecteur HTML5
    let mut current_time = use_signal(|| 0.0f32);
    let mut current_duration = use_signal(|| 0.0f32);
    
    let on_mouse_move = move |_| {
        if !ui_visible() { ui_visible.set(true); }
        let new_id = activity_id() + 1;
        activity_id.set(new_id);

        spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
            if activity_id() == new_id { ui_visible.set(false); }
        });
    };

    use_hook(move || {
        let new_id = activity_id() + 1;
        activity_id.set(new_id);
        spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
            if activity_id() == new_id { ui_visible.set(false); }
        });
    });
    
    let tx_init = cmd_tx.clone();
    use_hook(move || { 
        if list_signal().is_empty() { 
            tx_init.send(Command::GetAllMedia()).unwrap(); 
        } 
    });

    let css_style = "
        .modal-backdrop { position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0, 0, 0, 0.85); z-index: 2000; display: flex; align-items: center; justify-content: center; opacity: 0; animation: fadeIn 0.3s forwards; backdrop-filter: blur(5px); }
        .modal-content { background: #181818; width: 80%; max-width: 900px; border-radius: 10px; overflow: hidden; box-shadow: 0 0 50px rgba(0,0,0,0.5); display: flex; flex-direction: column; transform: scale(0.9); animation: popIn 0.3s forwards; border: 1px solid #333; }
        @keyframes fadeIn { to { opacity: 1; } }
        @keyframes popIn { to { transform: scale(1); } }
        .modal-header { height: 300px; background-size: cover; background-position: center; position: relative; }
        .modal-gradient { position: absolute; bottom: 0; left: 0; width: 100%; height: 100%; background: linear-gradient(to top, #181818, transparent); }
        .modal-body { padding: 40px; color: white; margin-top: -50px; position: relative; z-index: 10; }
        .btn-play { background: white; color: black; padding: 12px 30px; font-size: 1.2rem; font-weight: bold; border-radius: 4px; border: none; cursor: pointer; display: flex; align-items: center; gap: 10px; transition: transform 0.2s; }
        .btn-play:hover { transform: scale(1.05); background: #ddd; }
        .close-btn { position: absolute; top: 20px; right: 20px; background: rgba(0,0,0,0.5); color: white; border-radius: 50%; width: 40px; height: 40px; border: 2px solid white; display: flex; align-items: center; justify-content: center; cursor: pointer; z-index: 3000; font-size: 20px; }
        .close-btn:hover { background: white; color: black; }
        .card-text { width: 100%; padding: 0 10px 10px 10px; font-size: 0.9rem; text-align: center; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; box-sizing: border-box; }
        .page-title-centered { position: absolute; left: 50%; transform: translateX(-50%); font-size: 1.5rem; font-weight: bold; text-transform: uppercase; letter-spacing: 2px; pointer-events: none; }
        .player-ui { position: absolute; top: 0; left: 0; width: 100%; height: 100px; z-index: 10000; display: flex; align-items: center; padding-left: 20px; background: linear-gradient(to bottom, rgba(0,0,0,0.8), transparent); transition: opacity 0.5s ease-in-out; pointer-events: none; }
        .player-ui.visible { opacity: 1; pointer-events: auto; }
        .player-ui.hidden { opacity: 0; }
    ";

    rsx! {
        style { "{css_style}" }

        div { class: "container",
            
            // ==========================================
            // 1. LE LECTEUR VIDÉO
            // ==========================================
            if let Some(media) = playing_video() {
                div { 
                    style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: black; z-index: 9999; display: flex; flex-direction: column; overflow: hidden; box-sizing: border-box;",
                    onmousemove: on_mouse_move,

                    // --- ZONE BOUTON RETOUR ---
                    div { 
                        class: if ui_visible() { "player-ui visible" } else { "player-ui hidden" },
                        
                        button { 
                            class: "btn-nav", 
                            onclick: {
                                let tx = cmd_tx.clone();
                                let mut list_sig = list_signal.clone();
                                move |_| {
                                    let time = current_time();
                                    let dur = current_duration();
                                    
                                    // 💾 1. On envoie au backend (SQLite sera mis à jour direct)
                                    tx.send(Command::UpdateProgress(media.id, time, dur)).unwrap();
                                    
                                    // 🚀 2. On met à jour la mémoire locale pour voir la barre rouge changer IMMÉDIATEMENT
                                    if let Some(m) = list_sig.write().iter_mut().find(|i| i.id == media.id) {
                                        m.last_position = time;
                                        if dur > 0.0 { m.duration = Some(dur); }
                                    }
                                    
                                    playing_video.set(None);
                                    current_time.set(0.0);
                                    current_duration.set(0.0);
                                }
                            }, 
                            "⬅ Retour" 
                        }
                    },

                    // 📡 INPUT INVISIBLE (Sert de pont entre le Javascript du lecteur et Rust)
                    input {
                        id: "neokodi-time-tracker",
                        r#type: "hidden",
                        oninput: move |evt| {
                            if let Some((t, d)) = evt.value().split_once(',') {
                                if let (Ok(time), Ok(dur)) = (t.parse::<f32>(), d.parse::<f32>()) {
                                    current_time.set(time);
                                    current_duration.set(dur);
                                }
                            }
                        }
                    }

                    // --- LE LECTEUR ---
                    div { 
                        style: "flex: 1; display: flex; align-items: center; justify-content: center; background: black; width: 100%; height: 100%; padding-bottom: 5px;",
                        {
                            let mut url = make_url(&media.path, &root_path);
                            // 🚀 L'ASTUCE : On ajoute #t=Secondes pour que la vidéo reprenne direct
                            if media.last_position > 2.0 {
                                url = format!("{}#t={}", url, media.last_position);
                            }
                            
                            rsx! {
                                video { 
                                    id: "neokodi-player",
                                    src: "{url}", 
                                    controls: true, 
                                    autoplay: true, 
                                    style: "width: 100%; height: 100%; object-fit: contain;" 
                                }
                                
                                // 🧠 SCRIPT AVEC LES ACCOLADES DOUBLÉES POUR NE PAS CRASHER RUST
                                script {
                                    "
                                    setTimeout(function() {{
                                        var v = document.getElementById('neokodi-player');
                                        var i = document.getElementById('neokodi-time-tracker');
                                        if(v && i) {{
                                            v.ontimeupdate = function() {{
                                                if(!isNaN(v.duration)) {{
                                                    i.value = v.currentTime + ',' + v.duration;
                                                    i.dispatchEvent(new Event('input', {{ bubbles: true }}));
                                                }}
                                            }};
                                        }}
                                    }}, 500);
                                    "
                                }
                            }
                        }
                    }
                }
            } 
            
            // ==========================================
            // 2. LA GRILLE DE VIDÉOS
            // ==========================================
            else {
                div { class: "top-bar", 
                    style: "display: flex; align-items: center; justify-content: space-between; height: 60px; padding: 0 20px; position: relative;",
                    
                    div { style: "z-index: 2;", Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" } }
                    div { class: "page-title-centered", "Vidéos" }
                    div { style: "z-index: 2;",
                        input {
                            r#type: "text", placeholder: "🔍 Rechercher...",
                            style: "padding: 8px; border-radius: 5px; border: none; background: #333; color: white; width: 250px;",
                            oninput: move |evt| search_text.set(evt.value()),
                        }
                    }
                }
                
                div { class: "media-grid",
                    for item in list_signal().iter().filter(|i| i.media_type == MediaType::Video).filter(|i| { let q = search_text().to_lowercase(); if q.is_empty() { return true; } i.title.as_deref().unwrap_or(&i.path).to_lowercase().contains(&q) }) 
                    {
                        {
                            let last_pos = item.last_position;
                            let dur = item.duration.unwrap_or(0.0);
                            
                            let progress = if dur > 0.0 { (last_pos / dur) * 100.0 } else { 0.0 };
                            let is_started = last_pos > 5.0;

                            rsx! {
                                div { 
                                    class: "media-card",
                                    title: "{item.title.as_deref().unwrap_or(&item.path)}",
                                    style: "position: relative; overflow: hidden; display: flex; flex-direction: column;",
                                    
                                    onclick: { 
                                        let selected = item.clone();
                                        let tx = cmd_tx.clone();
                                        let raw_title = item.title.as_deref().unwrap_or(&item.path);
                                        let clean_title = std::path::Path::new(raw_title).file_stem().and_then(|s| s.to_str()).unwrap_or(raw_title).to_string();
                                        let mut history = plugin_history.clone();
                                        move |_| { 
                                            history.write().insert(0, "Chargement des infos...".to_string());
                                            selected_media.set(Some(selected.clone()));
                                            tx.send(Command::GetfilmMetadataFromPlugin(clean_title.clone())).unwrap();
                                        } 
                                    },
                                    
                                    div { class: "card-icon", "🎬" }
                                    div { class: "card-text", "{item.title.as_deref().unwrap_or(&item.path)}" }
                                    
                                    if progress > 0.0 { 
                                        div { style: "position: absolute; bottom: 0; left: 0; width: 100%; height: 8px; background: rgba(0,0,0,0.8); z-index: 50;", 
                                            div { style: "height: 100%; background: #e50914; width: {progress.max(2.0):.1}%;" } 
                                        } 
                                    } else if is_started { 
                                        div { style: "position: absolute; bottom: 0; left: 0; width: 100%; height: 8px; background: rgba(0,0,0,0.8); z-index: 50;", 
                                            div { style: "height: 100%; background: #3498db; width: 100%;" } 
                                        } 
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ==========================================
            // 3. LE POPUP NETFLIX
            // ==========================================
            if let Some(media) = selected_media() {
                div { 
                    class: "modal-backdrop",
                    onclick: move |_| selected_media.set(None),
                    div { 
                        class: "modal-content",
                        onclick: move |evt| evt.stop_propagation(), 
                        {
                            let thumb_url = make_url(&media.path, &root_path).replace("/drives/", "/thumbnail/");
                            rsx! {
                                div { 
                                    class: "modal-header",
                                    style: "background-image: linear-gradient(to bottom, rgba(0,0,0,0.1), #181818), url('{thumb_url}'); background-size: cover; background-position: center; position: relative;", 
                                    div { class: "close-btn", onclick: move |_| selected_media.set(None), "✕" }
                                }
                            }
                        }
                        div { class: "modal-body",
                            h1 { style: "font-size: 3rem; margin-bottom: 10px; text-shadow: 2px 2px 4px black;", 
                                "{media.title.as_deref().unwrap_or(&media.path)}" 
                            }
                            div { style: "display: flex; gap: 15px; margin-bottom: 20px;",
                                button { 
                                    class: "btn-play",
                                    onclick: move |_| {
                                        let id = media.id;
                                        // 👇 CORRECTION : On passe l'objet média et non plus juste le path !
                                        playing_video.set(Some(media.clone()));
                                        selected_media.set(None);
                                        cmd_tx.send(Command::Play(id)).unwrap();
                                    },
                                    "▶ Lecture"
                                }
                            }
                            div { 
                                style: "background: #222; padding: 20px; border-radius: 8px; min-height: 100px;",
                                if let Some(info) = plugin_history().first() {
                                    pre { style: "white-space: pre-wrap; font-family: 'Segoe UI', sans-serif; color: #ccc; line-height: 1.6;", "{info}" }
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
    
    // 👇 NOUVEAU : On récupère le root_path pour générer les URLs locales via Warp
    let root_path_signal = use_context::<Signal<String>>();
    let root_path = root_path_signal();
    
    let mut current_image = use_signal(|| Option::<String>::None);
    let mut search_text = use_signal(|| String::new());

    let tx_init = cmd_tx.clone();
    use_hook(move || { if list_signal().is_empty() { tx_init.send(Command::GetAllMedia()).unwrap(); } });

    rsx! {
        div { class: "container",
            
            // 1. VUE PLEIN ÉCRAN
            if let Some(url) = current_image() {
                div { 
                    style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.95); z-index: 9999; display: flex; flex-direction: column;",
                    
                    div { style: "height: 60px; padding: 10px; position: absolute; z-index: 10000;",
                        button { class: "btn-nav", style: "position: relative; top: 0; left: 0; transform: none;", onclick: move |_| current_image.set(None), "⬅ Retour" }
                    }
                    
                    div { style: "flex: 1; min-height: 0; display: flex; align-items: center; justify-content: center; padding: 40px;",
                         img { src: "{url}", style: "max-width: 100%; max-height: 100%; object-fit: contain; box-shadow: 0 0 30px rgba(0,0,0,0.8);" }
                    }
                }
            } 
            // 2. GRILLE D'IMAGES
            else {
                div { class: "top-bar", 
                    style: "display: flex; align-items: center; justify-content: space-between; position: relative; height: 60px; padding: 0 20px;",

                    div { style: "z-index: 2;", Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" } }
                    
                    div { class: "page-title-centered", "Images" } 

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
                            i.title.as_deref().unwrap_or(&i.path).to_lowercase().contains(&query)
                        })
                    {
                        {
                            // On génère la vraie URL servie par Warp
                            let url = make_url(&item.path, &root_path);
                            
                            rsx! {
                                div { 
                                    class: "media-card",
                                    title: "{item.title.as_deref().unwrap_or(&item.path)}",
                                    // Style spécifique : on enlève le padding pour que l'image prenne toute la carte
                                    style: "padding: 0; overflow: hidden; position: relative; min-height: 200px; border: none;",
                                    
                                    onclick: {
                                        let u = url.clone(); 
                                        let i = item.id; 
                                        let tx = cmd_tx.clone();
                                        move |_| { 
                                            // 🚀 FINI LE BASE64 LOURD ! On passe juste l'URL. Le navigateur s'occupe du reste.
                                            current_image.set(Some(u.clone()));
                                            tx.send(Command::Play(i)).unwrap();
                                        }
                                    },
                                    
                                    // La miniature (en fond pour gérer le ratio "cover" sans déformer)
                                    div {
                                        style: "width: 100%; height: 100%; background-image: url('{url}'); background-size: cover; background-position: center; transition: transform 0.3s;"
                                    }
                                    
                                    // Le texte avec un léger flou de fond (Glassmorphism) pour toujours être lisible
                                    div { 
                                        class: "card-text", 
                                        style: "position: absolute; bottom: 0; left: 0; width: 100%; background: rgba(0,0,0,0.7); padding: 10px; margin: 0; backdrop-filter: blur(5px); font-size: 0.85rem;", 
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

// --- PLUGINS (ADDONS) ---
#[component] 
pub fn Plugins() -> Element { 
    let cmd_tx = use_context::<std::sync::mpsc::Sender<Command>>();
    let plugin_history = use_context::<Signal<Vec<String>>>();
    
    // On garde le texte de recherche
    let mut search_text = use_signal(|| String::from("Inception")); 
    
    // 👇 NOUVEAU : On gère le mode de recherche (Music ou Film)
    let mut search_mode = use_signal(|| "music"); // "music" ou "film"

    rsx! { 
        div { class: "container", 
            div { class: "top-bar", 
                Link { to: Route::Home {}, class: "btn-nav", "🏠 Accueil" }, 
                div { class: "page-title", "PLUGINS" } 
            }
            
            div { style: "display: flex; flex-direction: column; align-items: center; gap: 30px; margin-top: 50px; padding-bottom: 50px;",
                
                // --- SÉLECTEUR DE TYPE ---
                div { style: "display: flex; gap: 20px; background: #1e1e1e; padding: 5px; border-radius: 8px; border: 1px solid #333;",
                    button {
                        style: if search_mode() == "music" { 
                            "background: #007acc; color: white; padding: 10px 20px; border: none; border-radius: 5px; cursor: pointer; font-weight: bold;" 
                        } else { 
                            "background: transparent; color: #aaa; padding: 10px 20px; border: none; cursor: pointer;" 
                        },
                        onclick: move |_| { search_mode.set("music"); search_text.set("Nirvana".to_string()); },
                        "🎵 Musique (MusicBrainz)"
                    }
                    button {
                        style: if search_mode() == "film" { 
                            "background: #e50914; color: white; padding: 10px 20px; border: none; border-radius: 5px; cursor: pointer; font-weight: bold;" 
                        } else { 
                            "background: transparent; color: #aaa; padding: 10px 20px; border: none; cursor: pointer;" 
                        },
                        onclick: move |_| { search_mode.set("film"); search_text.set("Inception".to_string()); },
                        "🎬 Films (TMDB)"
                    }
                }

                // --- BARRE DE RECHERCHE ---
                div { style: "display: flex; gap: 10px;",
                    input {
                        style: "padding: 10px; border-radius: 4px; border: 1px solid #333; background: #1e1e1e; color: white; width: 300px;",
                        value: "{search_text}",
                        placeholder: if search_mode() == "music" { "Nom de l'artiste..." } else { "Titre du film..." },
                        oninput: move |evt| search_text.set(evt.value())
                    }
                    button { 
                        class: "btn-nav", 
                        style: "position: relative; transform: none; top: auto; left: auto; background: #27ae60;",
                        onclick: move |_| {
                            if !search_text().is_empty() {
                                // 👇 LOGIQUE DE CHOIX DU PLUGIN
                                if search_mode() == "music" {
                                    cmd_tx.send(Command::GetArtistMetadataFromPlugin(search_text())).unwrap();
                                } else {
                                    cmd_tx.send(Command::GetfilmMetadataFromPlugin(search_text())).unwrap();
                                }
                            }
                        },
                        "🔍 Rechercher"
                    }
                }

                // --- RÉSULTATS ---
                div { style: "width: 80%; max-width: 800px; display: flex; flex-direction: column; gap: 20px;",
                    if plugin_history().is_empty() {
                        div { style: "text-align: center; color: #666; font-style: italic; margin-top: 20px;", 
                            "Aucune recherche pour le moment..." 
                        }
                    }

                    for res in plugin_history().iter() {
                        div { 
                            // Petit changement de style selon le contenu (simple detection)
                            style: if res.contains("Year:") { 
                                "background: #1e1e1e; padding: 20px; border-radius: 8px; border: 1px solid #333; border-left: 5px solid #e50914;" // Rouge pour Films
                            } else {
                                "background: #1e1e1e; padding: 20px; border-radius: 8px; border: 1px solid #333; border-left: 5px solid #007acc;" // Bleu pour Musique
                            },
                            h3 { style: "margin-top: 0; color: #aaa; font-size: 0.9rem; text-transform: uppercase;", "Résultat :" }
                            pre { 
                                style: "color: #eee; white-space: pre-wrap; font-family: 'Segoe UI', sans-serif; font-size: 1rem; margin: 0;",
                                "{res}"
                            }
                        }
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