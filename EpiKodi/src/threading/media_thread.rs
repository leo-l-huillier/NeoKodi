
/*
This file manages the media thread, which handles media playback commands
*/

use crate::library::media_library::MediaLibrary;
use super::command::{Command, Event};
use crate::media::data::MediaType;
use crate::iptv::parser::parse_m3u;

use crate::plugin::plugin_manager::PluginManager;

use crate::constants::constants::{PLAYING};

use std::thread;
use std::sync::{Arc, Mutex, mpsc};
use std::path::PathBuf;

// 👇 IMPORTS POUR LES PLUGINS (DLL)
use libloading::{Library, Symbol};
use std::ffi::{CString, CStr};
use std::os::raw::c_char;

pub fn launch_media_thread(cmd_rx: mpsc::Receiver<Command>, evt_tx: mpsc::Sender<Event>) {

    let library = Arc::new(Mutex::new(MediaLibrary::new()));
    let lib_thread = Arc::clone(&library);
    let mut plugin_manager = PluginManager::new();
    plugin_manager.load_plugins();
    
    // let media_thread =
    thread::spawn(move || {
        let mut library = lib_thread.lock().unwrap();
        library.init();


        // ----TESTS----
        //library.play_id(3);
        //library.update_media_status_and_time(1, PLAYING, 100.0);
        drop(library);

        loop {
            // On attend une commande...
            if let Ok(command) = cmd_rx.recv() {
                match command {
                    
                    // ========================================================
                    // 👇 GESTION DES PLUGINS (MODE BRUT / EFFICACE)
                    // ========================================================
                    Command::GetArtistInfo(artist_name) => {
                        println!("🔌 THREAD: Appel plugin pour '{}'", artist_name);

                        // 1. Détection de l'extension selon l'OS (Windows/Linux/Mac)
                        #[cfg(target_os = "windows")]
                        let lib_path = "plugins/musicbrainz_plugin.dll";
                        #[cfg(target_os = "linux")]
                        let lib_path = "plugins/libmusicbrainz_plugin.so";
                        #[cfg(target_os = "macos")]
                        let lib_path = "plugins/libmusicbrainz_plugin.dylib";

                        unsafe {
                            // 2. Chargement du fichier DLL
                            match Library::new(lib_path) {
                                Ok(lib) => {
                                    // 3. Récupération des fonctions "greet" et "free_string"
                                    if let Ok(greet) = lib.get::<GreetFunc>(b"greet\0") {
                                        if let Ok(free_string) = lib.get::<FreeStringFunc>(b"free_string\0") {
                                            
                                            // 4. Appel de la fonction C
                                            let c_input = CString::new(artist_name).unwrap();
                                            let result_ptr = greet(c_input.as_ptr());
                                            
                                            // 5. Conversion du résultat en String Rust
                                            let result = CStr::from_ptr(result_ptr).to_string_lossy().to_string();
                                            
                                            // 6. Nettoyage mémoire (Côté C)
                                            free_string(result_ptr);

                                            // 7. Envoi du résultat à l'interface
                                            println!("✅ THREAD: Résultat reçu -> {}", result);
                                            evt_tx.send(Event::ArtistInfoReceived(result)).unwrap();
                                        }
                                    }
                                }
                                Err(e) => {
                                    let err_msg = format!("Erreur chargement DLL ({}): {}", lib_path, e);
                                    println!("❌ {}", err_msg);
                                    evt_tx.send(Event::ArtistInfoReceived(err_msg)).unwrap();
                                }
                            }
                        }
                    }

                    // ========================================================
                    // 👇 GESTION DE LA BIBLIOTHÈQUE
                    // ========================================================

                    Command::ChangeLibraryPath(path) => {
                        println!("🔄 REÇU COTÉ BACKEND : CHANGEMENT DE RACINE vers {:?}", path);
                        let mut library = lib_thread.lock().unwrap();

                        // 1. On vide tout
                        library.clear();
                        evt_tx.send(Event::MediaList(Vec::new())).unwrap();

                        // 2. On scanne les 3 types de médias
                        library.add_source(path.clone(), MediaType::Video);
                        library.add_source(path.clone(), MediaType::Audio);
                        library.add_source(path, MediaType::Image);

                        // 3. On renvoie la liste mise à jour
                        evt_tx.send(Event::MediaList(library.get_all_media())).unwrap();
                    }

                    Command::AddSource(path, media_type) => {
                        let mut library = lib_thread.lock().unwrap();
                        library.add_source(path, media_type);
                    }

                    Command::GetAllMedia() => {
                        let library = lib_thread.lock().unwrap();
                        let media_list = library.get_all_media();
                        evt_tx.send(Event::MediaList(media_list)).unwrap();
                    }

                    Command::GetMediaFromPath(path) => {
                        let mut library = lib_thread.lock().unwrap();
                        let media_list = library.get_media_from_path(path);
                        evt_tx.send(Event::MediaList(media_list)).unwrap();
                    }

                    Command::GetMediaFromType(media_type) => {
                        let library = lib_thread.lock().unwrap();
                        let media_list = library.get_media_by_type(media_type);
                        evt_tx.send(Event::MediaList(media_list)).unwrap();
                    }

                    // ========================================================
                    // 👇 GESTION TAGS / PLAYLISTS
                    // ========================================================

                    Command::GetMediaFromTag(tag_name) => {
                        let mut library = lib_thread.lock().unwrap();
                        let media_list = library.get_media_from_tag(&tag_name);
                        evt_tx.send(Event::IDList(media_list)).unwrap();
                    }

                    Command::GetMediaFromPlaylist(playlist_id) => {
                        let mut library = lib_thread.lock().unwrap();
                        let media_list = library.get_media_from_playlist(playlist_id);
                        evt_tx.send(Event::IDList(media_list)).unwrap();
                    }

                    Command::LoadM3U(url) => {
                        println!("📡 Téléchargement de la playlist : {}", url);
                        
                        // On télécharge le contenu (opération bloquante, acceptable ici)
                        match reqwest::blocking::get(&url) {
                            Ok(resp) => {
                                if let Ok(content) = resp.text() {
                                    println!("✅ Playlist téléchargée, analyse en cours...");
                                    let channels = parse_m3u(&content);
                                    println!("📺 {} chaînes trouvées !", channels.len());
                                    
                                    // On envoie la liste au Front
                                    evt_tx.send(Event::M3UList(channels)).unwrap();
                                }
                            },
                            Err(e) => println!("❌ Erreur téléchargement M3U : {}", e),
                        }
                    }

                    // ========================================================
                    // 👇 GESTION LECTURE (PLAYER)
                    // ========================================================

                    Command::Play(id) => {
                        let mut library = lib_thread.lock().unwrap();
                        library.play_id(id);
                        evt_tx.send(Event::NowPlaying(id)).unwrap();
                    }

                    Command::Pause(id) => {
                        let mut library = lib_thread.lock().unwrap();
                        library.pause_id(id);
                    }

                    Command::Resume(id) => {
                        let mut library = lib_thread.lock().unwrap();
                        library.resume_id(id);
                        evt_tx.send(Event::NowPlaying(id)).unwrap();
                    }

                    Command::Stop(id) => {
                        let mut library = lib_thread.lock().unwrap();
                        library.stop_id(id);
                    }

                    Command::Info(id) => {
                        let library = lib_thread.lock().unwrap();
                        let info = library.info_id(id).unwrap();
                        evt_tx.send(Event::Info(info)).unwrap();
                    }

                    // ========================================================
                    // 👇 GESTION MODIFICATIONS (TAGS, PLAYLISTS)
                    // ========================================================

                    Command::AddTag(tag_name) => {
                        let mut library = lib_thread.lock().unwrap();
                        library.add_tag(&tag_name);
                    }

                    Command::GetTagId(tag_name) => {
                        let mut library = lib_thread.lock().unwrap();
                        let tag_id = library.get_tag_id(&tag_name);
                        evt_tx.send(Event::Data(tag_id.to_string())).unwrap();
                    }

                    Command::AddTagToMedia(media_id, tag_id) => {
                        let mut library = lib_thread.lock().unwrap();
                        library.add_tag_to_media(media_id, tag_id);
                    }

                    Command::AddPlaylist(name) => {
                        let mut library = lib_thread.lock().unwrap();
                        library.create_playlist(&name);
                    }

                    Command::AddMediaToPlaylist(media_id, playlist_id) => {
                        let mut library = lib_thread.lock().unwrap();
                        library.add_media_to_playlist(media_id, playlist_id);
                    }

                    Command::GetPlaylistId(name) => {
                        let mut library = lib_thread.lock().unwrap();
                        let playlist_id = library.get_playlist_id(&name);
                        evt_tx.send(Event::Data(playlist_id.to_string())).unwrap();
                    }
                }

                Ok(Command::AddSource(path, media_type)) => {
                    let mut library = lib_thread.lock().unwrap();
                    
                    library.add_source(path, media_type);
                }

                Ok(Command::RemoveSource(path, media_type)) => {
                    let mut library = lib_thread.lock().unwrap();
                    
                    library.remove_source(path, media_type);
                }

                Ok(Command::GetAllMedia()) => {
                    let library = lib_thread.lock().unwrap();
                    let media_list = library.get_all_media();
                    evt_tx.send(Event::MediaList(media_list)).unwrap();
                }

                Ok(Command::GetMediaFromPath(path)) => {
                    let mut library = lib_thread.lock().unwrap();
                    let media_list = library.get_media_from_path(path);
                    evt_tx.send(Event::MediaList(media_list)).unwrap();
                }

                Ok(Command::GetMediaFromType(media_type)) => {
                    let library = lib_thread.lock().unwrap();
                    let media_list = library.get_media_by_type(media_type);
                    evt_tx.send(Event::MediaList(media_list)).unwrap();
                }

                Ok(Command::GetMediaFromTag(tag_name)) => {
                    let mut library = lib_thread.lock().unwrap();
                    let media_list = library.get_media_from_tag(&tag_name);
                    // For simplicity, we just send the count of media items found
                    evt_tx.send(Event::IDList(media_list)).unwrap();
                }

                Ok(Command::GetMediaFromPlaylist(playlist_id)) => {
                    let mut library = lib_thread.lock().unwrap();
                    let media_list = library.get_media_from_playlist(playlist_id);
                    // For simplicity, we just send the count of media items found
                    evt_tx.send(Event::IDList(media_list)).unwrap();
                }

                Ok(Command::UpdateMediaState(media_id, status, time_stop)) => {
                    let mut library = lib_thread.lock().unwrap();
                    library.update_media_status_and_time(media_id, status, time_stop);
                }

                Ok(Command::Reload()) => {
                    let mut library = lib_thread.lock().unwrap();
                    library.reload();
                    let media_list = library.get_all_media();
                    evt_tx.send(Event::MediaList(media_list)).unwrap();
                }

                Ok(Command::Play(id)) => {
                    let mut library = lib_thread.lock().unwrap();
                    library.play_id(id);
                    evt_tx.send(Event::NowPlaying(id)).unwrap();
                }

                Ok(Command::Pause(id)) => {
                    let mut library = lib_thread.lock().unwrap();
                    library.pause_id(id);
                }

                Ok(Command::Resume(id)) => {
                    let mut library = lib_thread.lock().unwrap();
                    library.resume_id(id);
                    evt_tx.send(Event::NowPlaying(id)).unwrap();
                }

                Ok(Command::Stop(id)) => {
                    let mut library = lib_thread.lock().unwrap();
                    library.stop_id(id);
                }

                Ok(Command::Info(id)) => {
                    let library = lib_thread.lock().unwrap();
                    let info = library.info_id(id).unwrap();
                    evt_tx.send(Event::Info(info)).unwrap();
                }

                Ok(Command::AddTag(tag_name)) => {
                    let mut library = lib_thread.lock().unwrap();
                    library.add_tag(&tag_name);
                }

                Ok(Command::GetTagId(tag_name)) => {
                    let mut library = lib_thread.lock().unwrap();
                    let tag_id = library.get_tag_id(&tag_name);
                    evt_tx.send(Event::Data(tag_id.to_string())).unwrap();
                }

                Ok(Command::AddTagToMedia(media_id, tag_id)) => {
                    let mut library = lib_thread.lock().unwrap();
                    library.add_tag_to_media(media_id, tag_id);
                }

                Ok(Command::AddPlaylist(name)) => {
                    let mut library = lib_thread.lock().unwrap();
                    library.create_playlist(&name);
                }

                Ok(Command::AddMediaToPlaylist(media_id, playlist_id)) => {
                    let mut library = lib_thread.lock().unwrap();
                    library.add_media_to_playlist(media_id, playlist_id);
                }

                Ok(Command::DeletePlaylist(playlist_id)) => {
                    let mut library = lib_thread.lock().unwrap();
                    library.delete_playlist(playlist_id);
                }

                Ok(Command::RemoveMediaFromPlaylist(media_id, playlist_id)) => {
                    let mut library = lib_thread.lock().unwrap();
                    library.remove_media_from_playlist(media_id, playlist_id);
                }

                Ok(Command::GetPlaylistId(name)) => {
                    let mut library = lib_thread.lock().unwrap();
                    let playlist_id = library.get_playlist_id(&name);
                    evt_tx.send(Event::Data(playlist_id.to_string())).unwrap();
                }

                Ok(Command::GetAllPlaylists()) => {
                    let mut library = lib_thread.lock().unwrap();
                    let playlists = library.get_all_playlists();
                    // For simplicity, we just send the count of playlists found
                    evt_tx.send(Event::PlaylistList(playlists)).unwrap();
                }

                Ok(Command::GetArtistMetadataFromPlugin(name)) => {
                    let response = plugin_manager.get_metadata(name.as_str());
                    evt_tx.send(Event::Data(response.to_string())).unwrap();
                }

                Err(_) => break,
            }
        }
    });
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    fn setup_thread() -> (mpsc::Sender<Command>, mpsc::Receiver<Event>) {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let (evt_tx, evt_rx) = mpsc::channel();

        launch_media_thread(cmd_rx, evt_tx);

        thread::sleep(Duration::from_millis(100));

        (cmd_tx, evt_rx)
    }

    #[test]
    fn test_add_source_command() {
        let (cmd_tx, _evt_rx) = setup_thread();
        let path = PathBuf::from("/test/music");
        
        cmd_tx.send(Command::AddSource(path, crate::media::data::MediaType::Audio)).unwrap();
        thread::sleep(Duration::from_millis(50));
        // Command processed without panic
    }

    #[test]
    fn test_get_all_media_command() {
        let (cmd_tx, evt_rx) = setup_thread();
        
        cmd_tx.send(Command::GetAllMedia()).unwrap();
        
        match evt_rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Event::MediaList(_)) => {},
            _ => panic!("Expected MediaList event"),
        }
    }

    #[test]
    fn test_get_media_from_path_command() {
        let (cmd_tx, evt_rx) = setup_thread();
        let path = PathBuf::from("/test");
        
        cmd_tx.send(Command::GetMediaFromPath(path)).unwrap();
        
        match evt_rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Event::MediaList(_)) => {},
            _ => panic!("Expected MediaList event"),
        }
    }

    #[test]
    fn test_get_media_from_type_command() {
        let (cmd_tx, evt_rx) = setup_thread();
        
        cmd_tx.send(Command::GetMediaFromType(crate::media::data::MediaType::Audio)).unwrap();
        
        match evt_rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Event::MediaList(_)) => {},
            _ => panic!("Expected MediaList event"),
        }
    }

    #[test]
    fn test_get_media_from_tag_command() {
        let (cmd_tx, evt_rx) = setup_thread();
        
        cmd_tx.send(Command::GetMediaFromTag("action".to_string())).unwrap();
        
        match evt_rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Event::IDList(_)) => {},
            _ => panic!("Expected IDList event"),
        }
    }

    #[test]
    fn test_get_media_from_playlist_command() {
        let (cmd_tx, evt_rx) = setup_thread();
        
        cmd_tx.send(Command::GetMediaFromPlaylist(1)).unwrap();
        
        match evt_rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Event::IDList(_)) => {},
            _ => panic!("Expected IDList event"),
        }
    }

    #[test]
    fn test_play_command() {
        let (cmd_tx, evt_rx) = setup_thread();
        
        cmd_tx.send(Command::Play(1)).unwrap();
        
        match evt_rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Event::NowPlaying(id)) => assert_eq!(id, 1),
            _ => panic!("Expected NowPlaying event"),
        }
    }

    #[test]
    fn test_pause_command() {
        let (cmd_tx, _evt_rx) = setup_thread();
        
        cmd_tx.send(Command::Pause(1)).unwrap();
        thread::sleep(Duration::from_millis(50));
        // Command processed without panic
    }

    #[test]
    fn test_resume_command() {
        let (cmd_tx, evt_rx) = setup_thread();
        
        cmd_tx.send(Command::Resume(1)).unwrap();
        
        match evt_rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Event::NowPlaying(id)) => assert_eq!(id, 1),
            _ => panic!("Expected NowPlaying event"),
        }
    }

    #[test]
    fn test_stop_command() {
        let (cmd_tx, _evt_rx) = setup_thread();
        
        cmd_tx.send(Command::Stop(1)).unwrap();
        thread::sleep(Duration::from_millis(50));
        // Command processed without panic
    }

    #[test]
    #[ignore]
    fn test_info_command() {
        let (cmd_tx, evt_rx) = setup_thread();
        
        cmd_tx.send(Command::Info(1)).unwrap();
        
        match evt_rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Event::Info(_)) => {},
            _ => panic!("Expected Info event"),
        }
    }

    #[test]
    fn test_add_tag_command() {
        let (cmd_tx, _evt_rx) = setup_thread();
        
        cmd_tx.send(Command::AddTag("favorite".to_string())).unwrap();
        thread::sleep(Duration::from_millis(50));
        // Command processed without panic
    }

    #[test]
    fn test_get_tag_id_command() {
        let (cmd_tx, evt_rx) = setup_thread();
        
        cmd_tx.send(Command::GetTagId("action".to_string())).unwrap();
        
        match evt_rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Event::Data(_)) => {},
            _ => panic!("Expected Data event"),
        }
    }

    #[test]
    fn test_add_tag_to_media_command() {
        let (cmd_tx, _evt_rx) = setup_thread();
        
        cmd_tx.send(Command::AddTagToMedia(1, 1)).unwrap();
        thread::sleep(Duration::from_millis(50));
        // Command processed without panic
    }

    #[test]
    fn test_add_playlist_command() {
        let (cmd_tx, _evt_rx) = setup_thread();
        
        cmd_tx.send(Command::AddPlaylist("My Playlist".to_string())).unwrap();
        thread::sleep(Duration::from_millis(50));
        // Command processed without panic
    }

    #[test]
    fn test_add_media_to_playlist_command() {
        let (cmd_tx, _evt_rx) = setup_thread();
        
        cmd_tx.send(Command::AddMediaToPlaylist(1, 1)).unwrap();
        thread::sleep(Duration::from_millis(50));
        // Command processed without panic
    }

    #[test]
    fn test_get_playlist_id_command() {
        let (cmd_tx, evt_rx) = setup_thread();
        
        cmd_tx.send(Command::GetPlaylistId("My Playlist".to_string())).unwrap();
        
        match evt_rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Event::Data(_)) => {},
            _ => panic!("Expected Data event"),
        }
    }

    #[test]
    fn test_thread_exits_on_channel_close() {
        let (cmd_tx, _evt_rx) = setup_thread();
        drop(cmd_tx);
        thread::sleep(Duration::from_millis(100));
        // Thread should exit gracefully
    }

    #[test]
    fn test_multiple_sequential_commands() {
        let (cmd_tx, evt_rx) = setup_thread();
        
        cmd_tx.send(Command::AddTag("action".to_string())).unwrap();
        cmd_tx.send(Command::GetTagId("action".to_string())).unwrap();
        
        match evt_rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Event::Data(_)) => {},
            _ => panic!("Expected Data event"),
        }
    }
}