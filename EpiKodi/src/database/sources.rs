
/*
This file manages media sources for the library,
loading and saving source configurations to a JSON file.
*/

use std::fs;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

use crate::constants::constants::SOURCE_FILE;

#[derive(Serialize, Deserialize, Debug)]
pub struct MediaSource {
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LibraryConfig {
    pub sources: Vec<MediaSource>,
    pub music_sources: Vec<MediaSource>,
    pub video_sources: Vec<MediaSource>,
    pub image_sources: Vec<MediaSource>,
}

impl LibraryConfig {
    pub fn load(path: &str) -> Self {
        if let Ok(data) = fs::read_to_string(path) {
           // println!("Fichier chargé : {}", data);
            if let Ok(config) = serde_json::from_str(&data) {
                // println!("Configuration parsée : {:?}", config);
                return config;
            }
        }

        // Si le fichier n'existe pas ou est vide
        Self { sources: Vec::new(), music_sources: Vec::new(), video_sources: Vec::new(), image_sources: Vec::new() }
    }

    pub fn save(&self, path: &str) {
        let json = serde_json::to_string_pretty(&self).unwrap();
        fs::write(path, json).unwrap();
    }


    pub fn add_source(&mut self, folder: PathBuf) {
        if !self.sources.iter().any(|s| s.path == folder) {
            self.sources.push(MediaSource { path: folder });
        }
        self.save(SOURCE_FILE);
    }
    pub fn add_audio_source(&mut self, folder: PathBuf) {
        if !self.music_sources.iter().any(|s| s.path == folder) {
            self.music_sources.push(MediaSource { path: folder });
        }
        self.save(SOURCE_FILE);
    }
    pub fn add_video_source(&mut self, folder: PathBuf) {
        if !self.video_sources.iter().any(|s| s.path == folder) {
            self.video_sources.push(MediaSource { path: folder });
        }
        self.save(SOURCE_FILE);
    }
    pub fn add_image_source(&mut self, folder: PathBuf) {
        if !self.image_sources.iter().any(|s| s.path == folder) {
            self.image_sources.push(MediaSource { path: folder });
        }
        self.save(SOURCE_FILE);
    }

    
}

