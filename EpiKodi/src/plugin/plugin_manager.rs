/*


*/


use libloading::{Library, Symbol};
use std::fs;
use std::ffi::{CStr, CString};
use crate::constants::{PLUGIN_DIR, PLUGIN_EXT};


use super::functions::PluginTypeFunc;
use super::functions::GetArtistMetadataFunc;
use super::functions::GetFilmMetadataFunc;

use crate::logger::logger::Logger;
use crate::constants::LOG_FILE;

pub struct PluginManager {
    pub metadata_libs: Vec<Library>,
    pub film_metadata_libs: Vec<Library>,
}

impl PluginManager {
    pub fn new() -> Self {
        PluginManager {
            metadata_libs: Vec::new(),
            film_metadata_libs: Vec::new(),
        }
    }


    pub fn load_plugins(&mut self) {

        let logger = Logger::new(LOG_FILE);
        logger.debug("Loading plugins...");

        if let Ok(paths) = fs::read_dir(PLUGIN_DIR) {
            for entry in paths.flatten() {
                let path = entry.path();

                // Extension check
                let is_plugin = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| PLUGIN_EXT.contains(&ext.to_ascii_lowercase().as_str()))
                    .unwrap_or(false);
                logger.debug(&format!("Checking plugin file: {:?}", path));

                if is_plugin {
                    let lib = match unsafe { Library::new(path) } {
                        Ok(lib) => {
                            logger.debug("✓ Library loaded successfully: {:?}");
                            lib
                        }
                        Err(e) => {
                            logger.error(&format!("✗ Failed to load library: {} \nMake sure you've built the plugin first:", e));
                            continue;
                        }
                    };


                    unsafe {
                        let get_plugin_type: Symbol<PluginTypeFunc> = match lib.get(b"plugin_type\0") {
                            Ok(func) => func,
                            Err(e) => {
                                logger.error(&format!("✗ Failed to load 'plugin_type' function: {} ", e));
                                continue;
                            }
                        };

                        let type_ptr = get_plugin_type();
                        let result = CStr::from_ptr(type_ptr)
                            .to_str()
                            .unwrap_or("Error: Invalid UTF-8");

                        match result {
                             "metadata" => {
                                self.metadata_libs.push(lib);
                                logger.info("Loaded metadata plugin.");
                                }, 
                             "film_metadata" => {
                                self.film_metadata_libs.push(lib);
                                logger.info("Loaded film metadata plugin.");
                             },
                            _ => logger.error(&format!("✗ Unknown plugin type: {}", result)),
                        }
                    }

                }
            }
        }
    }


    pub fn get_metadata(&mut self, artist: &str) -> &str {

        let logger = Logger::new(LOG_FILE);

        let c_artist = CString::new(artist).unwrap();

        for lib in &self.metadata_libs {

            unsafe {
                let get_artist_metadata: Symbol<GetArtistMetadataFunc> = match lib.get(b"metadata\0") {
                    Ok(func) => func,
                    Err(e) => {
                        logger.error(&format!("✗ Failed to load 'get_artist_metadata' function: {}", e));
                        continue;
                    }
                };

                let metadata_ptr = get_artist_metadata(c_artist.as_ptr());
                let result = CStr::from_ptr(metadata_ptr)
                    .to_str()
                    .unwrap_or("Error: Invalid UTF-8");

                    if result == "artist not found" {
                        logger.error(&format!("Artist ({}) not found in this plugin, trying next if available... ", artist));
                        continue;
                    }
                    return result;

            }
        }
        return "artist not found";
    }

    pub fn get_film_metadata(&mut self, film: &str) -> &str {

        let logger = Logger::new(LOG_FILE);

        let c_film = CString::new(film).unwrap();

        for lib in &self.film_metadata_libs {

            unsafe {
                let get_film_metadata: Symbol<GetFilmMetadataFunc> = match lib.get(b"metadata\0") {
                    Ok(func) => func,
                    Err(e) => {
                        logger.error(&format!("✗ Failed to load 'get_film_metadata' function: {}", e));
                        continue;
                    }
                };

                let metadata_ptr = get_film_metadata(c_film.as_ptr());
                let result = CStr::from_ptr(metadata_ptr)
                    .to_str()
                    .unwrap_or("Error: Invalid UTF-8");

                    if result == "film not found" {
                        logger.error(&format!("Film ({}) not found in this plugin, trying next if available... ", film));
                        continue;
                    }
                    return result;

            }
        }
        return "film not found";
    }


}