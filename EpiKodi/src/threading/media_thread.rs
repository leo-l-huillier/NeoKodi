
/*
This file manages the media thread, which handles media playback commands
*/

use crate::library::media_library::MediaLibrary;

use super::command::Command;
use super::command::Event;


use std::thread;
use std::sync::{Arc, Mutex, mpsc};
use std::path::PathBuf;


pub fn launch_media_thread(cmd_rx: mpsc::Receiver<Command>, evt_tx: mpsc::Sender<Event>) {

    let library = Arc::new(Mutex::new(MediaLibrary::new()));
    let lib_thread = Arc::clone(&library);

    // let media_thread =
    thread::spawn(move || {
        let mut library = lib_thread.lock().unwrap();

        library.init();
        //library.play_id(3);

        drop(library);

        loop {
            // TODO handle errors
            match cmd_rx.recv() {
                // le mutex se drop en sortant du scope
                
                Ok(Command::AddSource(path, media_type)) => {
                    let mut library = lib_thread.lock().unwrap();

                    library.add_source(path, media_type);
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



                Ok(Command::Play(id)) => {
                    let mut library = lib_thread.lock().unwrap();
                    library.play_id(id);
                    evt_tx.send(Event::NowPlaying(id)).unwrap();
                }
                Ok(Command::Pause(id)) => {
                    println!("in thread pause 1");
                    let mut library = lib_thread.lock().unwrap();
                    println!("in thread pause 2");
                    library.pause_id(id);
                    println!("in thread pause 3");
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

                Err(_) => break,
            }
        }
    });
}
