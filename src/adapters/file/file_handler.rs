use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
pub use common_infrastructure::config::FileHandlerType;
pub use podfetch_storage::FileRequest;

pub fn resolve_file_handler_type(value: Option<String>) -> FileHandlerType {
    match value {
        Some(val) => FileHandlerType::from(val.as_str()),
        None => ENVIRONMENT_SERVICE.default_file_handler.clone(),
    }
}
