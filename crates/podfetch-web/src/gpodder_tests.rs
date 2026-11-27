//! gPodder sync: event contracts with the library and the user cleanup.

#[cfg(test)]
mod tests {
    use crate::test_support::tests::{app_state, handle_test_startup, process_events};
    use common_infrastructure::config::EnvironmentService;
    use podfetch_events::Event;
    use podfetch_gpodder::domain::device::{Device, DeviceRepository};
    use podfetch_gpodder::domain::device_sync_group::{DeviceSyncGroup, DeviceSyncGroupRepository};
    use podfetch_gpodder::domain::gpodder_setting::{GpodderSetting, GpodderSettingRepository};
    use podfetch_gpodder::domain::subscription::SubscriptionRepository;
    use podfetch_gpodder::persistence::{
        DeviceRepositoryImpl, DeviceSyncGroupRepositoryImpl, GpodderSettingRepositoryImpl,
        SubscriptionRepositoryImpl,
    };
    use podfetch_identity::events::UserDeleted;
    use podfetch_identity::persistence::adapters::UserAdminRepositoryImpl;
    use podfetch_identity::services::user_auth::service::UserAuthService;
    use podfetch_kernel::UserId;
    use podfetch_persistence::db::database;
    use serial_test::serial;
    use std::sync::Arc;
    use uuid::Uuid;

    /// gPodder publishes, the library reads: same type, version and fields.
    #[test]
    fn library_can_read_subscriptions_changed() {
        let published = podfetch_gpodder::events::SubscriptionsChanged {
            user_id: UserId::from(Uuid::from_u128(7)),
            device: "phone".into(),
            added: vec!["https://a.example/feed".into()],
            removed: vec!["https://b.example/feed".into()],
        };
        let read: podfetch_library::events::SubscriptionsChanged =
            serde_json::from_value(serde_json::to_value(&published).unwrap()).unwrap();
        assert_eq!(
            podfetch_gpodder::events::SubscriptionsChanged::TYPE,
            podfetch_library::events::SubscriptionsChanged::TYPE
        );
        assert_eq!(
            podfetch_gpodder::events::SubscriptionsChanged::VERSION,
            podfetch_library::events::SubscriptionsChanged::VERSION
        );
        assert_eq!(read.user_id, published.user_id);
        assert_eq!(read.added, published.added);
        assert_eq!(read.removed, published.removed);
    }

    /// The library publishes, gPodder reads.
    #[test]
    fn gpodder_can_read_podcast_favor_changed() {
        let published = podfetch_library::events::PodcastFavorChanged {
            user_id: UserId::from(Uuid::from_u128(7)),
            podcast_id: Uuid::from_u128(9),
            favored: true,
        };
        let read: podfetch_gpodder::events::PodcastFavorChanged =
            serde_json::from_value(serde_json::to_value(&published).unwrap()).unwrap();
        assert_eq!(
            podfetch_library::events::PodcastFavorChanged::TYPE,
            podfetch_gpodder::events::PodcastFavorChanged::TYPE
        );
        assert_eq!(
            podfetch_library::events::PodcastFavorChanged::VERSION,
            podfetch_gpodder::events::PodcastFavorChanged::VERSION
        );
        assert_eq!(
            read,
            podfetch_gpodder::events::PodcastFavorChanged {
                user_id: published.user_id,
                podcast_id: published.podcast_id,
                favored: true,
            }
        );
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

    fn seed(user_id: Uuid) {
        DeviceRepositoryImpl::new(database())
            .create(Device {
                id: None,
                deviceid: format!("phone-{user_id}"),
                kind: "mobile".into(),
                name: "Phone".into(),
                user_id,
                chromecast_uuid: None,
                agent_id: None,
                last_seen_at: None,
                ip: None,
                base_url: None,
            })
            .unwrap();
        SubscriptionRepositoryImpl::new(database())
            .update_subscriptions(
                &format!("phone-{user_id}"),
                user_id,
                &["https://example.com/feed.xml".to_string()],
                &[],
            )
            .unwrap();
        DeviceSyncGroupRepositoryImpl::new(database())
            .replace_all(
                user_id,
                vec![DeviceSyncGroup {
                    id: Uuid::now_v7(),
                    user_id,
                    group_id: 1,
                    device_id: format!("phone-{user_id}"),
                }],
            )
            .unwrap();
        GpodderSettingRepositoryImpl::new(database())
            .save_setting(GpodderSetting {
                id: Uuid::now_v7(),
                user_id,
                scope: "account".into(),
                scope_id: None,
                data: "{}".into(),
            })
            .unwrap();
    }

    fn gpodder_data_of(user_id: Uuid) -> (usize, usize, usize, bool) {
        (
            DeviceRepositoryImpl::new(database())
                .get_devices_of_user(user_id)
                .unwrap()
                .len(),
            SubscriptionRepositoryImpl::new(database())
                .get_user_subscriptions(user_id, chrono::NaiveDateTime::default(), 0)
                .unwrap()
                .add
                .len(),
            DeviceSyncGroupRepositoryImpl::new(database())
                .get_by_user_id(user_id)
                .unwrap()
                .len(),
            GpodderSettingRepositoryImpl::new(database())
                .get_setting(user_id, "account", None)
                .unwrap()
                .is_some(),
        )
    }

    /// `identity.user_deleted` removes devices, subscriptions, sync groups
    /// and settings of that user, and only of that user.
    #[tokio::test]
    #[serial]
    async fn user_deleted_removes_gpodder_data() {
        let _ts = handle_test_startup().await;
        let leaving = create_user("leaving_gpodder");
        let staying = create_user("staying_gpodder");
        seed(leaving);
        seed(staying);
        assert_eq!(gpodder_data_of(leaving), (1, 1, 1, true));

        app_state()
            .event_bus
            .publish(&UserDeleted {
                user_id: UserId::from(leaving),
                username: "leaving_gpodder".into(),
            })
            .unwrap();
        process_events().await;

        assert_eq!(gpodder_data_of(leaving), (0, 0, 0, false));
        assert_eq!(gpodder_data_of(staying), (1, 1, 1, true));
    }
}
