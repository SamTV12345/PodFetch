use crate::adapters::persistence::repositories::device::device_repository::DeviceRepositoryImpl;
use crate::application::repositories::device_repository::DeviceRepository;
use crate::application::usecases::devices::create_use_case::CreateUseCase;
use crate::application::usecases::devices::edit_use_case::EditUseCase;
use crate::application::usecases::devices::query_use_case::QueryUseCase;
use crate::domain::models::device::model::Device;
use crate::utils::error::CustomError;

pub struct DeviceService;

impl CreateUseCase for DeviceService {
    fn create(device_to_safe: Device) -> Result<Device, CustomError> {
        DeviceRepositoryImpl::create(device_to_safe)
    }
}

impl QueryUseCase for DeviceService {
    fn query_by_username(username: &str) -> Result<Vec<Device>, CustomError> {
        DeviceRepositoryImpl::get_devices_of_user(username)
    }
}

impl EditUseCase for DeviceService {
    fn delete_by_username(username: &str) -> Result<(), CustomError> {
        DeviceRepositoryImpl::delete_by_username(username)
    }
}
