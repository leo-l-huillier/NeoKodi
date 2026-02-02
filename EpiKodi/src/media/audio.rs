
/*
in this file we handle audio playback
TODO: 
*/

use super::data::Media;
use super::data::MediaType;
use super::data::MediaInfo;


use rodio::{Decoder, OutputStreamBuilder, Sink};
use std::fs::File;
use std::io::BufReader;


use lofty::prelude::*;
use lofty::{read_from_path};


struct Metadata {
    title: String,
    artist: String,
    album: String,
    year: String,
    genre: String,
    duration: f32,
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
        println!("{}", format!(
            "Audio: {} ({})\nTitle: {}\nArtist: {}\nAlbum: {}\nYear: {}\nGenre: {}\nDuration: {:.2} seconds",
            self.name,
            self.path,
            self.metadata.title,
            self.metadata.artist,
            self.metadata.album,
            self.metadata.year,
            self.metadata.genre,
            self.metadata.duration
        ));

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