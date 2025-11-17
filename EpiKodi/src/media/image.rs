use super::data::Media;

pub struct Image {
    path: String,
    name: String,
}
/*
impl Media for Image {
    fn play(&self) {
        println!(" image: {}", self.name);
    }

    fn pause(&self) {
        println!("Paused nothing it's an image");
    }

    fn info(&self) -> String {
        format!("image: {}, path : {}", self.name, self.path)
    }
} */