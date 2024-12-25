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
        div class="navbar" {
            div id="language-wrapper" {
            button id="language-select" {
                i class="material-icons" {"translate"};
                span {"Language"};
                i class="material-icons arrow" {"keyboard_arrow_down"};
                };
            div id="language-show" class="hidden" {
                div {"English"};
                div {"Deutsch"};
                div {"Français"};
                div {"Polski"};
                div {"Español"};
            };
            }
            div id="mode-selector" {
                button id="system-default" {
                    i class="material-icons" {"desktop_windows"};
                };
                button id="light-mode" {
                    i class="material-icons" {"light_mode"};
                };
                button id="dark-mode" {
                    i class="material-icons" {"dark_mode"};
                };
            };

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
                script src="/api/v1/assets/index.js" {}
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