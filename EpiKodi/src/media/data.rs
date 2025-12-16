
use std::fmt;

#[derive(Debug, Clone)]
pub enum MediaType {
    Audio,
    Video,
    Image,
}

impl MediaType {
    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "Video" => Some(MediaType::Video),
            "Audio" => Some(MediaType::Audio),
            "Image" => Some(MediaType::Image),
            _ => Some(MediaType::Image),
        }
    }

    pub fn as_db(&self) -> &'static str {
        match self {
            MediaType::Video => "video",
            MediaType::Audio => "audio",
            MediaType::Image => "image",
        }
    }
}


impl fmt::Display for MediaType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MediaType::Audio => write!(f, "Audio"),
            MediaType::Video => write!(f, "Video"),
            MediaType::Image => write!(f, "Image"),
        }
    }
}

//TODO: implement finished event handling
pub trait Media: Send + Sync {
    fn init(&mut self) {
        println!("Media initialized");
    }
    fn play(&mut self) {
        println!("Playing media");
    }
    fn pause(&self) {
        println!("Pausing media");
    }
    fn resume(&self) {
        println!("Resuming media");
    }
    fn stop(&self) {
        println!("Stopping media");
    }
    fn info(&self) -> String;
    fn media_type(&self) -> MediaType;

    //debug
    fn get_name(&self) -> String;
}
