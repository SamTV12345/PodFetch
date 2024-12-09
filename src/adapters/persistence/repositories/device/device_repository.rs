use diesel::RunQueryDsl;
use crate::adapters::persistence::model::device::device_entity::DeviceEntity;
use crate::application::repositories::device_repository::DeviceRepository;
use crate::dbconfig::DBType;
use crate::domain::models::device::model::Device;
use crate::execute_with_conn;
use crate::utils::error::{map_db_error, CustomError};

pub struct DeviceRepositoryImpl;

use diesel::QueryDsl;
use crate::dbconfig::schema::devices::dsl::*;
use diesel::ExpressionMethods;
use DBType as DbConnection;

impl DeviceRepository for DeviceRepositoryImpl {
    fn create(device: Device, conn: &mut DBType) -> Result<Device, CustomError> {
        let device_entity: DeviceEntity = device.into();
        execute_with_conn!(conn, |conn| diesel::insert_into(devices)
            .values(device_entity)
            .get_result(conn)
            .map_err(map_db_error)
            .map(|device_entity: DeviceEntity| device_entity.into()))

    }

    fn get_devices_of_user(username_to_find: String, conn: &mut DBType) -> Result<Vec<Device>,
        CustomError> {
        devices
            .filter(username.eq(username_to_find))
            .load::<DeviceEntity>(conn)
            .map(|device_entity| device_entity.into_iter().map(|device_entity| device_entity.into()).collect())
            .map_err(map_db_error)
    }

    fn delete_by_username(username1: &str, conn: &mut DBType) -> Result<(), CustomError> {
        use crate::dbconfig::schema::devices::dsl::*;
        diesel::delete(devices.filter(username.eq(username1))).execute(conn).map(|_|()).map_err(map_db_error)
    }
}