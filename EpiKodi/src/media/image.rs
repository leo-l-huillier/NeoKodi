use super::data::Media;
use super::data::MediaInfo;
use super::data::MediaType;

use crate::constants::LOG_FILE;
use crate::logger::logger::Logger;

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
            artist: None,
            duration: None,
            media_type: MediaType::Image,
            last_position: 0.0,
            tags: Vec::new(),
        }
    }

    fn media_type(&self) -> MediaType {
        MediaType::Image
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

    #[test]
    fn test_image_creation_and_info() {
        let img = Image::new(99, "/photos/mars_colony.jpg", "Colonie Martienne");
        let info = img.info();

        assert_eq!(info.id, 99);
        assert_eq!(info.path, "/photos/mars_colony.jpg");
        assert_eq!(info.title, Some("Colonie Martienne".to_string()));
        assert_eq!(info.duration, None); // Une image n'a pas de durée
        assert_eq!(info.last_position, 0.0);
        assert_eq!(info.media_type, MediaType::Image);
    }
}
