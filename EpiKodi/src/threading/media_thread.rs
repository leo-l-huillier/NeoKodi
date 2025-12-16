
/*
This file manages the media thread, which handles media playback commands
*/

use crate::library::media_library::MediaLibrary;

use super::command::Command;
use super::command::Event;


use std::thread;
use std::sync::{Arc, Mutex, mpsc};


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
            match cmd_rx.recv() {
                // le mutex se drop en sortant du scope
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
                    evt_tx.send(Event::Data(info)).unwrap();
                }
                Err(_) => break,
            }
        }
    });
}
