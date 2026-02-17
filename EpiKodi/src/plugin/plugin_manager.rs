/*
src/plugin/plugin_manager.rs - Version Debug
*/

use libloading::{Library, Symbol};
use std::fs;
use std::ffi::{CStr, CString};
use std::path::PathBuf;
use crate::constants::{PLUGIN_DIR, PLUGIN_EXT};

// Import des types de fonctions définis dans functions.rs
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
        println!("🔌 [PLUGIN] Démarrage du chargement des plugins...");
        
        // 1. Vérifier où on est
        if let Ok(cwd) = std::env::current_dir() {
            println!("📂 [PLUGIN] Dossier de travail actuel : {:?}", cwd);
        }

        // 2. Vérifier le dossier plugins
        let plugin_path = PathBuf::from(PLUGIN_DIR);
        if !plugin_path.exists() {
            println!("❌ [PLUGIN] Le dossier '{}' n'existe pas !", PLUGIN_DIR);
            return;
        }

        // 3. Lister les fichiers
        if let Ok(paths) = fs::read_dir(plugin_path) {
            for entry in paths.flatten() {
                let path = entry.path();
                
                // Vérif extension (dll, so, dylib)
                let is_plugin = path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| PLUGIN_EXT.contains(&ext.to_ascii_lowercase().as_str()))
                    .unwrap_or(false);

                if is_plugin {
                    println!("🔎 [PLUGIN] Tentative de chargement : {:?}", path);

                    // 4. Charger la DLL
                    // UNSAFE : libloading charge du code natif
                    let lib = match unsafe { Library::new(&path) } {
                        Ok(l) => {
                            println!("✅ [PLUGIN] DLL chargée en mémoire !");
                            l
                        },
                        Err(e) => {
                            // C'est souvent ici que ça casse (architecture 32/64 bits incorrecte, ou dépendance manquante)
                            println!("❌ [PLUGIN] ÉCHEC chargement DLL : {}", e);
                            continue;
                        }
                    };

                    // 5. Vérifier si c'est le bon type de plugin
                    unsafe {
                        // On cherche la fonction "plugin_type"
                        let get_plugin_type: Result<Symbol<PluginTypeFunc>, _> = lib.get(b"plugin_type\0");
                        
                        match get_plugin_type {
                            Ok(func) => {
                                let type_ptr = func();
                                let result = CStr::from_ptr(type_ptr).to_str().unwrap_or("Error");
                                
                                println!("ℹ️ [PLUGIN] Type détecté : '{}'", result);

                                if result == "metadata" {
                                    self.metadata_libs.push(lib);
                                    println!("🎉 [PLUGIN] Plugin METADATA ajouté avec succès.");
                                } else {
                                    println!("⚠️ [PLUGIN] Ignoré (ce n'est pas un plugin de métadonnées).");
                                }
                            },
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
                // On cherche la fonction "metadata"
                let get_metadata_func: Result<Symbol<GetArtistMetadataFunc>, _> = lib.get(b"metadata\0");

                match get_metadata_func {
                    Ok(func) => {
                        println!("🚀 [PLUGIN] Appel de la fonction 'metadata' dans la DLL...");
                        let metadata_ptr = func(c_artist.as_ptr());
                        let result = CStr::from_ptr(metadata_ptr).to_str().unwrap_or("Error UTF8");
                        
                        // Si le plugin répond "artist not found", on continue la boucle
                        if result == "artist not found" {
                            println!("⚠️ [PLUGIN] Artiste non trouvé dans ce plugin.");
                            continue;
                        }
                        
                        return result.to_string();
                    },
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