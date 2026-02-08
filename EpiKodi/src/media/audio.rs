
/*
in this file we handle audio playback
TODO: 
*/

use super::data::Media;
use super::data::MediaType;
use super::data::MediaInfo;

use crate::logger::logger::Logger;
use crate::constants::LOG_FILE;


use rodio::{Decoder, OutputStreamBuilder, Sink};
use std::fs::File;
use std::io::BufReader;


use lofty::prelude::*;
use lofty::{read_from_path};

use rand::seq::SliceRandom;
//use rand::rng::thread_rng;
use rand::thread_rng;




struct Metadata {
    title: String,
    artist: String,
    album: String,
    year: String,
    genre: String,
    duration: f32,
}

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

    pub fn get_current(&mut self) -> Option<i64> {

        let logger = Logger::new(LOG_FILE);

        if let Some(current) = self.audio_queue.first() {
            logger.info(&format!("Current media in queue: {}", current));
            Some(*current)
        } else {
            logger.info("Trying to get current media but queue is empty");
            None
        }

    }

    pub fn next(&mut self) -> Option<i64> {
       
        let logger = Logger::new(LOG_FILE);
       
        if !self.audio_queue.is_empty() {
            let next_media = self.audio_queue.remove(0);
            logger.info(&format!("Next media in queue: {}", next_media));
            Some(next_media)
        } else {

            if self.repeat {
                self.audio_queue = self.original_queue.clone();
                logger.info("Queue is empty, repeating original queue");
                Some(self.get_current());
            }

            logger.info("Trying to get next media but queue is empty");
            None
        }
    }

    pub fn toggle_shuffle(&mut self) {

        let logger = Logger::new(LOG_FILE);

        if self.audio_queue.is_empty() {
            logger.debug("Trying to shuffle queue but it's empty");
            return;
        }

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

pub struct Audio {
    pub path: String,
    pub name: String,

    media_type: MediaType,
    metadata: Metadata, // make it a json

    stream: Option<rodio::OutputStream>,    
    sink: Option<Sink>,

    
}

impl Audio {
    pub fn new(path: &str, name: &str) -> Self {


        //========= METADATA ========= 
        let tagged_file = read_from_path(path)
            .expect("Failed to read tags from file");
        let tag = match tagged_file.primary_tag() {
            Some(primary_tag) => primary_tag,
            None => tagged_file.first_tag().expect("ERROR: No tags found!"),
        };
        let props = tagged_file.properties();


        Self {
            path: path.to_string(),
            name: name.to_string(),
            sink: None,
            stream: None,
            media_type: MediaType::Audio,
            metadata: Metadata {
                title: tag.title().as_deref().unwrap_or("None").to_string(),
                artist: tag.artist().as_deref().unwrap_or("None").to_string(),
                album: tag.album().as_deref().unwrap_or("None").to_string(),
                year: tag.year().map_or("None".to_string(), |y| y.to_string()),
                genre: tag.genre().as_deref().unwrap_or("None").to_string(),
                duration: props.duration().as_secs_f32(),
            },
        }
    }
}


impl Media for Audio {
    fn init(&mut self) {
        let stream = OutputStreamBuilder::open_default_stream().expect("open_default_stream error");

        let sink = Sink::connect_new(&stream.mixer());

        self.stream = Some(stream);
        self.sink = Some(sink);
    }


    fn play(&mut self) {  
        let sink = self.sink.as_ref().expect("AudioPlayer not initialized");

        let file = File::open(&self.path).expect("open file error");
        let source = Decoder::new(BufReader::new(file)).expect("decoder error");

        sink.append(source);

        sink.play();
        println!("in media play");
    }

    fn pause(&self) {
        println!("in media pause");
        if let Some(sink) = &self.sink {
            sink.pause();
        }
    }

    fn resume(&self) {
        println!("in media resume");
        if let Some(sink) = &self.sink {
            sink.play();
        }
    }

    fn stop(&self) {
        if let Some(sink) = &self.sink {
            sink.stop();
        }
    }
    

    fn info(&self) -> MediaInfo {


        //print metadata info
        /*println!("{}", format!(
            "Audio: {} ({})\nTitle: {}\nArtist: {}\nAlbum: {}\nYear: {}\nGenre: {}\nDuration: {:.2} seconds",
            self.name,
            self.path,
            self.metadata.title,
            self.metadata.artist,
            self.metadata.album,
            self.metadata.year,
            self.metadata.genre,
            self.metadata.duration
        ));*/

        MediaInfo {
            id: 0,
            path: self.path.clone(),
            title: Some(self.name.clone()),
            duration: Some(self.metadata.duration),
            media_type: MediaType::Audio,
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
        let path = sample_path().expect("missing test audio file");
        let audio = Audio::new(&path, "Sample");
        assert_eq!(audio.media_type(), MediaType::Audio);
        assert_eq!(audio.get_path(), path);
        assert_eq!(audio.get_name(), "Sample");
    }

    #[test]
    fn init_and_play_then_stop() {
        let path = sample_path().expect("missing test audio file");
        let mut audio = Audio::new(&path, "Sample");
        audio.init();
        audio.play(); // should not panic
        audio.pause();
        audio.resume();
        audio.stop();
    }

    #[test]
    fn info_returns_media_info() {
        let path = sample_path().expect("missing test audio file");
        let audio = Audio::new(&path, "Sample");
        let info = audio.info();
        assert_eq!(info.path, path);
        assert_eq!(info.media_type, MediaType::Audio);
        assert!(info.duration.unwrap_or(0.0) >= 0.0);
    }
}