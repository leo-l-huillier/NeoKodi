use super::data::Media;
use super::data::MediaType;
use super::data::MediaInfo;


struct Metadata {
    title: String,
    year: String,
    genre: String,
    duration: f32,
    director: String,
    producer: String,
    writer: String,
}

pub struct Video {
    pub path: String,
    pub name: String,
    pub duration: f32,
    media_type: MediaType,
    metadata: Metadata,
}

impl Video {
    pub fn new(path: &str, name: &str) -> Self {
        Self {
            path: path.to_string(),
            name: name.to_string(),
            duration: 0.0,
            media_type: MediaType::Video,
            metadata: Metadata{
                title: "Unknown Title".to_string(),
                year: "Unknown Year".to_string(),
                genre: "Unknown Genre".to_string(),
                duration: 0.0,
                director: "none".to_string(),
                producer: "none".to_string(),
                writer: "none".to_string(),
            }
        }
    }
}

impl Media for Video {
    fn init(&mut self) {
    }

    fn play(&mut self) {
        println!("Simulate Playing Video: {}", self.name);
    }

    fn pause(&self) { }
    fn stop(&self) { }
    fn resume(&self) { }

    fn info(&self) -> MediaInfo {
        MediaInfo {
            id: 0,
            path: self.path.clone(),
            title: Some(self.name.clone()),
            duration: Some(self.metadata.duration),
            media_type: MediaType::Video,
        }
    }

    fn media_type(&self) -> MediaType {
        MediaType::Video
    }
    
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_path(&self) -> String {
        self.path.clone()
    }
}