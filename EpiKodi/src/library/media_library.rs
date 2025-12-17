
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
use crate::media::data::MediaInfo;


use std::fs;
use std::collections::HashMap;
use std::path::Path;

use dioxus::html::p;
use rusqlite::{Connection};

use crate::constants::constants::MEDIA_DB_FILE;
use crate::scan::scan::Scan;
use std::path::PathBuf;


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
    pub database: DB,
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

        // scan the libraries
        self.database.init_db().unwrap();
        self.scan_lib.scan_libraries();

        // update the database
        self.database.upsert_media_from_scan(self.scan_lib.scan.clone()).unwrap(); //TODO: ce clone me fait chier, il faudrait qu'on utilise juste scan (ca serait meme mieux si on donne la valeur direct comme ca il se fait drop (on en a plus besoin ) et mm en terme de performance c'est pas terrible parce que c'est un gros object )
        self.database.cleanup_missing_media(self.scan_lib.scan.clone()).unwrap(); // TODO to implement, shuld be called every scans
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



    // =========== PLAYLISTS FUNCTIONS ===========
    

    pub fn create_playlist(&mut self, name: &str) {

        match self.database.create_playlist(name) {
            Ok(playlist_id) => println!("Created playlist '{}' with ID {}", name, playlist_id),
            Err(e) => println!("Playlist '{}' already exists: {}", name, e),
        }
    }

    pub fn add_media_to_playlist(&mut self, media_id: i64, playlist_id: i64) {

        match self.database.add_media_to_playlist(media_id, playlist_id) {
            Ok(_) => println!("Media ID {} added to Playlist ID {}", media_id, playlist_id),
            Err(e) => println!("Error adding Media ID {} to Playlist ID {}: {}", media_id, playlist_id, e),
        }
    }

    pub fn get_media_from_playlist(&mut self, playlist_id: i64) -> Vec<i64> {

        match self.database.get_media_from_playlist(playlist_id) {
            Ok(media_list) => media_list,
            Err(e) => {
                println!("Error retrieving media from Playlist ID {}: {}", playlist_id, e);
                Vec::new()
            }
        }
    }

    pub fn get_playlist_id(&mut self, name: &str) -> i64 {
        
        match self.database.get_playlist_id(name) {
            Ok(playlist_id) => playlist_id,
            Err(e) => {
                println!("Error retrieving playlist ID for '{}': {}", name, e);
                -1
            }
        }
    }



    // =========== TAGS FUNCTIONS ===========


    // TODO; retuen result (tag id )
    pub fn add_tag(&mut self, tag_name: &str) {

        match self.database.get_or_create_tag(tag_name) {
            Ok(tag_id) => println!("Tag '{}' has ID {}", tag_name, tag_id),
            Err(e) => println!("Error adding tag '{}': {}", tag_name, e),
        }
    }

    pub fn add_tag_to_media(&mut self, media_id: i64, tag_id: i64) {

        match self.database.add_tag_to_media(media_id, tag_id) {
            Ok(_) => println!("Tag ID {} added to Media ID {}", tag_id, media_id),
            Err(e) => println!("Error adding Tag ID {} to Media ID {}: {}", tag_id, media_id, e),
        }
    }

    pub fn get_tag_id(&mut self, tag_name: &str) -> i64 {
        
        match self.database.get_tag_id(tag_name) {
            Ok(tag_id) => tag_id,
            Err(e) => {
                println!("Error retrieving tag ID for '{}': {}", tag_name, e);
                -1
            }
        }
    }



    //=========== SOURCES and SCAN FUNCTIONS ===========


    pub fn add_source(&mut self, path: PathBuf, media_type: MediaType) {
        match media_type {
            MediaType::Audio => self.scan_lib.libraries.add_audio_source(path),
            MediaType::Video => self.scan_lib.libraries.add_video_source(path),
            MediaType::Image => self.scan_lib.libraries.add_image_source(path),
        }

    }

    pub fn get_media_from_path(&mut self, path: PathBuf) -> Vec<MediaInfo> {
        let mut result = Vec::new();

        for (_id, item) in self.items.iter() {
            if item.get_path().starts_with(path.to_str().unwrap()) {
                result.push(item.info());
            }
        }

        result
    }

    pub fn get_all_media(&self) -> Vec<MediaInfo> {
        let mut result = Vec::new();

        for (_id, item) in self.items.iter() {
            result.push(item.info());
        }

        result
    }

    pub fn get_media_by_type(&self, media_type: MediaType) -> Vec<MediaInfo> {
        let mut result = Vec::new();

        for (_id, item) in self.items.iter() {
            if item.media_type() == media_type {
                result.push(item.info());
            }
        }

        result
    }

    pub fn get_media_from_tag(&mut self, tag_id: &str) -> Vec<i64> {
        let media_list = self.database.get_media_by_tag(tag_id).unwrap();
        media_list
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
            //println!("Stopping media ID {id}: {}", item.info());
            item.stop();
        } else {
            println!("Error: media with ID {id} not found.");
        }
    }


    pub fn info_id(&self, id: i64) -> Option<MediaInfo> {
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


