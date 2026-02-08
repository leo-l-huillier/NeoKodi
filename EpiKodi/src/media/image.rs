use super::data::Media;
use super::data::MediaType;
use super::data::MediaInfo;

use crate::logger::logger::Logger;
use crate::constants::LOG_FILE;

pub struct Image {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub media_type: MediaType,
}

impl Image {
    pub fn new(id: i64, path: &str, name: &str) -> Self {
        Self {
            id,
            path: path.to_string(),
            name: name.to_string(),
            media_type: MediaType::Image,
        }
    }
}

impl Media for Image {
    fn init(&mut self) {}

    fn play(&mut self) {

        let logger = Logger::new(LOG_FILE);

        logger.debug(&format!("Playing image: {}", self.name));
    }

    fn pause(&self) {}
    fn resume(&self) {}
    fn stop(&self) {}

    fn info(&self) -> MediaInfo {
        MediaInfo {
            id: self.id,
            path: self.path.clone(),
            title: Some(self.name.clone()),
            duration: None,
            media_type: MediaType::Image,
        }
    }

    fn media_type(&self) -> MediaType { MediaType::Image }
    fn get_name(&self) -> String { self.name.clone() }
    fn get_path(&self) -> String { self.path.clone() }
}