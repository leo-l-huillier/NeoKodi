use dioxus::prelude::*;
use super::pages::*;

#[derive(Routable, Clone)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Home {},

    #[route("/Videos")]
    Videos {},

    #[route("/images")]
    Images {},

    #[route("/music")]
    Music {},

    #[route("/series")]
    Series {},

    #[route("/iptv")]
    Iptv {},

    // 👇 VOILÀ CELUI QUI MANQUAIT
    #[route("/Plugins")]
    Plugins {},

    #[route("/settings")]
    Settings {},

    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}