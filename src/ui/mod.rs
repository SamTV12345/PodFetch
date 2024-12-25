pub mod ui_middleware;

use actix_web::web;
use maud::{html, Markup};
use crate::models::user::User;
use crate::service::notification_service::NotificationService;

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


pub fn navbar(requester: web::ReqData<User>) -> Markup {
    let notifications = NotificationService::get_unread_notifications().unwrap();

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
            div id="notification-container" {
              i class="material-icons" id="notification-bell" {"notifications"};
              div id="notification-dropdown" class="hidden" {
                    div class="arrow-up" {};
                    @for notification in notifications {
                        div class="notification-item" data-id=(notification.id) {
                            span {(notification.message)};
                            i class="material-icons" {"close"};
                        };
                    }
                }
            };
        }
    }
}

pub fn homepage(requester: web::ReqData<User>) -> Markup {
    html!{
        html {
            head {
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta charset="utf-8";
                title {"PodFetch"};
                link rel="stylesheet" href="/ui-new/assets/reset.css";
                link rel="stylesheet" href="/ui-new/assets/index.css";
                script src="/ui-new/assets/index.js" {}
            }
            body {
                div id="root" {
                        div class="main-container" {
                        (sidebar())
                        div class="main-content" {
                            (navbar(requester))
                        }
                        }
                }
            }
        }
    }
}


pub async fn index_ui(requester: web::ReqData<User>) -> actix_web::error::Result<Markup> {
    println!("{:?}", requester);
    Ok(homepage(requester))
}