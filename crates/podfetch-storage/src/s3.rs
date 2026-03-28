use crate::{FileRequest, StorageError};
use common_infrastructure::config::S3Config;
use s3::error::S3Error;
use s3::{Bucket, BucketConfiguration};
use std::future::Future;
use std::pin::Pin;

#[derive(Clone)]
pub struct S3StorageBackend {
    config: S3Config,
}

impl S3StorageBackend {
    pub fn new(config: S3Config) -> Self {
        Self { config }
    }

    async fn handle_write_async(
        &self,
        path: &str,
        content: &mut [u8],
    ) -> Result<(), StorageError> {
        self.get_bucket_async()
            .await?
            .put_object(path, content)
            .await
            .map_err(Self::map_s3_error)?;
        Ok(())
    }

    fn create_bucket(&self) -> Result<(), StorageError> {
        Bucket::create_with_path_style_blocking(
            &self.config.bucket,
            (&self.config).into(),
            (&self.config).into(),
            BucketConfiguration::default(),
        )
        .map_err(Self::map_s3_error)?;
        Ok(())
    }

    async fn create_bucket_async(&self) -> Result<(), StorageError> {
        Bucket::create_with_path_style(
            &self.config.bucket,
            (&self.config).into(),
            (&self.config).into(),
            BucketConfiguration::default(),
        )
        .await
        .map_err(Self::map_s3_error)?;
        Ok(())
    }

    async fn get_bucket_async(&self) -> Result<Box<Bucket>, StorageError> {
        let bucket: Box<Bucket> =
            <&S3Config as Into<Result<Box<Bucket>, S3Error>>>::into(&self.config)
                .map_err(Self::map_s3_error)?;
        let exists = bucket.exists().await.map_err(Self::map_s3_error)?;
        if !exists {
            self.create_bucket_async().await?;
        }

        Ok(bucket)
    }

    fn get_bucket(&self) -> Result<Box<Bucket>, StorageError> {
        let bucket: Box<Bucket> =
            <&S3Config as Into<Result<Box<Bucket>, S3Error>>>::into(&self.config)
                .map_err(Self::map_s3_error)?;

        let exists = bucket.exists_blocking().map_err(Self::map_s3_error)?;
        if !exists {
            self.create_bucket()?;
        }

        Ok(bucket)
    }

    fn get_url_for_file(&self, id: &str) -> String {
        self.config.clone().convert_to_string(id)
    }

    fn prepare_path_resolution(path: &str) -> String {
        format!("/{path}")
    }

    fn map_s3_error(error: S3Error) -> StorageError {
        StorageError::Backend {
            message: error.to_string(),
        }
    }

    pub fn read_file(&self, path: &str) -> Result<String, StorageError> {
        let resp = self
            .get_bucket()?
            .head_object_blocking(Self::prepare_path_resolution(path))
            .map_err(Self::map_s3_error)?;
        Ok(self.get_url_for_file(&resp.0.e_tag.unwrap_or_default()))
    }

    pub fn write_file(&self, path: &str, content: &mut [u8]) -> Result<(), StorageError> {
        self.get_bucket()?
            .put_object_blocking(Self::prepare_path_resolution(path), content)
            .map_err(Self::map_s3_error)?;
        Ok(())
    }

    pub fn write_file_async<'a>(
        &'a self,
        path: &'a str,
        content: &'a mut [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), StorageError>> + Send + 'a>> {
        Box::pin(async move {
            self.handle_write_async(&Self::prepare_path_resolution(path), content)
                .await
        })
    }

    pub fn create_dir(&self, _: &str) -> Result<(), StorageError> {
        Ok(())
    }

    pub fn path_exists(&self, path: &str, req: FileRequest) -> bool {
        match req {
            FileRequest::Directory => true,
            FileRequest::File => self.read_file(path).is_ok(),
            FileRequest::NoopS3 => false,
        }
    }

    pub fn remove_dir(&self, _: &str) -> Result<(), StorageError> {
        Ok(())
    }

    pub fn remove_file(&self, path: &str) -> Result<(), StorageError> {
        self.get_bucket()?
            .delete_object_blocking(path)
            .map_err(Self::map_s3_error)?;
        Ok(())
    }
}
