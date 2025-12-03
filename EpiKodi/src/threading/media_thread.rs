


use crate::database::media_library::MediaLibrary;

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

        library.scan_libraries();
        library.debug_print_items();
        library.play_id(3);

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
                    evt_tx.send(Event::Data(info)).unwrap();
                }
                Ok(Command::MediaScan(_)) => {
                    let mut library = lib_thread.lock().unwrap();
                    library.scan_libraries();
                }
                Err(_) => break,
            }
        }
    });
}
