
/*
in this file we handle video playback
TODO: get metadata for director, producer, writer -> for now set to "none"
TODO: metadata got from lofty crate but not all video formats are supported (only mp4)
*/

use super::data::Media;
use super::data::MediaType;
use super::data::MediaInfo;

use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer::parse;

use lofty::prelude::*;
use lofty::{read_from_path};


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

    pipeline: Option<gst::Pipeline>,

    media_type: MediaType,
    metadata: Metadata,
}

impl Video {
    pub fn new(path: &str, name: &str) -> Self {
        /*
        let tagged_file = read_from_path(path)
            .expect("Failed to read tags from file");
        let tag = match tagged_file.primary_tag() {
            Some(primary_tag) => primary_tag,
            None => tagged_file.first_tag().expect("ERROR: No tags found!"),
        };

        // let props = tagged_file.properties();
        */

        //print the content of the tag

        Self {
            path: path.to_string(),
            name: name.to_string(),
            duration: 0.0,
            pipeline: None,
            media_type: MediaType::Video,
            metadata: Metadata{
                /*title: tag.title().as_deref().unwrap_or("None").to_string(),
                year: tag.year().map_or("None".to_string(), |y| y.to_string()),
                genre: tag.genre().as_deref().unwrap_or("None").to_string(),
                duration: props.duration().as_secs_f32(),*/
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
        gst::init().expect("Failed to initialize GStreamer");

        let pipeline_description = format!(
            "filesrc location=\"{}\" ! decodebin name=dec \
             dec. ! queue ! audioconvert ! autoaudiosink \
             dec. ! queue ! videoconvert ! autovideosink",
            self.path
        );

        let pipeline = parse::launch(&pipeline_description)
            .expect("Failed to create pipeline")
            .downcast::<gst::Pipeline>()
            .expect("Failed to downcast to Pipeline");

        self.pipeline = Some(pipeline);
    }

    fn play(&mut self) {
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Playing).expect("Unable to set the pipeline to the Playing state");
        }
    }

    fn pause(&self) {
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Paused).expect("Unable to set the pipeline to the Paused state");
        }
    }

    fn stop(&self) {
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Null).expect("Unable to set the pipeline to the Null state");
        }
    }

    fn info(&self) -> MediaInfo {
        
        // print metadata info
        println!("{}", format!(
            "Title: {}\nYear: {}\nGenre: {}\nDuration: {} seconds\nDirector: {}\nProducer: {}\nWriter: {}",
            self.metadata.title,
            self.metadata.year,
            self.metadata.genre,
            self.metadata.duration,
            self.metadata.director,
            self.metadata.producer,
            self.metadata.writer
        ));

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
