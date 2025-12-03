mod media;
use media::data::Media;
use media::video::Video;
use media::audio::Audio;

mod database;
use database::media_library::MediaLibrary;
use database::sources::LibraryConfig;

use std::thread;
use std::time::Duration;
use std::thread::sleep;

use std::sync::{Arc, Mutex, mpsc};

enum Command {
    Play(u32),
    Pause(u32),
    Resume(u32),
    Stop(u32),
    Info(u32),
    MediaScan(u32),
}


fn main() {

    let library = Arc::new(Mutex::new(MediaLibrary::new()));
    let (tx, rx) = mpsc::channel::<Command>();

    let lib_thread = Arc::clone(&library);

    let media_thread = thread::spawn(move || {
        let mut library = lib_thread.lock().unwrap();

        library.scan_libraries();
        library.debug_print_items();
        library.play_id(3);

        drop(library);

        loop {
            match rx.recv() {
                // le mutex se drop en sortant du scope
                Ok(Command::Play(id)) => {
                    let mut library = lib_thread.lock().unwrap();
                    library.play_id(id);
                }
                Ok(Command::Pause(id)) => {
                    let mut library = lib_thread.lock().unwrap();
                    library.pause_id(id);
                }
                Ok(Command::Resume(id)) => {
                    let mut library = lib_thread.lock().unwrap();
                    library.resume_id(id);
                }
                Ok(Command::Stop(id)) => {
                    let mut library = lib_thread.lock().unwrap();
                    library.stop_id(id);
                }
                Ok(Command::Info(id)) => {
                    let library = lib_thread.lock().unwrap();
                    library.info_id(id);
                }
                Ok(Command::MediaScan(_)) => {
                    let mut library = lib_thread.lock().unwrap();
                    library.scan_libraries();
                }
                Err(_) => break,
            }
        }
    });

    let mut i = 0;

    loop {
        // Simulate GUI

        println!("GUI working...");
        sleep(Duration::from_secs(1));
        if i==10 {
            tx.send(Command::Pause(3)).unwrap();

        }
        if i==13 {
            tx.send(Command::Resume(3)).unwrap();
        }
        i += 1;
    }
}