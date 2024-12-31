use crate::adapters::persistence::model::device::device_entity::DeviceEntity;
use crate::application::repositories::device_repository::DeviceRepository;
use crate::domain::models::device::model::Device;
use crate::execute_with_conn;
use crate::utils::error::{map_db_error, CustomError};
use diesel::RunQueryDsl;

pub struct DeviceRepositoryImpl;

use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::schema::devices::dsl::*;
use diesel::ExpressionMethods;
use diesel::QueryDsl;

impl DeviceRepository for DeviceRepositoryImpl {
    fn create(device: Device) -> Result<Device, CustomError> {
        let device_entity: DeviceEntity = device.into();
        execute_with_conn!(|conn| diesel::insert_into(devices)
            .values(device_entity)
            .get_result(conn)
            .map_err(map_db_error)
            .map(|device_entity: DeviceEntity| device_entity.into()))
    }

    fn get_devices_of_user(username_to_find: &str) -> Result<Vec<Device>, CustomError> {
        devices
            .filter(username.eq(username_to_find))
            .load::<DeviceEntity>(&mut get_connection())
            .map(|device_entity| {
                device_entity
                    .into_iter()
                    .map(|device_entity| device_entity.into())
                    .collect()
            })
            .map_err(map_db_error)
    }

    fn delete_by_username(username1: &str) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::devices::dsl::*;
        diesel::delete(devices.filter(username.eq(username1)))
            .execute(&mut get_connection())
            .map(|_| ())
            .map_err(map_db_error)
    }
}
