use crate::adapters::file::file_handler::{FileHandler, FileRequest};
use crate::utils::error::{map_s3_error, CustomError};
use futures_util::TryFutureExt;
use std::future::Future;
use std::pin::Pin;
use std::sync::LazyLock;

#[derive(Clone)]
pub struct S3Handler;

use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::service::environment_service::S3Config;
use s3::error::S3Error;
use s3::{Bucket, BucketConfiguration};

pub static S3_BUCKET_CONFIG: LazyLock<S3Config> =
    LazyLock::new(|| ENVIRONMENT_SERVICE.s3_config.clone());

impl S3Handler {
    async fn handle_write_async(path: &str, content: &mut [u8]) -> Result<(), CustomError> {
        Self::get_bucket_async()
            .await?
            .put_object(path, content)
            .await
            .map_err(map_s3_error)?;
        Ok(())
    }

    fn create_bucket() -> Result<(), CustomError> {
        Bucket::create_with_path_style_blocking(
            &S3_BUCKET_CONFIG.bucket,
            (&*S3_BUCKET_CONFIG).into(),
            (&*S3_BUCKET_CONFIG).into(),
            BucketConfiguration::default(),
        )
        .map_err(map_s3_error)?;
        Ok(())
    }

    async fn create_bucket_async() -> Result<(), CustomError> {
        Bucket::create_with_path_style(
            &S3_BUCKET_CONFIG.bucket,
            (&*S3_BUCKET_CONFIG).into(),
            (&*S3_BUCKET_CONFIG).into(),
            BucketConfiguration::default(),
        )
        .map_err(map_s3_error)
        .await?;
        Ok(())
    }

    async fn get_bucket_async() -> Result<Box<Bucket>, CustomError> {
        let bucket: Box<Bucket> =
            <&S3Config as Into<Result<Box<Bucket>, S3Error>>>::into(&S3_BUCKET_CONFIG)
                .map_err(map_s3_error)?;
        if !bucket
            .exists()
            .await
            .map_err(map_s3_error)
            .expect("Error checking if bucket exists")
        {
            Self::create_bucket_async().await?;
        }

        Ok(bucket)
    }

    fn get_bucket() -> Result<Box<Bucket>, CustomError> {
        let bucket: Box<Bucket> =
            <&S3Config as Into<Result<Box<Bucket>, S3Error>>>::into(&S3_BUCKET_CONFIG)
                .map_err(map_s3_error)?;

        if !bucket
            .exists_blocking()
            .map_err(map_s3_error)
            .unwrap_or(false)
        {
            Self::create_bucket()?;
        }

        Ok(bucket)
    }

    fn get_url_for_file(id: &str) -> String {
        S3_BUCKET_CONFIG.clone().convert_to_string(id)
    }

    fn prepare_path_resolution(str: &str) -> String {
        format!("/{}", str)
    }
}

impl FileHandler for S3Handler {
    fn read_file(path: &str) -> Result<String, CustomError> {
        println!("Reading file {}", path);
        let resp = Self::get_bucket()?
            .head_object_blocking(Self::prepare_path_resolution(path))
            .map_err(map_s3_error)?;
        println!("Response {:?}", resp.0.e_tag);
        Ok(Self::get_url_for_file(&resp.0.e_tag.unwrap()))
    }

    fn write_file(path: &str, content: &mut [u8]) -> Result<(), CustomError> {
        Self::get_bucket()?
            .put_object_blocking(Self::prepare_path_resolution(path), content)
            .map_err(map_s3_error)?;
        Ok(())
    }

    fn write_file_async<'a>(
        path: &'a str,
        content: &'a mut [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), CustomError>> + 'a>> {
        Box::pin(async {
            Self::handle_write_async(&Self::prepare_path_resolution(path), content).await
        })
    }

    fn create_dir(_: &str) -> Result<(), CustomError> {
        Ok(())
    }

    fn path_exists(path: &str, req: FileRequest) -> bool {
        match req {
            FileRequest::Directory => true,
            FileRequest::File => Self::read_file(path).is_ok(),
            FileRequest::NoopS3 => {
                // Some Podfetch internals check if a path already exists before writing. This is
                // to prevent an infinite loop as s3 doesn't have a concept of directories
                false
            }
        }
    }

    fn remove_dir(_: &str) -> Result<(), CustomError> {
        Ok(())
    }

    fn remove_file(path: &str) -> Result<(), CustomError> {
        Self::get_bucket()?
            .delete_object_blocking(path)
            .map_err(map_s3_error)?;
        Ok(())
    }
}
