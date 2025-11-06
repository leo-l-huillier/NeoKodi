use super::data::Media;

pub struct Audio {
    pub path: String,
    pub name: String,
}

impl Media for Audio {
    fn play(&self) {
        println!("🎵 Playing audio: {}", self.name);
    }

    fn pause(&self) {
        println!("⏸️ Paused audio: {} ", self.name);
    }

    fn info(&self) -> String {
        format!("🎧 Audio: {}, path: {}", self.name, self.path)
    }
}