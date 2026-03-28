use std::fmt::Display;

pub enum FileType {
    Audio,
    Image,
}

impl Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::Audio => write!(f, "mp3"),
            FileType::Image => write!(f, "jpg"),
        }
    }
}
