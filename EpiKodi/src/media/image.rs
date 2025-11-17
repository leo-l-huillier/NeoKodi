use super::data::Media;

use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer::parse;

pub struct Image {
    pub path: String,
    pub name: String,
    pipeline: Option<gst::Pipeline>,
}

impl Image {
    pub fn new(path: &str, name: &str) -> Self {
        Self {
            path: path.to_string(),
            name: name.to_string(),
            pipeline: None,
        }
    }
}

impl Media for Image {
    fn init(&mut self) {
        gst::init().expect("Failed to initialize GStreamer");

        // Use d3dvideosink (Windows) or xvimagesink (Linux) to force separate video window from playbin
        let sink = if cfg!(windows) {
            "d3dvideosink"
        } else {
            "xvimagesink"
        };

        let pipeline_description = format!(
            "filesrc location=\"{}\" ! decodebin name=dec \
             dec. ! queue ! imagefreeze ! videoconvert ! {}",
            self.path, sink
        );

        let pipeline = parse::launch(&pipeline_description)
            .expect("Failed to create pipeline")
            .downcast::<gst::Pipeline>()
            .expect("Failed to downcast to Pipeline");

        self.pipeline = Some(pipeline);
    }

    fn play(&mut self) {
        if let Some(pipeline) = &self.pipeline {
            pipeline
                .set_state(gst::State::Playing)
                .expect("Unable to set the pipeline to the Playing state");
        }
    }

    fn pause(&self) {
        if let Some(pipeline) = &self.pipeline {
            pipeline
                .set_state(gst::State::Paused)
                .expect("Unable to set the pipeline to the Paused state");
        }
    }

    fn stop(&self) {
        if let Some(pipeline) = &self.pipeline {
            pipeline
                .set_state(gst::State::Null)
                .expect("Unable to set the pipeline to the Null state");
        }
    }

    fn info(&self) -> String {
        format!("🖼️ Image: {} ({})", self.name, self.path)
    }
}