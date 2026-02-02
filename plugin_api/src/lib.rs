// This defines the trait that plugins must implement
pub trait Greeter {
    fn greet(&self, name: &str) -> String;
}
