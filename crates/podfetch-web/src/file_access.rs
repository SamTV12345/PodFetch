use std::fmt::Display;

#[derive(Debug, thiserror::Error)]
pub enum FileAccessError<E: Display> {
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("{0}")]
    Service(E),
}

pub fn check_permissions_for_files<E, IsApiKeyValid, FindEpisodeByUrl, FindPodcastByImage>(
    uri_path: &str,
    api_key: Option<String>,
    any_auth_enabled: bool,
    server_url: &str,
    is_api_key_valid: IsApiKeyValid,
    find_episode_by_url: FindEpisodeByUrl,
    find_podcast_by_image: FindPodcastByImage,
) -> Result<(), FileAccessError<E>>
where
    E: Display,
    IsApiKeyValid: Fn(&str) -> bool,
    FindEpisodeByUrl: Fn(&str) -> Result<Option<String>, E>,
    FindPodcastByImage: Fn(&str) -> Result<bool, E>,
{
    if any_auth_enabled {
        let Some(api_key) = api_key.as_deref() else {
            return Err(FileAccessError::BadRequest(
                "No query parameters found".to_string(),
            ));
        };
        if !is_api_key_valid(api_key) {
            return Err(FileAccessError::Forbidden);
        }
    }

    let requested_path = uri_path.replace(server_url, "");
    let requested_path = requested_path.trim_start_matches('/').to_string();
    let decoded_path = urlencoding::decode(&requested_path)
        .map_err(|_| FileAccessError::BadRequest("Error while decoding URL".to_string()))?;
    let decoded_path = decoded_path.as_ref();

    let podcast_episode = find_episode_by_url(decoded_path).map_err(FileAccessError::Service)?;
    if podcast_episode.is_some() {
        return Ok(());
    }

    let podcast_found =
        find_podcast_by_image(&requested_path).map_err(FileAccessError::Service)?;
    if podcast_found {
        Ok(())
    } else {
        Err(FileAccessError::NotFound)
    }
}

