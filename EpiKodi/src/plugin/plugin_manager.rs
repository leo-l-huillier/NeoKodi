use crate::constants::{PLUGIN_DIR, PLUGIN_EXT};
use libloading::{Library, Symbol};
use std::ffi::{CStr, CString};
use std::fs;
use std::path::PathBuf;

// Assure-toi que ces types sont bien définis dans functions.rs
use super::functions::GetArtistMetadataFunc;
use super::functions::GetFilmMetadataFunc;
use super::functions::PluginTypeFunc;

use crate::constants::LOG_FILE;
use crate::logger::logger::Logger;

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
        // 1. On initialise le logger ICI pour pouvoir l'utiliser
        let logger = Logger::new(LOG_FILE);

        println!("🔌 [PLUGIN] Démarrage du chargement des plugins...");

        if let Ok(cwd) = std::env::current_dir() {
            println!("📂 [PLUGIN] Dossier de travail actuel : {:?}", cwd);
        }

        let plugin_path = PathBuf::from(PLUGIN_DIR);
        if !plugin_path.exists() {
            println!("❌ [PLUGIN] Le dossier '{}' n'existe pas !", PLUGIN_DIR);
            return;
        }

        if let Ok(paths) = fs::read_dir(plugin_path) {
            for entry in paths.flatten() {
                let path = entry.path();

                let is_plugin = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| PLUGIN_EXT.contains(&ext.to_ascii_lowercase().as_str()))
                    .unwrap_or(false);

                if is_plugin {
                    println!("🔎 [PLUGIN] Tentative de chargement : {:?}", path);

                    // CHARGEMENT DLL
                    let lib = match unsafe { Library::new(&path) } {
                        Ok(l) => {
                            println!("✅ [PLUGIN] DLL chargée en mémoire !");
                            l
                        }
                        Err(e) => {
                            logger.error(&format!("✗ ÉCHEC chargement DLL : {}", e));
                            continue;
                        }
                    };

                    // VÉRIFICATION DU TYPE
                    unsafe {
                        let get_plugin_type: Result<Symbol<PluginTypeFunc>, _> =
                            lib.get(b"plugin_type\0");

                        match get_plugin_type {
                            Ok(func) => {
                                let type_ptr = func();
                                let result = CStr::from_ptr(type_ptr).to_str().unwrap_or("Error");

                                println!("ℹ️ [PLUGIN] Type détecté : '{}'", result);

                                match result {
                                    "metadata" => {
                                        self.metadata_libs.push(lib);
                                        logger.info("Loaded metadata plugin.");
                                    }
                                    "film_metadata" => {
                                        self.film_metadata_libs.push(lib);
                                        logger.info("Loaded film metadata plugin.");
                                    }
                                    _ => {
                                        logger.error(&format!("✗ Unknown plugin type: {}", result));
                                    }
                                }
                            }
                            Err(e) => {
                                logger.error(&format!(
                                    "✗ Failed to load 'plugin_type' function: {} ",
                                    e
                                ));
                                continue;
                            }
                        };
                    }
                }
            }
        }
    }

    // Changement ici : on renvoie String, pas &str
    pub fn get_metadata(&mut self, artist: &str) -> String {
        let logger = Logger::new(LOG_FILE);
        let c_artist = CString::new(artist).unwrap();

        for lib in &self.metadata_libs {
            unsafe {
                let get_metadata_func: Result<Symbol<GetArtistMetadataFunc>, _> =
                    lib.get(b"metadata\0");

                match get_metadata_func {
                    Ok(func) => {
                        // On appelle la fonction de la DLL
                        let metadata_ptr = func(c_artist.as_ptr());
                        let result = CStr::from_ptr(metadata_ptr)
                            .to_str()
                            .unwrap_or("Error UTF8");

                        if result == "artist not found" {
                            continue; // On essaye le plugin suivant
                        }

                        // On a trouvé ! On renvoie une String propre
                        return result.to_string();
                    }
                    Err(e) => {
                        logger.error(&format!("✗ Failed to load 'metadata' function: {}", e));
                        continue;
                    }
                };
            }
        }
        return "artist not found".to_string();
    }

    // Changement ici aussi : on renvoie String
    // Dans src/plugin/plugin_manager.rs

    pub fn get_film_metadata(&mut self, film: &str) -> String {
        let logger = Logger::new(LOG_FILE);
        let c_film = CString::new(film).unwrap();

        // 👇 DEBUG : Combien de plugins de films sont chargés ?
        println!(
            "🎞️ [MANAGER] Nombre de plugins 'film' chargés : {}",
            self.film_metadata_libs.len()
        );

        for lib in &self.film_metadata_libs {
            unsafe {
                let get_film_metadata: Result<Symbol<GetFilmMetadataFunc>, _> =
                    lib.get(b"metadata\0");

                match get_film_metadata {
                    Ok(func) => {
                        println!("🚀 [MANAGER] Appel de la DLL TMDB pour '{}'...", film);
                        let metadata_ptr = func(c_film.as_ptr());
                        let result = CStr::from_ptr(metadata_ptr)
                            .to_str()
                            .unwrap_or("Error UTF8");

                        println!("📥 [MANAGER] Réponse de la DLL : {}", result);

                        if result == "film not found" {
                            continue;
                        }
                        return result.to_string();
                    }
                    Err(e) => {
                        logger.error(&format!(
                            "✗ Failed to load 'metadata' (film) function: {}",
                            e
                        ));
                        continue;
                    }
                }
            }
        }
        println!("❌ [MANAGER] Aucun plugin n'a trouvé le film (ou aucun plugin chargé).");
        return "film not found".to_string();
    }
}
