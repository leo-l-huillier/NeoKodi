use super::data::Media;
use super::data::MediaType;


use rodio::{Decoder, OutputStreamBuilder, Sink};
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;
use std::thread::sleep;

pub struct Audio {
    pub path: String,
    pub name: String,

    stream: Option<rodio::OutputStream>,
    sink: Option<Sink>,

    media_type: MediaType,
}

impl Audio {
    pub fn new(path: &str, name: &str) -> Self {
        Self {
            path: path.to_string(),
            name: name.to_string(),
            sink: None,
            stream: None,
            media_type: MediaType::Audio,
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
    }

    fn pause(&self) {
        if let Some(sink) = &self.sink {
            sink.pause();
        }
    }

    fn resume(&self) {
        if let Some(sink) = &self.sink {
            sink.play();
        }
    }

    fn stop(&self) {
        if let Some(sink) = &self.sink {
            sink.stop();
        }
    }

    fn info(&self) -> String {
        format!("🎧 Audio: {}, path: {}", self.name, self.path)
    }

    fn media_type(&self) -> MediaType {
        MediaType::Audio
    }
}