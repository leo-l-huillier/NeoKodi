use dioxus::prelude::*;
use dioxus_router::prelude::*;

// On importe les composants depuis les autres fichiers du module front
use super::layout::AppLayout;
use super::pages::*;

#[derive(Routable, Clone)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AppLayout)] 
        #[route("/")]
        Home {},
        #[route("/tv")]
        TV {},
        #[route("/films")]
        Films {},
        #[route("/series")]
        Series {},
        #[route("/music")]
        Music {},
        #[route("/images")]
        Images {},
        #[route("/addons")]
        Addons {},
        #[route("/settings")]
        Settings {},
    #[end_layout]
    
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}