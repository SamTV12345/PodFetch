//! Fetching episode audio and images over HTTP, and PodFetch's default
//! image for episodes and podcasts without one.

use common_infrastructure::config::EnvironmentService;
use common_infrastructure::error::{
    CustomError, CustomErrorInner, ErrorSeverity, map_reqwest_error,
};
use common_infrastructure::http::{get_async_sync_client, get_sync_client};
use podfetch_storage::DetermineFileExtensionReturn;
use reqwest::header::ACCEPT_ENCODING;
use std::thread::sleep;
use std::time::Duration as StdDuration;

const DEFAULT_IMAGE_BYTES: &[u8] = include_bytes!("../../../ui/public/default.jpg");

/// PodFetch's default image as `(suffix, bytes)`.
pub fn default_image() -> (String, Vec<u8>) {
    ("jpg".to_string(), DEFAULT_IMAGE_BYTES.to_vec())
}

fn binary_sync_client(
    environment: &EnvironmentService,
) -> Result<reqwest::blocking::Client, CustomError> {
    get_sync_client(environment)
        .no_gzip()
        .no_brotli()
        .no_deflate()
        .no_zstd()
        .build()
        .map_err(map_reqwest_error)
}

fn binary_async_client(environment: &EnvironmentService) -> Result<reqwest::Client, CustomError> {
    get_async_sync_client(environment)
        .no_gzip()
        .no_brotli()
        .no_deflate()
        .no_zstd()
        .build()
        .map_err(map_reqwest_error)
}

fn fetch_bytes_with_retries(
    environment: &EnvironmentService,
    url: &str,
) -> Result<Vec<u8>, CustomError> {
    const MAX_RETRIES: usize = 3;
    for attempt in 1..=MAX_RETRIES {
        let result = binary_sync_client(environment)?
            .get(url)
            .header(ACCEPT_ENCODING, "identity")
            .send()
            .map_err(map_reqwest_error)
            .and_then(|resp| resp.bytes().map_err(map_reqwest_error))
            .map(|bytes| bytes.as_ref().to_vec());

        match result {
            Ok(bytes) => return Ok(bytes),
            Err(err) if attempt == MAX_RETRIES => return Err(err),
            Err(err) => {
                tracing::warn!(
                    "Download attempt {}/{} failed for {}: {}",
                    attempt,
                    MAX_RETRIES,
                    url,
                    err
                );
                sleep(StdDuration::from_millis(250 * attempt as u64));
            }
        }
    }
    Err(CustomErrorInner::Conflict(
        "Download failed after retries".to_string(),
        ErrorSeverity::Error,
    )
    .into())
}

async fn fetch_bytes_with_retries_async(
    environment: &EnvironmentService,
    url: &str,
) -> Result<Vec<u8>, CustomError> {
    const MAX_RETRIES: usize = 3;
    for attempt in 1..=MAX_RETRIES {
        let client = binary_async_client(environment)?;
        let result = match client
            .get(url)
            .header(ACCEPT_ENCODING, "identity")
            .send()
            .await
        {
            Ok(resp) => resp
                .bytes()
                .await
                .map(|bytes| bytes.as_ref().to_vec())
                .map_err(map_reqwest_error),
            Err(err) => Err(map_reqwest_error(err)),
        };

        match result {
            Ok(bytes) => return Ok(bytes),
            Err(err) if attempt == MAX_RETRIES => return Err(err),
            Err(err) => {
                tracing::warn!(
                    "Async download attempt {}/{} failed for {}: {}",
                    attempt,
                    MAX_RETRIES,
                    url,
                    err
                );
                tokio::time::sleep(StdDuration::from_millis(250 * attempt as u64)).await;
            }
        }
    }
    Err(CustomErrorInner::Conflict(
        "Download failed after retries".to_string(),
        ErrorSeverity::Error,
    )
    .into())
}

/// The suffix and bytes of a file whose type was determined from `url`,
/// downloading it unless the type check already fetched it.
pub fn fetch_with_suffix(
    environment: &EnvironmentService,
    dt: DetermineFileExtensionReturn,
    podcast_episode_url: &str,
) -> Result<(String, Vec<u8>), CustomError> {
    match dt {
        DetermineFileExtensionReturn::FileExtension(suffix, bytes) => Ok((suffix, bytes)),
        DetermineFileExtensionReturn::String(suffix) => {
            let resp = fetch_bytes_with_retries(environment, podcast_episode_url)?;
            Ok((suffix, resp))
        }
    }
}

/// [`fetch_with_suffix`] for async callers.
pub async fn fetch_with_suffix_async(
    environment: &EnvironmentService,
    dt: DetermineFileExtensionReturn,
    podcast_episode_url: &str,
) -> Result<(String, Vec<u8>), CustomError> {
    match dt {
        DetermineFileExtensionReturn::FileExtension(suffix, bytes) => Ok((suffix, bytes)),
        DetermineFileExtensionReturn::String(suffix) => {
            let resp = fetch_bytes_with_retries_async(environment, podcast_episode_url).await?;
            Ok((suffix, resp))
        }
    }
}
