
/*
In this file we handle the media library, calling the scan functions for different sources,
and store the found media in a data structure
*/

use crate::database::db::DB;

use crate::media::data::Media;
use crate::media::data::MediaType;
use crate::media::audio::Audio;
use crate::media::image::Image;
use crate::media::video::Video;

use std::fs;
use std::collections::HashMap;
use std::path::Path;

use dioxus::html::p;
use rusqlite::{Connection};

use crate::constants::constants::MEDIA_DB_FILE;
use crate::scan::scan::Scan;


#[derive(Debug, Clone)]
pub struct ScannedMedia {
    pub path: String,
    pub name: String,
    pub duration: f32,
    pub media_type: MediaType,
}
pub struct MediaLibrary {
    pub items: HashMap<i64, Box<dyn Media>>,
    pub scan_lib: Scan,
    database: DB,
}

impl MediaLibrary {
    pub fn new() -> Self {

        Self {
            items: HashMap::new(),
            scan_lib: Scan::new(),
            database: DB::new(),
        }
    }

    pub fn init(&mut self) {

        self.database.init_db().unwrap();
        self.scan_lib.scan_libraries();

        self.database.upsert_media_from_scan(self.scan_lib.scan.clone()).unwrap(); //TODO: ce clone me fait chier, il faudrait qu'on utilise juste scan (ca serait meme mieux si on donne la valeur direct comme ca il se fait drop (on en a plus besoin ) et mm en terme de performance c'est pas terrible parce que c'est un gros object )
        self.database.cleanup_missing_media(self.scan_lib.scan.clone()).unwrap(); // TODO to implement, shuld be called every scans
        //self.database.print_media_rows();
        self.database.get_all_media().unwrap();
        //self.database.print_media_rows();

     
        
        for row in self.database.media_rows.iter() {
            let media: Box<dyn Media> = match row.media_type {
                MediaType::Audio => Box::new(Audio::new(&row.path,&row.title.as_deref().unwrap_or(""))),
                MediaType::Video => Box::new(Video::new(&row.path,&row.title.as_deref().unwrap_or(""))),
                MediaType::Image => Box::new(Image::new(&row.path,&row.title.as_deref().unwrap_or(""))),
            };
            self.items.insert(row.id, media);
        }

    }
   

    //=========== MEDIA FUNCTIONS ===========

    pub fn play_id(&mut self, id: i64) {
        if let Some(item) = self.items.get_mut(&id) {
            println!("Playing media ID {id}: ");
            item.init();
            item.play();
        } else {
            println!("Error: media with ID {id} not found.");
        }
    }

    pub fn pause_id(&mut self, id: i64) {
        println!("in library pause");
        if let Some(item) = self.items.get_mut(&id) {
            println!("in library pause");
            item.pause();
        } else {
            println!("in library pause error");
            println!("Error: media with ID {id} not found.");
        }
    }

    pub fn resume_id(&mut self, id: i64) {
        if let Some(item) = self.items.get_mut(&id) {
            item.resume();
        } else {
            println!("Error: media with ID {id} not found.");
        }
    }

        pub fn stop_id(&mut self, id: i64) {
        if let Some(item) = self.items.get_mut(&id) {
            println!("Stopping media ID {id}: {}", item.info());
            item.stop();
        } else {
            println!("Error: media with ID {id} not found.");
        }
    }


    pub fn info_id(&self, id: i64) -> Option<String> {
        if let Some(item) = self.items.get(&id) {
            Some(item.info())
        } else {
            None
        }
    }

    pub fn media_type_id(&self, id: i64) -> Option<crate::media::data::MediaType> {
        if let Some(item) = self.items.get(&id) {
            Some(item.media_type())
        } else {
            None
        }
    }

}


