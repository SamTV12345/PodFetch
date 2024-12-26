pub mod ui_middleware;

use actix_web::web;
use maud::{html, Markup};
use crate::models::user::User;
use crate::service::notification_service::NotificationService;
use crate::ENVIRONMENT_SERVICE;

fn home_icon() -> Markup {
    html! {
        i class="material-icons" {"home"}
    }
}

fn podcast_icon() -> Markup {
    html! {
        i class="material-icons" {"podcasts"}
    }
}

fn favorite_icon() -> Markup {
    html! {
        i class="material-icons" {"favorite"}
    }
}

fn magic_icon() -> Markup {
    html! {
        i class="material-icons" {"magic_button"}
    }
}


fn user_icon() -> Markup {
    html! {
        i class="material-icons" {"account_circle"}
    }
}

fn info_icon() -> Markup {
    html! {
        i class="material-icons" {"info"}
    }
}


fn settings_icon() -> Markup {
    html! {
        i class="material-icons" {"settings"}
    }
}

fn sell_icon() -> Markup {
    html! {
        i class="material-icons" {"sell"}
    }
}

pub fn sidebar() -> Markup {
    html!{
        div class="sidebar" aria-label="Sidebar" {
        span class="logo-container" {
            i class="material-icons"{"auto_detect_voice"};
                span class="logo-text" {"PodFetch"};
            };
        ul {
            (sidebar_item("Home", true, home_icon()))
            (sidebar_item("All Subscriptions", false, podcast_icon()))
            (sidebar_item("Favorites", false, favorite_icon()))
            (sidebar_item("Latest Episodes", false, magic_icon()))
            (sidebar_item("Tags", false, sell_icon()))
            }
        }
    }
}


pub fn sidebar_item(text: &str, active: bool, icon: Markup) -> Markup {
    let sidebar_item_class = if active {
        "sidebar-item active"
    } else {
        "sidebar-item"
    };

    html!{
        li class {
            a class=(sidebar_item_class) href="#" {
                (icon)
                span class="sidebar-item-text" { (text) }
            }
        }
    }
}


fn navbar_settings_dropdown_item(text: &str, target: &str) -> Markup {
    html! {
        a href=(target) class="navbar-settings-dropdown-item" {
            span {(text)}
        }
    }
}

pub fn navbar(requester: &web::ReqData<User>) -> Markup {
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
            div id="user-container" {
                (user_icon());
                div id="user-dropdown" class="hidden" {
                    (info_icon())(navbar_settings_dropdown_item("System information",
                        "/ui-new/system-information"))
                    @if ENVIRONMENT_SERVICE.http_basic {
                      (user_icon())(navbar_settings_dropdown_item("Profil" , "/ui-new/profil"))
                    }
                    (settings_icon())(navbar_settings_dropdown_item("Settings" , "/ui-new/settings"))
                };
            };
        }
    }
}

pub fn homepage(requester: &web::ReqData<User>, main_content: Markup, additional_css: Markup) ->
                                                                                          Markup {
    html!{
        html {
            head {
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta charset="utf-8";
                title {"PodFetch"};
                link rel="stylesheet" href="/ui-new/assets/reset.css";
                link rel="stylesheet" href="/ui-new/assets/index.css";
                (additional_css)
                script type="module" src="/ui-new/assets/index.js" {}
            }
            body {
                div id="root" {
                        div class="main-container" {
                        (sidebar())
                        div class="main-content" {
                            (navbar(&requester))
                            (main_content)
                        }
                        }
                }
            }
        }
    }
}

use crate::adapters::api::models::podcast_episode_dto::PodcastEpisodeDto;
use crate::models::episode::Episode;
use crate::models::misc_models::PodcastWatchedEpisodeModelWithPodcastEpisode;
use crate::service::podcast_episode_service::PodcastEpisodeService;

fn podcast_episode_preview(podcast_episode: PodcastWatchedEpisodeModelWithPodcastEpisode) -> Markup {

    let url_to_use = if podcast_episode.podcast_episode.status == "D" {
        podcast_episode.podcast_episode.local_url
    } else {
        ENVIRONMENT_SERVICE.server_url.to_string() + "proxy/podcast?episodeId=" + podcast_episode
            .podcast_episode.episode_id.as_str()
    };

    html! {
        div class="podcast-episode-preview" {
            div class="image-wrapper"  data-url=(url_to_use){
                img src=(podcast_episode.podcast_episode.local_image_url) alt=(podcast_episode.name) {};
                i class="material-icons" {"play_circle"};
            }
            div class="podcast-episode-preview-text" {
                p {(podcast_episode.name)}
                p {(podcast_episode.podcast.name)}
            }
        }
    }
}

pub fn main_page(requester: &web::ReqData<User>) -> Markup {
    let mut episodes = Episode::get_last_watched_episodes(&requester.username).unwrap();
    episodes.sort_by(|a, b| a.date.cmp(&b.date).reverse());


    html! {
        div class="main-page" {
            div class="main-page-header" {
                h2 {"Startseite"}
                span class="active" {
                    span class="material-icons" {"home"};
                    span class="text" {"Startseite"};
                }
                span {
                    span class="material-icons" {"playlist_play"};
                    span class="text" {"Playlisten"};
                }
            }
            div class="recent-listened" {
                h3 {"Kürzlich gehört"}
                div {
                   @for episode in episodes {
                    (podcast_episode_preview(episode))
                    }
                }
            }
            div class="recently-added" {
                h3 {"Nächste Folgen"}
            }
            audio id="main-audio";
        }
    }
}

pub fn main_page_headers() -> Markup {
    html! {
        link rel="stylesheet" href="/ui-new/assets/components/main_page.css";
        script type="module" src="/ui-new/assets/js/main_page.js" {};
    }
}


pub async fn index_ui(requester: web::ReqData<User>) -> actix_web::error::Result<Markup> {
    Ok(homepage(&requester, main_page(&requester), main_page_headers()))
}