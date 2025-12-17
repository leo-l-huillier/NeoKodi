use dioxus::prelude::*;
use dioxus_router::prelude::*;
use super::route::Route;       // On a besoin de connaître les routes pour les liens
use super::style::GLOBAL_STYLE; // On importe le style

pub fn AppLayout() -> Element {
    rsx! {
        style { "{GLOBAL_STYLE}" }
        div { class: "container",
            // -------- MENU LATERAL --------
            nav { class: "sidebar",
                Link { to: Route::Home {}, class: "nav-item", "Accueil" }
                Link { to: Route::TV {}, class: "nav-item", "TV" }
                Link { to: Route::Films {}, class: "nav-item", "Films" }
                Link { to: Route::Series {}, class: "nav-item", "Séries" }
                Link { to: Route::Music {}, class: "nav-item", "Musique" }
                Link { to: Route::Images {}, class: "nav-item", "Images" }
                Link { to: Route::Addons {}, class: "nav-item", "Add-ons" }
                Link { to: Route::Settings {}, class: "nav-item", "Paramètres" }
            }

            // -------- ZONE CONTENU DYNAMIQUE --------
            main { class: "content",
                Outlet::<Route> {}
            }
        }
    }
}