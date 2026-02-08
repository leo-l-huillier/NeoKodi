
/*
This file contains constant values used throughout the application
TODO: check the if there is a correct way to do that and good practices
*/

pub const SOURCE_FILE: &str = "db/sources.json";
pub const MEDIA_DB_FILE: &str = "db/library.db";

pub const DEBUG: bool = true;
pub const LOG_FILE: &str = "epikodi.log";
pub const LOG_FILE_MEDIA_ITEMS: &str = "media_items.log";
pub const LOG_IN_CONSOLE: bool = false;

pub const AUDIO_EXTS: [&str; 5] = ["mp3", "wav", "flac", "ogg", "mp4"];
pub const VIDEO_EXTS: [&str; 4] = ["mp4", "mkv", "avi", "mov"];
pub const IMAGE_EXTS: [&str; 4] = ["jpg", "png", "bmp", "gif"];

pub const PLUGIN_DIR: &str = "./plugins/";
pub const PLUGIN_EXT: &str = if cfg!(target_os = "windows") {
    "dll"
} else if cfg!(target_os = "macos") {
    "dylib"
} else {
    "so"
};

// Media status constants
pub const NOT_STARTED: i32 = 0;
pub const PLAYING: i32 = 1;
pub const FINISHED: i32 = 2;