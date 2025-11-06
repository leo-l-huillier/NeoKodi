


pub trait Media {
    fn play(&self) {
        println!("Playing media");
    }
    fn pause(&self) {
        println!("Pausing media");
    }
    fn stop(&self) {
        println!("Stopping media");
    }
    fn info(&self) -> String;
}
