use super::data::Media;
use super::data::MediaType;

use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer::parse;

pub struct Video {
    pub path: String,
    pub name: String,
    pub duration: f32,
    pipeline: Option<gst::Pipeline>,
    media_type: MediaType,
}

impl Video {
    pub fn new(path: &str, name: &str) -> Self {
        Self {
            path: path.to_string(),
            name: name.to_string(),
            duration: 0.0,
            pipeline: None,
            media_type: MediaType::Video,
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

    fn info(&self) -> String {
        format!("Video: {} ({})", self.name, self.path)
    }

    fn media_type(&self) -> MediaType {
        MediaType::Video
    }
}
