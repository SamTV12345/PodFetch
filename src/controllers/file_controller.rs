use std::path::{Path, PathBuf};
use rocket::{response, Response};
use rocket::http::Status;
pub struct CachedFile(NamedFile);
use rocket::Request;
use rocket::response::{NamedFile, Responder};

impl <'r> Responder<'r> for CachedFile {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        Response::build_from(self.0.respond_to(req)?)
            .status(Status::Ok)
            //.raw_header("Connection", "keep-alive")
            .ok()
    }
}

#[get("/<file..>")]
pub fn files(file: PathBuf) -> Option<CachedFile> {
    let path = Path::new("podcasts\\").join(file);
    let string = path.to_str().unwrap();
    let lower_case_path = string.to_lowercase();
    if lower_case_path.contains("mp3")|| lower_case_path.contains("m4a") {
         NamedFile::open(path.clone()).ok()
            .map(|nf| CachedFile(nf));
    }
    NamedFile::open(path.clone())
        .ok()
        .map(|nf| CachedFile(nf))
}