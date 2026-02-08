
/*
This file manages the media thread, which handles media playback commands
*/


use crate::library::media_library::MediaLibrary;
use super::command::Command;
use super::command::Event;
use crate::media::data::MediaType;

use crate::plugin::plugin_manager::PluginManager;

use crate::constants::constants::{PLAYING};

use std::thread;
use std::sync::{Arc, Mutex, mpsc};
use std::path::PathBuf;

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
            // TODO handle errors
            match cmd_rx.recv() {
                // le mutex se drop en sortant du scope
                
                //TODO - virer cette merde
                // this thing is a heresy.... can't stay like this
                Ok(Command::ChangeLibraryPath(path)) => {
                    println!("🔄 REÇU COTÉ BACKEND : CHANGEMENT DE RACINE vers {:?}", path);
                    let mut library = lib_thread.lock().unwrap();

                    library.clear();

                    evt_tx.send(Event::MediaList(Vec::new())).unwrap();

                    library.add_source(path.clone(), MediaType::Video);
                    library.add_source(path.clone(), MediaType::Audio);
                    library.add_source(path, MediaType::Image);

                    evt_tx.send(Event::MediaList(library.get_all_media())).unwrap();
                }

                Ok(Command::ChangeLibraryPath(path)) => {
                    println!("🔄 REÇU COTÉ BACKEND : CHANGEMENT DE RACINE vers {:?}", path);
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

                Ok(Command::GetPlaylistId(name)) => {
                    let mut library = lib_thread.lock().unwrap();
                    let playlist_id = library.get_playlist_id(&name);
                    evt_tx.send(Event::Data(playlist_id.to_string())).unwrap();
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