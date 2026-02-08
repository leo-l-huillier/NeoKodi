// This defines the trait that plugins must implement
pub trait Greeter {
    fn greet(&self, name: &str) -> String;
}

pub trait Plugin {
    fn name(&self) -> String;
    fn version(&self) -> String;
    fn plugin_type(&self) -> String; // "metadata",

    //get artist metadata by name
    fn metadata(&self, name: &str) -> String;
}