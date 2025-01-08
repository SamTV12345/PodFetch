use std::sync::{LazyLock, OnceLock};
use crate::adapters::file::file_handler::{FileHandler, FileRequest};
use crate::utils::error::{map_s3_error, CustomError, CustomErrorInner};

#[derive(Clone)]
pub struct S3Handler;
use s3::{Bucket, BucketConfiguration, Region};
use s3::creds::Credentials;

static S3_BUCKET_CRED: LazyLock<Credentials> = LazyLock::new(||Credentials::default().unwrap());

impl S3Handler {
    pub fn new() -> Self {
        S3Handler
    }
}

static BUCKET_EXISTS: OnceLock<bool> = OnceLock::new();

static BUCKET_NAME: &str = "podcast";

impl S3Handler {
    fn create_bucket() -> Result<(), CustomError> {
        let region = Region::Custom {
            region: "eu-central-1".to_owned(),
            endpoint: "http://localhost:9000".to_owned(),
        };

        Bucket::create_blocking(
            BUCKET_NAME,
            region,
            S3_BUCKET_CRED.clone(),
            BucketConfiguration::default()
        ).map_err(map_s3_error)?;
        Ok(())
    }


    fn get_bucket() -> Result<Box<Bucket>, CustomError> {
        let region = Region::Custom {
            region: "eu-central-1".to_owned(),
            endpoint: "http://localhost:9000".to_owned(),
        };

        let bucket = Bucket::new(
            BUCKET_NAME,
            region.clone(),
            S3_BUCKET_CRED.clone(),
        ).map_err(map_s3_error)?;
        BUCKET_EXISTS.get_or_init(||{
            if !bucket.exists_blocking().map_err(map_s3_error).unwrap_or(false) {
                return Self::create_bucket().is_ok();
            }
           true
        });

        Ok(bucket)
    }

    fn get_url_for_file(id: &str) -> String {
        format!("http://localhost:9000/{}/{}", BUCKET_NAME, id)
    }
}

impl FileHandler for S3Handler {
    fn read_file(&self, path: &str) -> Result<String, CustomError> {
        let resp = Self::get_bucket()?.head_object_blocking(path).map_err(map_s3_error)?;
        Ok(Self::get_url_for_file(&resp.0.e_tag.ok_or::<CustomError>(CustomErrorInner::NotFound
            .into())?))

    }

    fn write_file(&self, path: &str, content: &mut [u8]) -> Result<(), CustomError> {
        Self::get_bucket()?.put_object_blocking(path, content).map_err(map_s3_error)?;
        Ok(())
    }

    fn create_dir(&self, _: &str) -> Result<(), CustomError> {
        Ok(())
    }

    fn path_exists(&self, path: &str, req: FileRequest) -> bool {
        match req {
            FileRequest::Directory => {
                true
            }
            FileRequest::File => {
                self.read_file(path).is_ok()
            },
            FileRequest::NoopS3 => {
                // Some Podfetch internals check if a path already exists before writing. This is
                // to prevent an infinite loop as s3 doesn't have a concept of directories
                false
            }
        }
    }

    fn remove_dir(&self, _: &str) -> Result<(), CustomError> {
        Ok(())
    }

    fn remove_file(&self, path: &str) -> Result<(), CustomError> {
        Self::get_bucket()?.delete_object_blocking(path).map_err(map_s3_error)?;
        Ok(())
    }
}