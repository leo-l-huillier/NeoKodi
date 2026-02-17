/*
in this file we handle audio playback
*/

use super::data::{Media, MediaType, MediaInfo};

use lofty::prelude::*; 
use lofty::read_from_path;

// use std::fs::File; // Plus besoin
// use std::io::BufReader; // Plus besoin
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::logger::logger::Logger;
use crate::constants::LOG_FILE;

// --- STRUCTURE METADATA ---
pub struct Metadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: f32,
}

impl Metadata {
    pub fn from_path(path: &str) -> Self {
        match read_from_path(path) {
            Ok(tagged_file) => {
                let properties = tagged_file.properties();
                let tag = tagged_file.primary_tag();

                Metadata {
                    duration: properties.duration().as_secs_f32(),
                    title: tag.and_then(|t| t.title().map(|s| s.to_string())),
                    artist: tag.and_then(|t| t.artist().map(|s| s.to_string())),
                    album: tag.and_then(|t| t.album().map(|s| s.to_string())),
                }
            },
            Err(_) => Metadata {
                duration: 0.0,
                title: None,
                artist: None,
                album: None,
            }
        }
    }
}

// --- QUEUE (Inchangé) ---
pub struct Queue {
    audio_queue: Vec<i64>,
    original_queue: Vec<i64>,
    repeat: bool,
}

impl Queue {
    pub fn new() -> Self {
        Queue {
            audio_queue: Vec::new(),
            original_queue: Vec::new(),
            repeat: false,
        }
    }

    pub fn add_to_queue(&mut self, media: i64) {
        let logger = Logger::new(LOG_FILE);
        self.audio_queue.push(media);
        self.original_queue.push(media);
        logger.debug(&format!("Added to queue: {}", media));
    }

    pub fn get_current(&self) -> Option<i64> {
        self.audio_queue.first().cloned()
    }

    pub fn next(&mut self) -> Option<i64> {
        let logger = Logger::new(LOG_FILE);
        
        if !self.audio_queue.is_empty() {
            let next_media = self.audio_queue.remove(0);
            logger.info(&format!("Next media in queue: {}", next_media));
            Some(next_media)
        } else {
            if self.repeat && !self.original_queue.is_empty() {
                self.audio_queue = self.original_queue.clone();
                logger.info("Queue is empty, repeating original queue");
                return self.get_current();
            }
            logger.info("Trying to get next media but queue is empty");
            None
        }
    }

    pub fn toggle_shuffle(&mut self) {
        let logger = Logger::new(LOG_FILE);
        if self.audio_queue.is_empty() { return; }
        let mut rng = thread_rng();
        self.audio_queue.shuffle(&mut rng);
        logger.info("Queue shuffled");
    }

    pub fn toggle_repeat(&mut self) {
        self.repeat = !self.repeat;
    }

    pub fn clear_queue(&mut self) {
        self.audio_queue.clear();
        self.original_queue.clear();
    }
}

// --- STRUCTURE AUDIO (NETTOYÉE) ---
pub struct Audio {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub metadata: Metadata, 
    pub last_position: f32,
    // Plus de champs Sink/Stream !
}

impl Audio {
    pub fn new(id: i64, path: &str, name: &str, last_pos: f32) -> Self {
        let metadata = Metadata::from_path(path);
        Self {
            id,
            path: path.to_string(),
            name: name.to_string(),
            metadata,
            last_position: last_pos,
        }
    }
}

// 👇 C'EST ICI LA MAGIE : ON NE FAIT RIEN EN RUST
// On laisse l'interface HTML gérer le vrai son.
impl Media for Audio {
    fn init(&mut self) {
    }

    fn play(&mut self) {
        println!("▶️ Audio sélectionné (Played by Frontend): {}", self.name);
    }

    fn pause(&self) {
    }

    fn resume(&self) {
    }

    fn stop(&self) {
    }
    
    fn info(&self) -> MediaInfo {
        MediaInfo {
            id: self.id,
            path: self.path.clone(),
            title: Some(self.name.clone()),
            artist: self.metadata.artist.clone(),
            duration: Some(self.metadata.duration),
            media_type: MediaType::Audio,
            last_position: self.last_position,
            tags: Vec::new(),
        }
    }
    
    fn media_type(&self) -> MediaType {
        MediaType::Audio
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_path(&self) -> String {
        self.path.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn sample_path() -> Option<String> {
        let p = Path::new("tests/data/sample.mp3");
        if p.exists() { Some(p.to_string_lossy().to_string()) } else { None }
    }

    #[test]
    fn new_reads_metadata() {
        if let Some(path) = sample_path() {
            let audio = Audio::new(1, &path, "Sample", 0.0);
            assert_eq!(audio.media_type(), MediaType::Audio);
            assert_eq!(audio.id, 1);
        }
    }

    #[test]
    fn info_returns_media_info() {
        if let Some(path) = sample_path() {
            let audio = Audio::new(1, &path, "Sample", 10.5);
            let info = audio.info();
            assert_eq!(info.path, path);
            assert_eq!(info.last_position, 10.5);
            assert!(info.tags.is_empty());
        }
    }
}