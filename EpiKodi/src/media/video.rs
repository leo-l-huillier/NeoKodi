use super::data::Media;

pub struct Video {
    pub path: String,
    pub name: String,
    pub duration: f32,
}

/*

impl Media for Video {
    fn play(&self) {
        println!("Playing video: {}", self.name);
    }
    fn pause(&self) {
        println!("Pausing video: {}", self.name);
    }
    fn stop(&self) {
        println!("Stopping video: {}", self.name);
    }
    fn info(&self) -> String {
        format!("Video Name: {}, Path: {}, Duration: {}", self.name, self.path, self.duration)
    }
}


// ------------------ TESTS ------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_info_format() {
        let video = Video {
            path: "/videos/lofi_beats.mp4".to_string(),
            name: "Lo-Fi Beats".to_string(),
            duration: 120.0,
        };
        let info = video.info();
        assert!(info.contains("Lo-Fi Beats"));
        assert!(info.contains("/videos/lofi_beats.mp4"));
        assert!(info.contains("120"));
    }
}
*/