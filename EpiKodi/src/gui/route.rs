use dioxus::prelude::*;
use dioxus_router::prelude::*;
use super::pages::*;

#[derive(Routable, Clone)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Home {},

    #[route("/films")]
    Films {},

    #[route("/images")]
    Images {},

    #[route("/music")]
    Music {},

    #[route("/series")]
    Series {},

    #[route("/tv")]
    TV {},

    // 👇 VOILÀ CELUI QUI MANQUAIT
    #[route("/addons")]
    Addons {},

    #[route("/settings")]
    Settings {},

    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}