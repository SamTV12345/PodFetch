use notify::{EventKind, RecursiveMode, Watcher};
use std::path::{Path, MAIN_SEPARATOR_STR};
use std::sync::mpsc::channel;
use std::{fs, thread};
use std::collections::HashMap;
use std::ffi::OsStr;
use substring::Substring;

pub struct AudioBookService {}

struct DirEntry {
    pub file_name: String,
    pub kind: DirKind,
}

pub enum DirKind {
    Series,
    Author,
    Title,

}

pub struct LeveledCache {
    depth: i32,
    files: HashMap<std::path::PathBuf, LeveledFiles>,
}


pub enum FileType {
    Dir(LeveledFiles),
    File(FileInfo)
}

pub struct LeveledFiles {
    pub data: HashMap<String, FileType>,
}


pub struct FileInfo {
    pub filename: String,
    pub author: Option<String>,
    pub series: Option<String>,
    pub title: Option<String>
}


fn prepare_path(path: &str, level: usize, root: &str, separator: &str) -> Result<FileInfo, std::io::ErrorKind> {
    let root_start = path.find(root).map(|i|i+root.len());
    match root_start {
        Some(e)=>{
            let root_path = path.substring(e, path.len());
            let split: Vec<&str> = root_path.split(separator).collect();
            if split.len() < level {
                return Err(std::io::ErrorKind::InvalidData)
            }

            if split.len() == 5 {
                return Ok(FileInfo {
                    filename: split[level].to_string(),
                    author: Some(split[1].to_string()),
                    series: Some(split[2].to_string()),
                    title: Some(split[3].to_string())
                })
            }

            if split.len() == 4 {
                return Ok(FileInfo{
                    filename: split[level].to_string(),
                    author: Some(split[1].to_string()),
                    title: Some(split[2].to_string()),
                    series: None
                })
            }

            if split.len() == 3 {
                return Ok(FileInfo{
                    filename: split[level].to_string(),
                    title: Some(split[1].to_string()),
                    author: None,
                    series: None
                })
            }

            if split.len() == 2 {
                return Ok(FileInfo{
                    filename: split[level].to_string(),
                    title: Some(split[level].split(".").collect::<Vec<_>>()[0].to_string()),
                    author: None,
                    series: None
                })
            }


            Err(std::io::ErrorKind::NotFound)
        }
        None=>{
           Err(std::io::ErrorKind::NotFound)
        }
    }
}

const ROOT: &str = "audiobooks";

fn traverse_directory(filename: std::path::PathBuf, level: i32) -> FileType {
    let mut hash_set = HashMap::new();
    for entry in fs::read_dir(filename).unwrap() {
        if entry.is_err() {
            continue;
        }
        let unwrapped_entry = entry.unwrap();
        let meta = fs::metadata(unwrapped_entry.path());
        match meta {
            Ok(m) => {
                if m.is_dir() {
                    let result = traverse_directory(unwrapped_entry.path(), level + 1);
                    if let FileType::Dir(e) = result {
                        hash_set.insert(unwrapped_entry.file_name().to_str().unwrap().to_string(), FileType::Dir(e));
                    }
                } else if m.is_file() {
                    let file_name = unwrapped_entry.file_name();
                    let extension = Path::new(&file_name).extension().and_then(OsStr::to_str);
                    match extension {
                        Some(e) => {
                            match e {
                                "mp4"=> {
                                    let result = prepare_path(file_name.to_str().unwrap(), (level + 1) as usize,ROOT, MAIN_SEPARATOR_STR);
                                    if let Ok(e) = result {
                                        hash_set.insert(e.filename.parse().unwrap(), FileType::File(e));
                                    }

                                }
                                "mp3"=>{

                                }
                                _ => {}
                            }

                        }
                        None => {
                            continue
                        }
                    }
                }
            }
            Err(e) => {}
        }
    }
    let leveled_files = LeveledFiles{
        data: hash_set,
    };
    FileType::Dir(leveled_files)
}

impl AudioBookService {
    pub fn new() -> AudioBookService {
        thread::spawn(|| {
            let (tx, rx) = channel();
            // Create a new Watcher with the channel


            let mut watcher = notify::recommended_watcher(tx).unwrap();

            // Add a path to be watched. All files and directories at that path and
            // below will be monitored for changes.
            watcher.watch(Path::new("./audiobook"), RecursiveMode::Recursive).expect("TODO: panic message");
            println!("Result is");


            loop {
                match rx.recv() {
                    Ok(e) => {
                        match e {
                            Ok(e) => {
                                match e.kind {
                                    EventKind::Any => {}
                                    EventKind::Access(_) => {}
                                    EventKind::Create(_) => {}
                                    EventKind::Modify(_) => {}
                                    EventKind::Remove(_) => {}
                                    EventKind::Other => {}
                                }
                            }
                            Err(e) => {}
                        }
                    }
                    Err(e) => {
                        println!("Error is {:?}", e)
                    }
                }
            }
        });
        AudioBookService{

        }
    }
}


#[cfg(test)]
mod tests {
    use crate::service::audiobook::audio_book_service::prepare_path;
    #[test]
    fn test_prepare_path_complete_path() {
        let path = prepare_path("/mnt/c/users/audiobooks/Terry Goodkind/Sword of Truth/Vol1 - 1994 - Wizards First Rule/Audio Track 1.mp3", 4, "audiobooks", "/");
        assert!(!path.is_err());
        let unwrapped_path = path.unwrap();
        assert_eq!("Terry Goodkind", unwrapped_path.author.unwrap());
        assert_eq!("Sword of Truth", unwrapped_path.series.unwrap());
        assert_eq!("Vol1 - 1994 - Wizards First Rule", unwrapped_path.title.unwrap());
        assert_eq!("Audio Track 1.mp3", unwrapped_path.filename);
    }

    #[test]
    fn test_prepare_path_filename_path() {
        let path = prepare_path("/mnt/c/users/audiobooks/Animal Farm.mp3", 1, "audiobooks", "/");
        assert!(!path.is_err());
        let unwrapped_path = path.unwrap();
        assert_eq!("Animal Farm.mp3", unwrapped_path.filename);
        assert_eq!("Animal Farm", unwrapped_path.title.unwrap());
    }

    #[test]
    fn test_prepare_author_title() {
        let path = prepare_path("/mnt/c/users/audiobooks/Steven Levy/Hackers - Heroes of the Computer Revolution/Audio File.m4a", 3, "audiobooks", "/");
        assert!(!path.is_err());
        let unwrapped_path = path.unwrap();
        assert_eq!("Steven Levy", unwrapped_path.author.unwrap());
        assert_eq!("Hackers - Heroes of the Computer Revolution", unwrapped_path.title.unwrap());
        assert_eq!("Audio File.m4a", unwrapped_path.filename);

    }
}