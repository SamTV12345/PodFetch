use crate::constants::constants::ITUNES_URL;
use reqwest::{Request, Response};
use reqwest::blocking::ClientBuilder;
use crate::models::itunes_models::ResponseModel;


pub fn find_podcast(podcast: &str)-> ResponseModel {
    let client = ClientBuilder::new().build().unwrap();
    println!("{}",ITUNES_URL.to_owned()+podcast);
    let result = client.get(ITUNES_URL.to_owned()+podcast).send().unwrap();
    return result.json::<ResponseModel>().unwrap();
}