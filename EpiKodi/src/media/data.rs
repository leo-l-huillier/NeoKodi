

pub enum MediaType {
    Audio,
    Video,
    Image,
}

pub trait Media {
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
}
