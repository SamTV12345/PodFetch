use std::io::Write;
use std::path::Path;
use reqwest::ClientBuilder as AsyncClientBuilder;
use crate::service::rust_service::get_url_file_suffix;


pub fn check_if_podcast_episode_downloaded(podcast_id: &str, episode_id: String) ->
                                                                                            bool {
    return Path::new(&format!("podcasts\\{}\\{}",podcast_id, episode_id )).exists()
}

pub fn check_if_podcast_main_image_downloaded(podcast_id: &str) -> bool {
    return Path::new(&format!("podcasts\\{}\\image.png",podcast_id)).exists()
}

pub fn create_podcast_root_directory_exists(){
    if !Path::new("podcasts").exists() {
        std::fs::create_dir("podcasts").expect("Error creating directory");
    }
}

pub fn create_podcast_directory_exists(podcast_id: &str){
    if !Path::new(&format!("podcasts\\{}",podcast_id)).exists() {
        std::fs::create_dir(&format!("podcasts\\{}",podcast_id))
            .expect("Error creating directory");
    }
}

pub async fn download_podcast_image(podcast_id: &str, image_url: &str){
    let client = AsyncClientBuilder::new().build().unwrap();
    let image_response = client.get(image_url).send().await.unwrap();
    let image_suffix = get_url_file_suffix(image_url);
    let mut image_out = std::fs::File::create(format!("podcasts\\{}\\image.{}",
                                                      podcast_id,
                                                      image_suffix))
        .unwrap();
    let bytes  =image_response.bytes().await.unwrap();
    image_out.write_all(&bytes).unwrap();
}