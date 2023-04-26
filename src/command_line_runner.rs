use std::env::Args;
use std::fmt::format;
use std::io::stdin;
use diesel::result::Error::RollbackErrorOnCommit;
use crate::constants::constants::Role;
use crate::models::user::User;

pub fn start_command_line(mut args: Args){
    match args.nth(1).unwrap().as_str() {
        "users"=>{
            match args.nth(0).unwrap().as_str() {
                "add"=> {
                    read_user_account();
                }
                "remove"=> {
                    // remove user
                }
                "list"=> {
                    // list users
                }
                _ => {
                    // error
                }
            }
        }
        _ => {}
    }
}



pub fn read_user_account()->User{
    let mut username = String::new();
    let mut password = String::new();
    let role = Role::VALUES.map(|v|{
        return v.to_string()
    }).join(", ");
    retry_read("Enter your username: ", &mut username);
    retry_read("Enter your password: ", &mut password);
    retry_read(&format!("Select your role {}",&role), &mut password);

    User{
        id: 0,
        username,
        role: "".to_string(),
        password: Some(password),
        explicit_consent: false,
        created_at: Default::default(),
    }
}

pub fn retry_read(prompt: &str, input: &mut String){
    print!("{}",prompt);
    match  stdin().read_line(input).unwrap().len()>0{
        Ok(e) => {
            if input.trim().len()>0{
                retry_read(prompt, input);
            }
        }
        Err(e) => {
            print!("Error reading input: {}", e);
            retry_read(prompt, input);
        }
    }

}

