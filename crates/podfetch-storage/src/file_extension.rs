use crate::file_type::FileType;
use file_format::FileFormat;
use reqwest::header::CONTENT_TYPE;
use std::io::Error;
use std::path::Path;
use url::Url;

pub enum DetermineFileExtensionReturn {
    FileExtension(String, Vec<u8>),
    String(String),
}

pub fn determine_file_extension(
    url: &str,
    client: &reqwest::blocking::Client,
    file_type: FileType,
) -> DetermineFileExtensionReturn {
    if let Some(suffix) = get_suffix_by_url(url).ok().filter(|s| !s.is_empty()) {
        return DetermineFileExtensionReturn::String(suffix);
    }

    let fallback = |content_type_ext: Option<String>| {
        content_type_ext.unwrap_or_else(|| file_type.to_string())
    };

    let Ok(response) = client.get(url).send() else {
        return DetermineFileExtensionReturn::String(fallback(None));
    };
    let extension_by_content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|c| c.to_str().ok())
        .and_then(map_content_type_to_extension);

    let Ok(bytes) = response.bytes() else {
        return DetermineFileExtensionReturn::String(fallback(extension_by_content_type));
    };

    let sniffed = FileFormat::from(bytes.as_ref()).to_string().to_lowercase();
    let extension = if is_valid_extension(&sniffed) {
        sniffed
    } else {
        fallback(extension_by_content_type)
    };
    DetermineFileExtensionReturn::FileExtension(extension, bytes.to_vec())
}

fn map_content_type_to_extension(content_type: &str) -> Option<String> {
    let mime_type = content_type
        .split(';')
        .next()
        .unwrap_or(content_type)
        .trim()
        .to_ascii_lowercase();
    let extension = match mime_type.as_str() {
        "audio/mpeg" | "audio/mp3" => "mp3",
        "audio/mp4" | "audio/x-m4a" => "m4a",
        "video/mp4" => "mp4",
        "audio/aac" => "aac",
        "audio/ogg" => "ogg",
        "audio/wav" | "audio/x-wav" | "audio/wave" => "wav",
        _ => return None,
    };
    Some(extension.to_string())
}

fn is_valid_extension(ext: &str) -> bool {
    !ext.is_empty() && ext.len() <= 8 && ext.chars().all(|c| c.is_ascii_alphanumeric())
}

fn get_suffix_by_url(url: &str) -> Result<String, Error> {
    let mut parsed = Url::parse(url).map_err(Error::other)?;
    parsed.set_query(None);
    Ok(Path::new(parsed.path())
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned())
}

#[cfg(test)]
mod tests {
    static URLS: &[&str] = &[
        "http://www.contoso.com/test",
        "http://www.contoso.com/test.jpg",
        "http://www.contoso.com/test.mp3",
        "http://www.contoso.com/test.mp3?Parameter1=42",
        "http://www.contoso.com/test.jpg?Parameter1=test&Parameter2=42",
        "http://www.contoso.com/test?Parameter1=test&Parameter2=42",
    ];

    static URL_EXTENSIONS: &[&str] = &["", "jpg", "mp3", "mp3", "jpg", ""];

    #[test]
    fn stripping_filename_works() {
        for (idx, url) in URLS.iter().enumerate() {
            assert_eq!(super::get_suffix_by_url(url).unwrap(), URL_EXTENSIONS[idx]);
        }
    }
}
