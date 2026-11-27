//! Integration tests of the casting context against the real database.

#[cfg(test)]
mod tests {
    use crate::test_support::tests::{app_state, handle_test_startup};
    use common_infrastructure::config::EnvironmentService;
    use podfetch_casting::devices::{CastDevice, CastDeviceRepository, kind};
    use podfetch_casting::handlers::CLEANUP_ON_USER_DELETED;
    use podfetch_casting::persistence::CastDeviceRepositoryImpl;
    use podfetch_gpodder::domain::device::{Device, DeviceRepository};
    use podfetch_gpodder::persistence::DeviceRepositoryImpl;
    use podfetch_identity::events::UserDeleted;
    use podfetch_identity::persistence::adapters::UserAdminRepositoryImpl;
    use podfetch_identity::services::user_auth::service::UserAuthService;
    use podfetch_kernel::UserId;
    use podfetch_persistence::db::database;
    use serial_test::serial;
    use std::sync::Arc;
    use std::time::Duration;
    use uuid::Uuid;

    fn cast_devices() -> CastDeviceRepositoryImpl {
        CastDeviceRepositoryImpl::new(database())
    }

    fn create_user(name: &str) -> Uuid {
        UserAuthService::new(
            Arc::new(UserAdminRepositoryImpl::new(database())),
            Arc::new(EnvironmentService::for_tests()),
        )
        .create_user(name.to_string(), "user".to_string(), None, false)
        .unwrap()
        .id
    }

    fn mopidy(owner: Uuid, kind: &str, url: &str) -> CastDevice {
        CastDevice {
            id: None,
            deviceid: url.to_string(),
            kind: kind.to_string(),
            name: "Living Room".to_string(),
            user_id: owner,
            chromecast_uuid: Some(Uuid::now_v7().to_string()),
            agent_id: None,
            last_seen_at: None,
            ip: None,
            base_url: Some(url.to_string()),
        }
    }

    fn gpodder_device(owner: Uuid) -> Device {
        Device {
            id: None,
            deviceid: format!("phone-{owner}"),
            kind: "mobile".to_string(),
            name: "Phone".to_string(),
            user_id: owner,
            chromecast_uuid: None,
            agent_id: None,
            last_seen_at: None,
            ip: None,
            base_url: None,
        }
    }

    #[tokio::test]
    #[serial]
    async fn registered_shared_mopidy_is_visible_to_everyone() {
        let _ts = handle_test_startup().await;
        let owner = create_user("cast_owner");
        let viewer = create_user("cast_viewer");

        let created = cast_devices()
            .register(mopidy(owner, kind::MOPIDY_SHARED, "http://m.local:6680"))
            .unwrap();
        assert_eq!(created.base_url.as_deref(), Some("http://m.local:6680"));

        let visible = cast_devices().list_visible_to(viewer).unwrap();
        assert!(visible.iter().any(|d| d.id == created.id));
        let id = created.id.unwrap();
        assert_eq!(
            cast_devices().find_by_id(id).unwrap().unwrap().base_url,
            created.base_url
        );
        assert_eq!(cast_devices().delete_by_id(id).unwrap(), 1);
        assert!(cast_devices().find_by_id(id).unwrap().is_none());
    }

    #[tokio::test]
    #[serial]
    async fn personal_mopidy_is_hidden_from_other_users() {
        let _ts = handle_test_startup().await;
        let owner = create_user("cast_owner");
        let other = create_user("cast_other");

        let created = cast_devices()
            .register(mopidy(owner, kind::MOPIDY_PERSONAL, "http://p.local:6680"))
            .unwrap();

        assert!(
            cast_devices()
                .list_visible_to(owner)
                .unwrap()
                .iter()
                .any(|d| d.id == created.id)
        );
        assert!(
            !cast_devices()
                .list_visible_to(other)
                .unwrap()
                .iter()
                .any(|d| d.id == created.id)
        );
    }

    /// Each context removes only its own rows of the shared devices table:
    /// casting the cast targets, gPodder sync its devices.
    #[tokio::test]
    #[serial]
    async fn user_deleted_removes_each_contexts_devices() {
        let _ts = handle_test_startup().await;
        let state = app_state();
        let leaving = create_user("leaving_caster");
        let staying = create_user("staying_caster");

        cast_devices()
            .register(mopidy(
                leaving,
                kind::MOPIDY_PERSONAL,
                "http://a.local:6680",
            ))
            .unwrap();
        let kept_cast = cast_devices()
            .register(mopidy(
                staying,
                kind::MOPIDY_PERSONAL,
                "http://b.local:6680",
            ))
            .unwrap();
        let gpodder = DeviceRepositoryImpl::new(database());
        gpodder.create(gpodder_device(leaving)).unwrap();

        // Published without deleting the user row, so the handler is what
        // removes the devices, not the foreign key cascade.
        let processor = state.event_bus.spawn_processor();
        state
            .event_bus
            .publish(&UserDeleted {
                user_id: UserId::from(leaving),
                username: "leaving_caster".into(),
            })
            .unwrap();

        let mut done = false;
        for _ in 0..100 {
            done = state
                .event_bus
                .deliveries(None, 100)
                .unwrap()
                .iter()
                .any(|d| d.handler == CLEANUP_ON_USER_DELETED && d.status == "done");
            if done {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        processor.stop(Duration::from_secs(1)).await;
        assert!(done, "cleanup handler ran");

        assert!(cast_devices().list_visible_to(leaving).unwrap().is_empty());
        assert_eq!(
            cast_devices().list_visible_to(staying).unwrap(),
            vec![kept_cast],
            "other users keep their cast devices"
        );
        assert!(
            gpodder.get_devices_of_user(leaving).unwrap().is_empty(),
            "gpodder sync removed its own device"
        );
    }

    /// gPodder's cleanup leaves the cast targets in the shared table alone.
    #[tokio::test]
    #[serial]
    async fn gpodder_device_cleanup_keeps_cast_devices() {
        let _ts = handle_test_startup().await;
        let owner = create_user("mixed_devices");
        let cast = cast_devices()
            .register(mopidy(owner, kind::MOPIDY_PERSONAL, "http://c.local:6680"))
            .unwrap();
        let gpodder = DeviceRepositoryImpl::new(database());
        gpodder.create(gpodder_device(owner)).unwrap();

        gpodder.delete_by_user_id(owner).unwrap();

        assert_eq!(cast_devices().list_visible_to(owner).unwrap(), vec![cast]);
        assert!(gpodder.get_devices_of_user(owner).unwrap().is_empty());
    }

    /// Both contexts read the shared `kind` column; they must agree on
    /// which values are cast targets.
    #[test]
    fn device_kinds_agree_between_contexts() {
        assert_eq!(
            kind::CASTABLE,
            podfetch_gpodder::domain::device::kind::CASTABLE
        );
        for castable in kind::CASTABLE {
            assert!(podfetch_gpodder::domain::device::kind::is_castable(
                castable
            ));
        }
        for gpodder in ["desktop", "laptop", "mobile", "server", "other"] {
            assert!(!podfetch_gpodder::domain::device::kind::is_castable(
                gpodder
            ));
            assert!(!kind::is_castable(gpodder));
        }
    }
}
