use crate::service::podcast_episode_service::PodcastEpisodeService;


pub enum FileType {
    Audio,
    Image
}

//
// File extension determination
// It determines the current file extension
// 1. By the mime type
// 2. By the file name
// 3. By a set of extensions
pub fn determine_file_extension(url: &str, client: &reqwest::blocking::Client, file_type: FileType)
    ->
                                                                                           String{
    let result = client.head(url).send();

    return match result {
        Ok(response) => {
            let mime_type = response.headers().get("content-type");
            match mime_type {
                Some(mime_type) => {
                    let complete_extension = mime_type.to_str().unwrap();
                    if let Some(extension) =  complete_extension.split('/').last() {
                            let file_extension = extension;
                            if complete_extension.contains("audio") || complete_extension.contains("image") {
                                if file_extension.contains(';'){
                                    let f_ext = file_extension.split(';').next().unwrap().to_string();
                                    return f_ext;
                                }
                                return file_extension.to_string();
                            }
                    }


                    get_suffix_by_url(url, &file_type)
                },
                None => {
                    get_suffix_by_url(url, &file_type)
                }
            }
        },
        Err(..) => {
            get_suffix_by_url(url, &file_type)
        }
    }
}

fn get_suffix_by_url(url: &str, file_type: &FileType) -> String {
    let result_suffix = PodcastEpisodeService::get_url_file_suffix(url);
    match result_suffix {
        Ok(suffix) => suffix,
        Err(..) => {
            match file_type {
                FileType::Audio => "mp3".to_string(),
                FileType::Image => "jpg".to_string()
            }
        }
    }
}