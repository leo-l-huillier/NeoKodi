/*#![allow(non_snake_case)]

mod constants;
mod threading;
mod media;
mod database;
mod library;
mod scan;
mod gui;
mod config;

use crate::gui::style::GLOBAL_STYLE;
use config::AppConfig;
use threading::media_thread::launch_media_thread;
use threading::command::{Command, Event};
use gui::route::Route;
use crate::media::data::{MediaInfo, MediaType}; 
use std::sync::mpsc;
use std::path::PathBuf; 
use std::time::Duration;
use std::cell::RefCell;
use std::rc::Rc;
use dioxus::prelude::*;
use dioxus::desktop::{Config, WindowBuilder};
use dioxus_router::prelude::*;
use warp::Filter;
use std::thread;
use tokio::sync::broadcast;
use std::sync::OnceLock;

static RELOAD_SIGNAL: OnceLock<broadcast::Sender<()>> = OnceLock::new();

struct Backend {
    tx: mpsc::Sender<Command>,
    rx: RefCell<Option<mpsc::Receiver<Event>>>,
}

fn main() {
    unsafe {
        std::env::set_var("RUST_LOG", "warp=info");
        std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", "--disable-web-security --allow-file-access-from-files --allow-running-insecure-content --autoplay-policy=no-user-gesture-required");
    }

    let (reload_tx, _) = broadcast::channel::<()>(16); // Augmenté à 16 par sécurité
    let _ = RELOAD_SIGNAL.set(reload_tx.clone());

    let reload_tx_server = reload_tx.clone(); 

    // SALBATAR: POURQUOI TA BESOIN D4UN SERVER?????????????????????
    // --- LE THREAD SERVEUR ROBUSTE ---
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            loop {
                let app_config = AppConfig::load();
                let server_root = app_config.media_path.clone();
                
                println!("\n========================================");
                println!("🌍 SERVEUR : Démarrage sur {}", server_root);
                println!("========================================\n");

                let mut rx = reload_tx_server.subscribe();

                let media_route = warp::path("media").and(warp::fs::dir(server_root));
                let cors = warp::cors().allow_any_origin().allow_methods(vec!["GET", "HEAD"]);
                let routes = media_route.with(cors); 

                // On lance le serveur
                let (_addr, server) = warp::serve(routes)
                    .bind_with_graceful_shutdown(([127, 0, 0, 1], 3030), async move {
                        // On attend le signal
                        let _ = rx.recv().await;
                        println!("\n🛑 SERVEUR : Signal d'arrêt reçu ! Fermeture en cours...\n");
                    });

                // On attend que le serveur s'arrête proprement
                server.await;
                
                println!("⏳ SERVEUR : Port 3030 en cours de libération...");
                // 👇 ON AUGMENTE LE DÉLAI DE SÉCURITÉ (1 seconde)
                // Windows met parfois du temps à libérer le port TCP.
                tokio::time::sleep(Duration::from_millis(1000)).await;
                println!("✅ SERVEUR : Prêt à redémarrer.");
            }
        });
    });

    let window = WindowBuilder::new().with_title("NeoKodi").with_resizable(true).with_maximized(true);
    let config_dioxus = Config::new().with_window(window).with_custom_head(format!("<style>{}</style>", GLOBAL_STYLE)).with_disable_context_menu(false);

    LaunchBuilder::desktop().with_cfg(config_dioxus).launch(App);
}

fn App() -> Element {
    let backend_channels = use_hook(|| {

        let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();
        let (evt_tx, evt_rx) = mpsc::channel::<Event>();
        
        launch_media_thread(cmd_rx, evt_tx);

        let config = AppConfig::load();
        let root_path = PathBuf::from(config.media_path);
        println!("🚀 UI : Démarrage sur {:?}", root_path);
        
        // SALBATAR: POURQUOI T4ADD DES SOURCES ICI CA A AUCUNS INTERET, ET EN PLUS C'EST SENC2 ETRE FAIS DANS LE BACK
        // SALBATAR: ET EN PLUS APPELER LE TRUC 3 FOIS C4EST TELLEMENT MOCHE ET CA MARCHE QUE PARCE QUE J4AI BIEN FAIS MON TRAVAIL
        // 👇 CHANGEMENT ICI :
        // On n'utilise PAS ChangeLibraryPath au démarrage (car ça vide la DB !).
        // On utilise AddSource pour s'assurer que le dossier est bien surveillé.
        let _ = cmd_tx.send(Command::AddSource(root_path.clone(), MediaType::Video));
        let _ = cmd_tx.send(Command::AddSource(root_path.clone(), MediaType::Audio));
        let _ = cmd_tx.send(Command::AddSource(root_path, MediaType::Image));

        // 👇 TRES IMPORTANT :
        // On demande tout de suite au backend : "Envoie-moi ce que tu as déjà en base de données !"
        // Comme ça, l'affichage est instantané, même si le scan n'est pas fini.
        let _ = cmd_tx.send(Command::GetAllMedia());

        Rc::new(Backend { tx: cmd_tx, rx: RefCell::new(Some(evt_rx)) })
    });

    use_context_provider(|| backend_channels.tx.clone());
    use_context_provider(|| Signal::new(Option::<(String, String)>::None));
    let mut media_list = use_context_provider(|| Signal::new(Vec::<MediaInfo>::new()));
    
    // Récupération sécurisée du signal
    if let Some(tx) = RELOAD_SIGNAL.get() {
        use_context_provider(|| tx.clone());
    } else {
        println!("⚠️ Signal de reload non initialisé (Normal au premier rendu)");
    }
    
    let current_config = AppConfig::load();
    use_context_provider(|| Signal::new(current_config.media_path));

    use_coroutine(|_: UnboundedReceiver<()>| {
        let backend = backend_channels.clone();
        async move {
            let mut rx_opt = backend.rx.borrow_mut();
            if let Some(rx) = rx_opt.take() {
                drop(rx_opt);
                loop {
                    if let Ok(msg) = rx.try_recv() {
                        if let Event::MediaList(list) = msg { 
                            // On ne log que si la liste n'est pas vide pour éviter le spam
                            if !list.is_empty() {
                                println!("📦 UI REÇUE : {} médias", list.len());
                            }
                            media_list.set(list); 
                        }
                    }
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }
        }
    });

    rsx! { Router::<Route> {} }
}


    */


// All the above code is to be rechecked and clean..........


/*
main.rs
*/


/*
mod constants;

//======== MEDIA THREADING ========
mod threading;

use threading::media_thread::launch_media_thread;
use threading::command::Command;
use threading::command::Event;

//======== DATABASE ========
mod database;
use database::db::DB;
use rusqlite::{Connection};

mod media;
mod library;
mod scan;

//mod plugin;
//use plugin::plugin_manager::PluginManager;

//======== GUI ========
//mod gui;
//use gui::route::Route;

use std::time::Duration;
use std::thread::sleep;
use std::sync::mpsc;


use dioxus::prelude::*;
use dioxus::desktop::{Config, WindowBuilder};
use dioxus_router::prelude::*;

use crate::constants::constants::MEDIA_DB_FILE;



use crate::media::image::Image;
*/
/* 
fn main() {

    // ========== MEDIA THREADING ===========


    let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();
    let (evt_tx, evt_rx) = mpsc::channel::<Event>();
    launch_media_thread(cmd_rx, evt_tx);

    // ========== GUI ===========


    /*
    let config = Config::new().with_window(WindowBuilder::new().with_title("NeoKodi").with_resizable(true));

    // 2. Lancer l'application
    LaunchBuilder::desktop().with_cfg(config).launch(|| rsx! { Router::<Route> {} });
    */

    // ========== GUI back test ===========
     
    let mut i = 0;
    loop {
        // Simulate GUI

        //println!("GUI working...");
        if let Ok(event) = evt_rx.try_recv() {
                match event {
                    Event::Finished(id) => println!("Media finished item {id}"),
                    Event::NowPlaying(msg) => println!("MEDIA says: {msg}"),
                    Event::Data(info) => println!("MEDIA info: {info}"),
                    Event::IDList(ids) => println!("MEDIA ID List: {:?}", ids),
                    Event::Info(info) => println!("MEDIA Info: {:?}", info),
                    //medialist
                    _ => {}
                    
                }
        }

        sleep(Duration::from_secs(1));
        let id = 30;
        if i==2 {
            cmd_tx.send(Command::AddMediaToPlaylist(33, 1)).unwrap();
            cmd_tx.send(Command::GetMediaFromPlaylist(1)).unwrap();
            cmd_tx.send(Command::AddTagToMedia(33, 1)).unwrap();
            cmd_tx.send(Command::GetMediaFromTag("chill".to_string())).unwrap();
            
            //cmd_tx.send(Command::Info(id)).unwrap();

           
        }
        if i==5 {
            cmd_tx.send(Command::Pause(id)).unwrap();
        }
        if i==8 {
            cmd_tx.send(Command::Resume(id)).unwrap();
        }
        if i==10 {
            break;
        }
        i += 1;
        
    }
}

*/

/*
use libloading::{Library, Symbol};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

fn main() {
    println!("=== Dynamic Library Loading Demo ===\n");

    #[cfg(target_os = "windows")]
    let lib_path = "plugins/hello_plugin.dll";
    
    #[cfg(target_os = "linux")]
    let lib_path = "plugins/libhello_plugin.so";
    
    #[cfg(target_os = "macos")]
    let lib_path = "plugins/libhello_plugin.dylib";

    println!("Loading library from: {}", lib_path);
    
    // Load the dynamic library
    let lib = match unsafe { Library::new(lib_path) } {
        Ok(lib) => {
            println!("✓ Library loaded successfully!\n");
            lib
        }
        Err(e) => {
            eprintln!("✗ Failed to load library: {}", e);
            eprintln!("\nMake sure you've built the plugin first:");
            eprintln!("  cd plugin_aaa");
            eprintln!("  cargo build");
            std::process::exit(1);
        }
    };

    // Define the function signature
    type GreetFunc = unsafe extern "C" fn(*const c_char) -> *mut c_char;
    type FreeStringFunc = unsafe extern "C" fn(*mut c_char);

    unsafe {
        // Load the greet function
        let greet: Symbol<GreetFunc> = match lib.get(b"greet\0") {
            Ok(func) => func,
            Err(e) => {
                eprintln!("✗ Failed to load 'greet' function: {}", e);
                std::process::exit(1);
            }
        };

        // Load the free_string function
        let free_string: Symbol<FreeStringFunc> = match lib.get(b"free_string\0") {
            Ok(func) => func,
            Err(e) => {
                eprintln!("✗ Failed to load 'free_string' function: {}", e);
                std::process::exit(1);
            }
        };

        // Test the function with different names
        let names = vec!["Alice", "Bob", "Rustacean"];
        
        for name in names {
            println!("Calling plugin with name: {}", name);
            
            // Convert Rust string to C string
            let c_name = CString::new(name).unwrap();
            
            // Call the plugin function
            let result_ptr = greet(c_name.as_ptr());
            
            // Convert the result back to Rust string
            let result = CStr::from_ptr(result_ptr)
                .to_str()
                .unwrap_or("Error: Invalid UTF-8");
            
            println!("  Plugin response: {}\n", result);
            
            // Free the string allocated by the plugin
            free_string(result_ptr);
        }
    }

    println!("=== Demo Complete ===");
}

*/
/*
use libloading::{Library, Symbol};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::env;

fn main() {
    println!("=== Dynamic Library Loading Demo ===\n");

    // Check command line arguments to decide which plugin to load
    let args: Vec<String> = env::args().collect();
    let plugin_name = if args.len() > 1 {
        args[1].as_str()
    } else {
        "musicbrainz"
    };

    let lib_path = match plugin_name {
        "hello" => {
            #[cfg(target_os = "windows")]
            { "plugins/hello_plugin.dll" }
            #[cfg(target_os = "linux")]
            { "../hello-plugin/target/debug/libhello_plugin.so" }
            #[cfg(target_os = "macos")]
            { "../hello-plugin/target/debug/libhello_plugin.dylib" }
        }
        "musicbrainz" => {
            #[cfg(target_os = "windows")]
            { "plugins/musicbrainz_plugin.dll" }
            #[cfg(target_os = "linux")]
            { "../musicbrainz-plugin/target/debug/libmusicbrainz_plugin.so" }
            #[cfg(target_os = "macos")]
            { "../musicbrainz-plugin/target/debug/libmusicbrainz_plugin.dylib" }
        }
        _ => {
            eprintln!("Unknown plugin: {}", plugin_name);
            eprintln!("Available plugins: hello, musicbrainz");
            std::process::exit(1);
        }
    };//TODO recuperer tous les fichiers qui sont dans le bon path

    println!("Loading '{}' plugin from: {}", plugin_name, lib_path);
    
    // Load the dynamic library
    let lib = match unsafe { Library::new(lib_path) } {
        Ok(lib) => {
            println!("✓ Library loaded successfully!\n");
            lib
        }
        Err(e) => {
            eprintln!("✗ Failed to load library: {}", e);
            eprintln!("\nMake sure you've built the plugin first:");
            eprintln!("  cd hello-plugin");
            eprintln!("  cargo build");
            std::process::exit(1);
        }
    };

    // Define the function signature
    type GreetFunc = unsafe extern "C" fn(*const c_char) -> *mut c_char;
    type FreeStringFunc = unsafe extern "C" fn(*mut c_char);

    unsafe {
        // Load the greet function
        let greet: Symbol<GreetFunc> = match lib.get(b"greet\0") {
            Ok(func) => func,
            Err(e) => {
                eprintln!("✗ Failed to load 'greet' function: {}", e);
                std::process::exit(1);
            }
        };

        // Load the free_string function
        let free_string: Symbol<FreeStringFunc> = match lib.get(b"free_string\0") {
            Ok(func) => func,
            Err(e) => {
                eprintln!("✗ Failed to load 'free_string' function: {}", e);
                std::process::exit(1);
            }
        };

        // Test the function with different inputs based on plugin type
        let test_inputs = if plugin_name == "musicbrainz" {
            vec!["The Beatles", "Nirvana", "Metallica"]
        } else {
            vec!["Alice", "Bob", "Rustacean"]
        };
        
        for input in test_inputs {
            println!("Calling plugin with input: {}", input);
            
            // Convert Rust string to C string
            let c_input = CString::new(input).unwrap();
            
            // Call the plugin function
            let result_ptr = greet(c_input.as_ptr());
            
            // Convert the result back to Rust string
            let result = CStr::from_ptr(result_ptr)
                .to_str()
                .unwrap_or("Error: Invalid UTF-8");
            
            println!("  Plugin response: {}\n", result);
            
            // Free the string allocated by the plugin
            free_string(result_ptr);
        }
    }

    println!("=== Demo Complete ===");
}
*/
#![allow(non_snake_case)]

mod constants;
mod threading;
mod database;
mod media;
mod library;
mod scan;
mod config;
mod gui;
mod iptv;

use crate::gui::style::GLOBAL_STYLE;
use crate::gui::route::Route;
use crate::config::AppConfig;
use crate::media::data::{MediaInfo, MediaType};
// 👇 IMPORTANT : On importe la structure définie dans pages.rs
use crate::gui::pages::PluginSearchResult; 

use threading::media_thread::launch_media_thread;
use threading::command::{Command, Event};

use std::sync::mpsc;
use std::path::PathBuf;
use std::time::Duration;
use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::sync::OnceLock;

use dioxus::prelude::*;
use dioxus::desktop::{Config, WindowBuilder};
use dioxus_router::prelude::*;
use warp::Filter;
use tokio::sync::broadcast;
use crate::iptv::parser::TVChannel;

static RELOAD_SIGNAL: OnceLock<broadcast::Sender<()>> = OnceLock::new();

struct Backend {
    tx: mpsc::Sender<Command>,
    rx: RefCell<Option<mpsc::Receiver<Event>>>,
}

fn main() {
    unsafe {
        std::env::set_var("RUST_LOG", "warp=info");
        std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", "--disable-web-security --allow-file-access-from-files --allow-running-insecure-content --autoplay-policy=no-user-gesture-required");
    }

    let (reload_tx, _) = broadcast::channel::<()>(16);
    let _ = RELOAD_SIGNAL.set(reload_tx.clone());
    let reload_tx_server = reload_tx.clone(); 

    // --- THREAD SERVEUR ---
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            loop {
                let app_config = AppConfig::load();
                let server_root = app_config.media_path.clone();
                println!("🌍 SERVEUR : Démarrage sur {}", server_root);

                let mut rx = reload_tx_server.subscribe();
                let media_route = warp::path("media").and(warp::fs::dir(server_root));
                let cors = warp::cors().allow_any_origin().allow_methods(vec!["GET", "HEAD"]);
                
                let (_addr, server) = warp::serve(media_route.with(cors))
                    .bind_with_graceful_shutdown(([127, 0, 0, 1], 3030), async move {
                        let _ = rx.recv().await;
                    });

                server.await;
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        });
    });

    // --- FENÊTRE ---
    let window = WindowBuilder::new()
        .with_title("NeoKodi")
        .with_resizable(true)
        .with_maximized(true);

    let config_dioxus = Config::new()
        .with_window(window)
        .with_custom_head(format!(
            r#"
            <style>{}</style>
            <script src="https://cdn.jsdelivr.net/npm/hls.js@1"></script>
            "#, 
            GLOBAL_STYLE
        ))
        .with_disable_context_menu(false);

    LaunchBuilder::desktop().with_cfg(config_dioxus).launch(App);
}

// === COMPOSANT RACINE ===
fn App() -> Element {
    let backend_channels = use_hook(|| {
        let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();
        let (evt_tx, evt_rx) = mpsc::channel::<Event>();
        
        launch_media_thread(cmd_rx, evt_tx);

        let config = AppConfig::load();
        let root_path = PathBuf::from(config.media_path);
        
        // Initialisation scan
        let _ = cmd_tx.send(Command::AddSource(root_path.clone(), MediaType::Video));
        let _ = cmd_tx.send(Command::AddSource(root_path.clone(), MediaType::Audio));
        let _ = cmd_tx.send(Command::AddSource(root_path, MediaType::Image));
        let _ = cmd_tx.send(Command::GetAllMedia());

        Rc::new(Backend { tx: cmd_tx, rx: RefCell::new(Some(evt_rx)) })
    });

    use_context_provider(|| backend_channels.tx.clone());
    
    let mut media_list = use_context_provider(|| Signal::new(Vec::<MediaInfo>::new()));
    
    // 👇👇👇 C'EST ICI QUE CA SE JOUE 👇👇👇
    // On fournit le type "PluginSearchResult" à l'application.
    // Sans ça, la page Addons plante car elle ne trouve pas ce contexte.
    let mut plugin_result = use_context_provider(|| Signal::new(PluginSearchResult { 
        text: String::from("En attente d'une recherche...") 
    }));
    
    let mut root_path_signal = use_context_provider(|| Signal::new(String::new()));
    let current_config = AppConfig::load();
    root_path_signal.set(current_config.media_path);

    if let Some(tx) = RELOAD_SIGNAL.get() {
        use_context_provider(|| tx.clone());
    }

    let mut iptv_loading = use_context_provider(|| Signal::new(false)); 
    let mut iptv_channels = use_context_provider(|| Signal::new(Vec::<TVChannel>::new()));

    use_coroutine(|_: UnboundedReceiver<()>| {
        let backend = backend_channels.clone();
        async move {
            let mut rx_opt = backend.rx.borrow_mut();
            if let Some(rx) = rx_opt.take() {
                drop(rx_opt);
                loop {
                    if let Ok(msg) = rx.try_recv() {
                        match msg {
                            Event::MediaList(list) => { media_list.set(list); },
                            
                            // 👇 Mise à jour du résultat plugin
                            Event::ArtistInfoReceived(info) => {
                                println!("🧩 UI : Donnée reçue -> {}", info);
                                plugin_result.set(PluginSearchResult { text: info });
                            },

                            Event::NowPlaying(id) => println!("▶️ Lecture ID: {}", id),
                            Event::Info(info) => println!("ℹ️ Info: {:?}", info.title),
                            Event::M3UList(channels) => {
                                println!("📺 UI : Reçu {} chaînes !", channels.len());
                                iptv_channels.set(channels);
                                iptv_loading.set(false); // <--- C'est fini !
                            },
                            _ => {} 
                        }
                    }
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }
        }
    });

    rsx! { Router::<Route> {} }
}