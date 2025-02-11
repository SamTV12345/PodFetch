use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::application::services::device::service::DeviceService;
use crate::application::usecases::devices::edit_use_case::EditUseCase;
use crate::constants::inner_constants::Role;
use crate::controllers::sys_info_controller::built_info;
use crate::models::episode::Episode;
use crate::models::favorites::Favorite;
use crate::models::podcasts::Podcast;
use crate::models::session::Session;
use crate::models::subscription::Subscription;
use crate::models::user::{User, UserWithoutPassword};
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::service::rust_service::PodcastService;
use crate::utils::error::{CustomError, CustomErrorInner};
use crate::utils::time::get_current_timestamp_str;
use log::error;
use rpassword::read_password;
use sha256::digest;
use std::env::Args;
use std::io::{stdin, stdout, Error, ErrorKind, Write};
use std::process::exit;
use std::str::FromStr;

pub async fn start_command_line(mut args: Args) -> Result<(), CustomError> {
    println!("Starting from command line");
    let conn = &mut get_connection();
    // Skip first argument
    args.next();
    let arg = match args.next() {
        Some(arg) => arg,
        None => {
            println!("Please provide a command");
            exit(1);
        }
    };

    match arg.as_str() {
        "help" | "--help" => {
            println!(
                r" The following commands are available:
            users => Handles user management
            podcasts => Handles podcast management
            "
            );
            Ok(())
        }
        "podcasts" => {
            println!("Podcast management");
            let podcast_args = match args.next() {
                Some(arg) => arg,
                None => {
                    println!("Please provide a command");
                    exit(1);
                }
            };
            match podcast_args.as_str() {
                "refresh" => {
                    let podcast_rss_feed = args.next();

                    match podcast_rss_feed {
                        Some(feed) => {
                            let conn = &mut get_connection();

                            let replaced_feed = feed.replace(['\'', ' '], "");
                            println!("Refreshing podcast {}", replaced_feed);

                            let podcast = Podcast::get_podcast_by_rss_feed(replaced_feed, conn)
                                .expect("Error getting podcast");

                            PodcastEpisodeService::insert_podcast_episodes(&podcast)?;
                            PodcastService::schedule_episode_download(&podcast)
                        }
                        None => {
                            println!("Please provide a podcast rss feed url");
                            exit(1);
                        }
                    }
                }
                "refresh-all" => {
                    let podcasts = Podcast::get_all_podcasts()?;
                    for podcast in podcasts {
                        println!("Refreshing podcast {}", podcast.name);

                        PodcastEpisodeService::insert_podcast_episodes(&podcast)?;
                        PodcastService::schedule_episode_download(&podcast)?;
                    }
                    Ok(())
                }
                "list" => {
                    let podcasts = Podcast::get_all_podcasts();
                    match podcasts {
                        Ok(podcasts) => {
                            println!("Id - Name - RSS Feed");
                            for podcast in podcasts {
                                println!("{} - {} - {}", podcast.id, podcast.name, podcast.rssfeed);
                            }
                            Ok(())
                        }
                        Err(..) => {
                            println!("Error getting podcasts");
                            Ok(())
                        }
                    }
                }
                "help" | "--help" => {
                    println!(
                        r" The following commands are available:
                    refresh => Refreshes a podcast
                    refresh-all => Refreshes all podcasts
                    list => Lists all podcasts
                    "
                    );
                    Ok(())
                }
                _ => {
                    println!("Unknown command");
                    Err(CustomErrorInner::BadRequest("Unknown command".to_string()).into())
                }
            }
        }
        "users" => {
            println!("User management");
            let user_args = match args.next() {
                Some(arg) => arg,
                None => {
                    println!("Please provide a command");
                    exit(1);
                }
            };
            match user_args.as_str() {
                "add" => {
                    let mut user = read_user_account()?;

                    println!(
                        "Should a user with the following settings be applied {:?}",
                        user
                    );

                    if ask_for_confirmation().is_ok() {
                        user.password =
                            Some(digest(user.password.expect("Error digesting password")));
                        if User::insert_user(&mut user).is_ok() {
                            println!("User succesfully created")
                        }
                    }
                    Ok(())
                }
                "generate" => {
                    let arg = match args.next() {
                        Some(arg) => arg,
                        None => {
                            error!("Command not found");
                            return Err(CustomErrorInner::BadRequest(
                                "Command not found".to_string(),
                            )
                            .into());
                        }
                    };

                    match arg.as_str() {
                        "apiKey" => {
                            User::find_all_users(conn).iter().for_each(|u| {
                                log::info!("Updating api key of user {}", &u.username);
                                User::update_api_key_of_user(
                                    &u.username,
                                    uuid::Uuid::new_v4().to_string(),
                                )
                                .expect("Error updating api key")
                            });
                            Ok(())
                        }
                        _ => {
                            error!("Command not found");
                            Err(
                                CustomErrorInner::BadRequest("Command not found".to_string())
                                    .into(),
                            )
                        }
                    }
                }
                "remove" => {
                    let mut username = String::new();
                    // remove user
                    let available_users = list_users();
                    retry_read(
                        "Please enter the username of the user you want to delete",
                        &mut username,
                    );
                    username = trim_string(&username);
                    match available_users.iter().find(|u| u.username == username) {
                        Some(..) => {
                            Episode::delete_by_username(&username)
                                .expect("Error deleting entries for podcast history item");
                            DeviceService::delete_by_username(&username)
                                .expect("Error deleting devices");
                            Episode::delete_by_username_and_episode(&username, conn)
                                .expect("Error deleting episodes");
                            Favorite::delete_by_username(trim_string(&username), conn)
                                .expect("Error deleting favorites");
                            Session::delete_by_username(&trim_string(&username))
                                .expect("Error deleting sessions");
                            Subscription::delete_by_username(
                                &trim_string(&username),
                                &mut get_connection(),
                            )
                            .expect("TODO: panic message");
                            User::delete_by_username(trim_string(&username), &mut get_connection())
                                .expect("Error deleting user");
                            println!("User deleted");
                            Ok(())
                        }
                        None => {
                            println!("Username not found");
                            Ok(())
                        }
                    }
                }
                "update" => {
                    //update a user
                    list_users();
                    let mut username = String::new();

                    retry_read(
                        "Please enter the username of the user you want to update",
                        &mut username,
                    );
                    username = trim_string(&username);
                    let user = User::find_by_username(username.as_str())?;

                    do_user_update(user);
                    Ok(())
                }
                "list" => {
                    // list users

                    list_users();
                    Ok(())
                }
                "help" | "--help" => {
                    println!(
                        r" The following commands are available:
                    add => Adds a user
                    remove => Removes a user
                    update => Updates a user
                    list => Lists all users
                    "
                    );
                    Ok(())
                }
                _ => {
                    error!("Command not found");
                    Ok(())
                }
            }
        }
        "migration" => {
            error!("Command not found");
            Ok(())
        }
        "debug" => {
            create_debug_message();
            Ok(())
        }
        _ => {
            error!("Command not found");
            Ok(())
        }
    }
}

fn list_users() -> Vec<UserWithoutPassword> {
    let users = User::find_all_users(&mut get_connection());

    users.iter().for_each(|u| {
        println!("|Username|Role|Explicit Consent|Created at|",);
        println!(
            "|{}|{}|{}|{}|",
            u.username, u.role, u.explicit_consent, u.created_at
        );
    });
    users
}

pub fn read_user_account() -> Result<User, CustomError> {
    let mut username = String::new();

    let role = Role::VALUES.map(|v| v.to_string()).join(", ");
    retry_read("Enter your username: ", &mut username);

    let user = User::find_by_username(&username);

    if user.is_err() {
        println!("User does not exist");
    }

    let password = retry_read_secret("Enter your password: ");
    let assigned_role = retry_read_role(&format!("Select your role {}", &role));
    let mut api_key_generated = uuid::Uuid::new_v4().to_string();
    api_key_generated = api_key_generated.replace('-', "");

    let user = User {
        id: 0,
        username: trim_string(&username),
        role: assigned_role.to_string(),
        password: Some(trim_string(&password)),
        explicit_consent: false,
        created_at: get_current_timestamp_str(),
        api_key: Some(api_key_generated),
    };

    Ok(user)
}

pub fn retry_read(prompt: &str, input: &mut String) {
    println!("{}", prompt);
    match stdin().read_line(input) {
        Ok(..) => {}
        Err(..) => {
            println!("Error reading from terminal");
            exit(1);
        }
    }
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
    match stdout().flush() {
        Ok(..) => {}
        Err(..) => {
            println!("Error reading from terminal");
            exit(1);
        }
    }
    let input = match read_password() {
        Ok(input) => input,
        Err(..) => {
            println!("Error reading from terminal");
            exit(1);
        }
    };
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
    match stdin().read_line(&mut input) {
        Ok(..) => {}
        Err(..) => {
            println!("Error reading from terminal");
            exit(1);
        }
    }
    let res = Role::try_from(trim_string(&input));
    match res {
        Err(..) => {
            println!("Error setting role. Please choose one of the possible roles.");
            retry_read_role(prompt)
        }
        Ok(e) => e,
    }
}

fn ask_for_confirmation() -> Result<(), Error> {
    let mut input = String::new();
    println!("Y[es]/N[o]");
    stdin()
        .read_line(&mut input)
        .expect("Error reading from terminal");
    match input.to_lowercase().starts_with('y') {
        true => Ok(()),
        false => Err(Error::new(ErrorKind::WouldBlock, "Interrupted by user.")),
    }
}

fn trim_string(string_to_trim: &str) -> String {
    string_to_trim
        .trim_end_matches('\n')
        .trim()
        .parse()
        .expect("Error parsing string")
}

fn do_user_update(mut user: User) {
    let mut input = String::new();
    println!(
        "The following settings of a user should be updated: {:?}",
        user
    );
    println!(
        "Enter which field of a user should be updated [role, password, \
    consent]"
    );
    stdin()
        .read_line(&mut input)
        .expect("Error reading from terminal");
    input = trim_string(&input);
    match input.as_str() {
        "role" => {
            user.role = Role::to_string(&retry_read_role(
                "Enter the new role [user,\
            uploader or admin]",
            ));
            User::update_user(user).expect("Error updating role");
            println!("Role updated");
        }
        "password" => {
            let mut password = retry_read_secret("Enter the new password");
            password = digest(password);
            user.password = Some(password);
            User::update_user(user).expect("Error updating password");
            println!("Password updated");
        }
        "consent" => {
            user.explicit_consent = !user.explicit_consent;
            User::update_user(user).expect("Error switching consent");
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

    let podcasts = Podcast::get_all_podcasts();

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
