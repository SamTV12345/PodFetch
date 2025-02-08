#[cfg(test)]
pub mod tests {
    use crate::models::notification::Notification;
    use derive_builder::Builder;
    use fake::faker::chrono::en::Time;
    use fake::{Fake, Faker};

    #[derive(Builder)]
    pub struct NotificationTestDataBuilder {
        pub message: String,
        pub status: String,
        pub created_at: String,
    }

    impl Default for NotificationTestDataBuilder {
        fn default() -> Self {
            Self::new()
        }
    }

    impl NotificationTestDataBuilder {
        pub fn new() -> NotificationTestDataBuilder {
            NotificationTestDataBuilder {
                status: "unread".to_string(),
                message: Faker.fake::<String>(),
                created_at: Time().fake::<String>(),
            }
        }
        pub fn random() -> NotificationTestDataBuilder {
            NotificationTestDataBuilder::new()
        }

        pub fn build(self) -> Notification {
            Notification {
                id: 0,
                message: Faker.fake::<String>(),
                status: self.status,
                created_at: Time().fake(),
                type_of_message: "Download".to_string(),
            }
        }
    }
}
