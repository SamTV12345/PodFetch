#[derive(Clone, Copy, Debug)]
pub enum FileRequest {
    Directory,
    File,
    NoopS3,
}
