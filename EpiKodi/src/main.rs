
/*
main.rs
*/
mod media;
mod database;
mod threading;
mod constants;

use threading::media_thread::launch_media_thread;
use threading::command::Command;
use threading::command::Event;

use std::time::Duration;
use std::thread::sleep;
use std::sync::mpsc;

fn main() {

    // ========== MEDIA THREADING ===========

    let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();
    let (evt_tx, evt_rx) = mpsc::channel::<Event>();
    launch_media_thread(cmd_rx, evt_tx);

    // ========== GUI ===========

    let mut i = 0;
    loop {
        // Simulate GUI

        println!("GUI working...");
        sleep(Duration::from_secs(1));
        let id = 55;
        if i==2 {
            cmd_tx.send(Command::Play(id)).unwrap();
            cmd_tx.send(Command::Info(id)).unwrap();

            if let Ok(event) = evt_rx.try_recv() {
                match event {
                    Event::Finished(id) => println!("Media finished item {id}"),
                    Event::NowPlaying(msg) => println!("MEDIA says: {msg}"),
                    Event::Data(info) => println!("MEDIA info: {info}"),
            }
        }
        }
        if i==5 {
            cmd_tx.send(Command::Pause(3)).unwrap();
        }
        if i==8 {
            cmd_tx.send(Command::Resume(3)).unwrap();
        }
        if i==10 {
            break;
        }
        i += 1;
    }
}