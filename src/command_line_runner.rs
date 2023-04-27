use std::alloc::System;
use std::env::Args;
use std::io::{Read, stdin};
use std::str::FromStr;
use log::error;
use crate::constants::constants::Role;
use crate::models::user::User;
use crate::utils::time::get_current_timestamp_str;

pub fn start_command_line(mut args: Args){
    println!("Starting from command line");
    println!("{:?}", args);
    match args.nth(1).unwrap().as_str() {
        "users"=>{
            println!("User management");
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
                    error!("Command not found")
                }
            }
        }
        _ => {
            error!("Command not found")
        }
    }
}



pub fn read_user_account()->User{
    let mut username = String::new();
    let mut password = String::new();
    let mut role_change = Role::User;

    let role = Role::VALUES.map(|v|{
        return v.to_string()
    }).join(", ");
    retry_read("Enter your username: ", &mut username);
    retry_read("Enter your password: ", &mut password);
    let assigned_role = retry_read_role(&format!("Select your role {}",&role));

    let user = User{
        id: 0,
        username: username.trim_end_matches("\n").parse().unwrap(),
        role: assigned_role.to_string(),
        password: Some(password.trim_end_matches("\n").parse().unwrap()),
        explicit_consent: false,
        created_at: get_current_timestamp_str(),
    };
    println!("{:?}",user);

    user
}

pub fn retry_read(prompt: &str, input: &mut String){
    println!("{}",prompt);
    stdin().read_line(input).unwrap();
    match  input.len()>0{
        true => {
            if input.trim().len()==0{
                retry_read(prompt, input);
            }
        }
        false => {
            retry_read(prompt, input);
        }
    }
}

pub fn retry_read_role(prompt: &str)->Role{
    let mut input = String::new();
    println!("{}",prompt);
    stdin().read_line(&mut input).unwrap();
    let res = Role::from_str(input.as_str().trim_end_matches("\n"));
    if res.is_err(){
        println!("Error setting role. Please choose one of the possible roles.");
        retry_read_role(prompt);
    }
    res.unwrap()
}

