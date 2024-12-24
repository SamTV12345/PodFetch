mod ui_middleware;

use maud::{html, Markup};

pub fn sidebar() -> Markup {
    html!{
        div class="sidebar" aria-label="Sidebar" {
        span class="logo-container" {
            i class="material-icons"{"auto_detect_voice"};
                span class="logo-text" {"PodFetch"};
            };
        ul {
            (sidebar_item("Home", true))
            (sidebar_item("All Subscriptions", false))
            (sidebar_item("Favorites", false))
            (sidebar_item("Latest Episodes", false))
            (sidebar_item("Tags", false))
            }
        }
    }
}


pub fn sidebar_item(text: &str, active: bool) -> Markup {
    let sidebar_item_class = if active {
        "sidebar-item active"
    } else {
        "sidebar-item"
    };

    html!{
        li class {
            a class=(sidebar_item_class) href="#" {
                span class="sidebar-item-text" { (text) }
            }
        }
    }
}


pub fn navbar() -> Markup {
    html!{
        div class="navbar" onclick="toggleLanguageDropdown()" {
            select default="English" {
                option {"German"}
                option { "English"}
                option { "Français"}
                option {"Polski"}
                option { "Español"}
            }
        }
    }
}

pub fn homepage() -> Markup {
    html!{
        html class="dark" {
            head {
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta charset="utf-8";
                title {"PodFetch"};
                link rel="stylesheet" href="/api/v1/assets/reset.css";
                link rel="stylesheet" href="/api/v1/assets/index.css";
            }
            body {
                div id="root" {
                        div class="main-container" {
                        (sidebar())
                        div class="main-content" {
                            (navbar())
                        }
                        }
                }
            }
        }
    }
}


pub async fn index_ui() -> actix_web::error::Result<Markup> {
    Ok(homepage())
}