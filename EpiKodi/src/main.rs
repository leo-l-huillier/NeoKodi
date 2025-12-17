
/*
main.rs
*/


mod threading;
mod constants;

//======== MEDIA THREADING ========
mod media;
use threading::media_thread::launch_media_thread;
use threading::command::Command;
use threading::command::Event;

//======== DATABASE ========
mod database;
use database::db::DB;
use rusqlite::{Connection};

mod library;
mod scan;

//======== GUI ========
mod gui;
use gui::route::Route;

use std::time::Duration;
use std::thread::sleep;
use std::sync::mpsc;


use dioxus::prelude::*;
use dioxus::desktop::{Config, WindowBuilder};
use dioxus_router::prelude::*;

use crate::constants::constants::MEDIA_DB_FILE;



fn main() {

    // ========== MEDIA THREADING ===========


    let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();
    let (evt_tx, evt_rx) = mpsc::channel::<Event>();
    launch_media_thread(cmd_rx, evt_tx);

    // ========== GUI ===========


    /*
    let config = Config::new().with_window(WindowBuilder::new().with_title("NeoKodi").with_resizable(true));

    // 2. Lancer l'application
    LaunchBuilder::desktop().with_cfg(config).launch(|| rsx! { Router::<Route> {} });
    */

    // ========== GUI back test ===========
     
    let mut i = 0;
    loop {
        // Simulate GUI

        //println!("GUI working...");
        sleep(Duration::from_secs(1));
        let id = 30;
        if i==2 {
            cmd_tx.send(Command::Play(id)).unwrap();
            //cmd_tx.send(Command::Info(id)).unwrap();

            if let Ok(event) = evt_rx.try_recv() {
                match event {
                    Event::Finished(id) => println!("Media finished item {id}"),
                    Event::NowPlaying(msg) => println!("MEDIA says: {msg}"),
                    Event::Data(info) => println!("MEDIA info: {info}"),
                    Event::IDList(ids) => println!("MEDIA ID List: {:?}", ids),
                    Event::Info(info) => println!("MEDIA Info: {:?}", info),
                    //medialist
                    _ => {}
                    
                }
            }
        }
        if i==5 {
            cmd_tx.send(Command::Pause(id)).unwrap();
        }
        if i==8 {
            cmd_tx.send(Command::Resume(id)).unwrap();
        }
        if i==10 {
            break;
        }
        i += 1;
        
    }
}

