/* src/media/video.rs */

use super::data::Media;
use super::data::MediaType;
use super::data::MediaInfo;

use crate::logger::logger::Logger;
use crate::constants::LOG_FILE;

// On garde une structure simple pour les métadonnées (optionnel pour l'instant)
#[derive(Clone)]
pub struct VideoMetadata {
    pub duration: f32,
    // On pourra remettre realisateur/année plus tard
}

pub struct Video {
    pub id: i64,
    pub last_position: f32,
    pub path: String,
    pub name: String,
    pub metadata: VideoMetadata,
}

impl Video {
    pub fn new(id: i64, path: &str, name: &str, last_position: f32, duration: f32) -> Self {
        Self {
            id,
            last_position,
            path: path.to_string(),
            name: name.to_string(),
            metadata: VideoMetadata {
            duration: duration, // On utilise la durée passée en paramètre
            }
        }
    }
}

impl Media for Video {
    fn init(&mut self) {}

    fn play(&mut self) {
        let logger = Logger::new(LOG_FILE);
        logger.debug(&format!("Playing video: {}", self.name));
    }

    fn pause(&self) {}
    fn resume(&self) {}
    fn stop(&self) {}

    fn info(&self) -> MediaInfo {
        MediaInfo {
            id: self.id,
            path: self.path.clone(),
            title: Some(self.name.clone()),
            duration: Some(self.metadata.duration),
            media_type: MediaType::Video,
            last_position: self.last_position,
        }
    }

    fn media_type(&self) -> MediaType { MediaType::Video }
    fn get_name(&self) -> String { self.name.clone() }
    fn get_path(&self) -> String { self.path.clone() }
}