//! Locates or extracts a cover image for an audiobook folder.

use std::path::{Path, PathBuf};
use std::process::Command;

const COVER_FILENAMES: &[&str] = &[
    "cover.jpg",
    "cover.jpeg",
    "cover.png",
    "cover.webp",
    "folder.jpg",
    "folder.png",
];

/// Returns a path to an existing cover file inside the folder, if any.
pub fn find_existing_cover(folder: &Path) -> Option<PathBuf> {
    for name in COVER_FILENAMES {
        let candidate = folder.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Extracts embedded artwork from an audio file via ffmpeg into `output_path`.
/// Returns Ok(()) if extracted; Err otherwise. Caller decides what to do if
/// extraction failed (e.g. no video stream).
pub fn extract_embedded_cover(source_audio: &Path, output_path: &Path) -> Result<(), String> {
    let status = Command::new("ffmpeg")
        .args(["-v", "error", "-i"])
        .arg(source_audio)
        .args(["-map", "0:v", "-frames:v", "1", "-y"])
        .arg(output_path)
        .status()
        .map_err(|e| format!("spawn ffmpeg: {e}"))?;
    if !status.success() {
        return Err(format!(
            "ffmpeg exit status: {}",
            status.code().unwrap_or(-1)
        ));
    }
    if !output_path.exists() {
        return Err("ffmpeg produced no output".to_string());
    }
    Ok(())
}
