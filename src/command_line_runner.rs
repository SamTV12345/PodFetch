use std::env::Args;
use std::io::{Error, ErrorKind, stdin, stdout, Write};
use std::process::exit;
use std::str::FromStr;
use log::error;
use sha256::{digest};
use crate::config::dbconfig::establish_connection;
use crate::constants::inner_constants::Role;
use crate::models::user::{User, UserWithoutPassword};
use crate::utils::time::get_current_timestamp_str;
use rpassword::read_password;
use crate::controllers::sys_info_controller::built_info;
use crate::models::device::Device;
use crate::models::episode::Episode;
use crate::models::favorites::Favorite;
use crate::models::session::Session;
use crate::models::subscription::Subscription;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::service::rust_service::PodcastService;
use crate::models::podcast_history_item::PodcastHistoryItem;
use crate::models::podcasts::Podcast;
use crate::utils::error::CustomError;


pub fn start_command_line(mut args: Args) {
    println!("Starting from command line");
    match args.nth(1).unwrap().as_str() {
        "help" | "--help" => {
            println!(r" The following commands are available:
            users => Handles user management
            podcasts => Handles podcast management
            ")
        }
        "podcasts" => {
            println!("Podcast management");
            match args.next().unwrap().as_str() {
                "refresh" => {
                    let podcast_rss_feed = args.next();
                    if podcast_rss_feed.is_none() {
                        println!("Please provide a podcast rss feed url");
                        exit(1);
                    }
                    let rss_feed = podcast_rss_feed.clone().unwrap();
                    let mut podcast_service = PodcastService::new();
                    let conn = &mut establish_connection();


                    let replaced_feed = rss_feed.replace(['\'', ' '], "");
                    println!("Refreshing podcast {}", replaced_feed);

                    let podcast = Podcast::get_podcast_by_rss_feed(replaced_feed, conn).expect("Error getting podcast");

                    let mut podcast_episode_service = PodcastEpisodeService::new();
                    podcast_episode_service.insert_podcast_episodes(conn, podcast.clone()).unwrap();
                    podcast_service.schedule_episode_download(podcast, None, conn).unwrap();
                }
                "refresh-all" => {
                    let conn = &mut establish_connection();
                    let podcasts = Podcast::get_all_podcasts(&mut establish_connection());
                    let mut podcast_service = PodcastService::new();
                    for podcast in podcasts.unwrap() {
                        println!("Refreshing podcast {}", podcast.name);

                        let mut podcast_episode_service = PodcastEpisodeService::new();
                        podcast_episode_service.insert_podcast_episodes(&mut establish_connection
                            (), podcast.clone()).unwrap();
                        podcast_service.schedule_episode_download(podcast, None, conn).unwrap();
                    }
                }
                "list" => {
                    let podcasts = Podcast::get_all_podcasts(&mut establish_connection());
                    match podcasts {
                        Ok(podcasts) => {
                            println!("Id - Name - RSS Feed");
                            for podcast in podcasts {
                                println!("{} - {} - {}", podcast.id, podcast.name, podcast.rssfeed);
                            }
                        }
                        Err(..) => {
                            println!("Error getting podcasts");
                        }
                    }
                }
                "help" | "--help" => {
                    println!(r" The following commands are available:
                    refresh => Refreshes a podcast
                    refresh-all => Refreshes all podcasts
                    list => Lists all podcasts
                    ")
                }
                _ => {
                    println!("Unknown command");
                }
            }
        }
        "users" => {
            println!("User management");
            match args.next().unwrap().as_str() {
                "add" => {
                    let mut user = read_user_account().unwrap();


                    println!("Should a user with the following settings be applied {:?}", user);

                    if ask_for_confirmation().is_ok() {
                        user.password = Some(digest(user.password.unwrap()));
                        if User::insert_user(&mut user, &mut establish_connection()).is_ok() {
                            println!("User succesfully created")
                        }
                    }
                }
                "remove" => {
                    let mut username = String::new();
                    // remove user
                    let available_users = list_users();
                    retry_read("Please enter the username of the user you want to delete",
                               &mut username);
                    username = trim_string(username);
                    println!("{}", username);
                    match available_users.iter().find(|u| u.username == username) {
                        Some(..) => {
                            PodcastHistoryItem::delete_by_username(trim_string(username.clone()),
                                                                   &mut establish_connection())
                                .expect("Error deleting entries for podcast history item");
                            Device::delete_by_username(username.clone(), &mut
                                establish_connection())
                                .expect("Error deleting devices");
                            Episode::delete_by_username_and_episode(trim_string(username.clone()),
                                                                    &mut establish_connection())
                                .expect("Error deleting episodes");
                            Favorite::delete_by_username(trim_string(username.clone()),
                                                         &mut establish_connection())
                                .expect("Error deleting favorites");
                            Session::delete_by_username(&trim_string(username.clone()),
                                                        &mut establish_connection())
                                .expect("Error deleting sessions");
                            Subscription::delete_by_username(&trim_string(username.clone()),
                                                             &mut establish_connection()).expect("TODO: panic message");
                            User::delete_by_username(trim_string(username.clone()),
                                                     &mut establish_connection())
                                .expect("Error deleting user");
                            println!("User deleted")
                        }
                        None => {
                            println!("Username not found")
                        }
                    }
                }
                "update" => {
                    //update a user
                    list_users();
                    let mut username = String::new();

                    retry_read("Please enter the username of the user you want to update",
                               &mut username);
                    username = trim_string(username);
                    println!(">{}<", username);
                    let user = User::find_by_username(username.as_str(), &mut
                        establish_connection()).unwrap();

                    do_user_update(user)
                }
                "list" => {
                    // list users

                    list_users();
                }
                "help" | "--help" => {
                    println!(r" The following commands are available:
                    add => Adds a user
                    remove => Removes a user
                    update => Updates a user
                    list => Lists all users
                    ")
                }
                _ => {
                    error!("Command not found")
                }
            }
        }
        "debug" => {
            create_debug_message();
        }
        _ => {
            error!("Command not found")
        }
    }
}

fn list_users() -> Vec<UserWithoutPassword> {
    let users = User::find_all_users(&mut establish_connection());

    users.iter().for_each(|u| {
        println!("|Username|Role|Explicit Consent|Created at|", );
        println!("|{}|{}|{}|{}|", u.username, u.role, u.explicit_consent, u.created_at);
    });
    users
}


pub fn read_user_account() -> Result<User, CustomError> {
    let mut username = String::new();

    let role = Role::VALUES.map(|v| {
        v.to_string()
    }).join(", ");
    retry_read("Enter your username: ", &mut username);

    let user = User::find_by_username(&username, &mut establish_connection());

    if user.is_err() {
        println!("User does not exist");
    }

    let password = retry_read_secret("Enter your password: ");
    let assigned_role = retry_read_role(&format!("Select your role {}", &role));

    let user = User {
        id: 0,
        username: trim_string(username.clone()),
        role: assigned_role.to_string(),
        password: Some(trim_string(password)),
        explicit_consent: false,
        created_at: get_current_timestamp_str(),
    };

    Ok(user)
}

pub fn retry_read(prompt: &str, input: &mut String) {
    println!("{}", prompt);
    stdin().read_line(input).unwrap();
    match !input.is_empty() {
        true => {
            if input.trim().is_empty() {
                retry_read(prompt, input);
            }
        }
        false => {
            retry_read(prompt, input);
        }
    }
}


pub fn retry_read_secret(prompt: &str) -> String {
    println!("{}", prompt);
    stdout().flush().unwrap();
    let input = read_password().unwrap();
    match !input.is_empty() {
        true => {
            if input.trim().is_empty() {
                retry_read_secret(prompt);
            }
        }
        false => {
            retry_read_secret(prompt);
        }
    }
    input
}

pub fn retry_read_role(prompt: &str) -> Role {
    let mut input = String::new();
    println!("{}", prompt);
    stdin().read_line(&mut input).unwrap();
    let res = Role::from_str(&trim_string(input));
    match res {
        Err(..) => {
            println!("Error setting role. Please choose one of the possible roles.");
            retry_read_role(prompt)
        }
        Ok(..) => {
            res.unwrap()
        }
    }
}

fn ask_for_confirmation() -> Result<(), Error> {
    let mut input = String::new();
    println!("Y[es]/N[o]");
    stdin().read_line(&mut input).expect("Error reading from terminal");
    match input.to_lowercase().starts_with('y') {
        true => Ok(()),
        false => Err(Error::new(ErrorKind::WouldBlock, "Interrupted by user."))
    }
}


fn trim_string(string_to_trim: String) -> String {
    string_to_trim.trim_end_matches('\n').trim().parse().unwrap()
}


fn do_user_update(mut user: User) {
    let mut input = String::new();
    println!("The following settings of a user should be updated: {:?}", user);
    println!("Enter which field of a user should be updated [role, password, \
    consent]");
    stdin().read_line(&mut input)
        .expect("Error reading from terminal");
    input = trim_string(input);
    match input.as_str() {
        "role" => {
            user.role = Role::to_string(&retry_read_role("Enter the new role [user,\
            uploader or admin]"));
            User::update_user(user, &mut establish_connection())
                .expect("Error updating role");
            println!("Role updated");
        }
        "password" => {
            let mut password = retry_read_secret("Enter the new password");
            password = digest(password);
            user.password = Some(password);
            User::update_user(user, &mut establish_connection())
                .expect("Error updating password");
            println!("Password updated");
        }
        "consent" => {
            user.explicit_consent = !user.explicit_consent;
            User::update_user(user, &mut establish_connection())
                .expect("Error switching consent");
            println!("Consent preference switched");
        }
        _ => {
            println!("Field not found");
        }
    }
}


pub fn create_debug_message() {
    println!("OS: {}", built_info::CFG_OS);
    println!("Target: {}", built_info::TARGET);
    println!("Endian: {}", built_info::CFG_ENDIAN);
    println!("Debug: {}", built_info::DEBUG);
    println!("Git Version: {:?}", "");
    println!("Git Commit Hash: {:?}", "");
    println!("Git Head Ref: {:?}", "");
    println!("Build Time: {}", built_info::BUILT_TIME_UTC);
    println!("Version: {}", built_info::PKG_VERSION);
    println!("Authors: {}", built_info::PKG_AUTHORS);
    println!("Name: {}", built_info::PKG_NAME);
    println!("Description: {}", built_info::PKG_DESCRIPTION);
    println!("Homepage: {}", built_info::PKG_HOMEPAGE);
    println!("Repository: {}", built_info::PKG_REPOSITORY);
    println!("Rustc Version: {}", built_info::RUSTC_VERSION);
    println!("Rustc: {}", built_info::RUSTC_VERSION);

    let podcasts = Podcast::get_all_podcasts(&mut establish_connection());

    match podcasts {
        Ok(podcasts) => {
            podcasts.iter().for_each(|p| {
                println!("Podcast: {:?}", p);
            });
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}
