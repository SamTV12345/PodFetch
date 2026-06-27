#[cfg(test)]
pub mod tests {
    use crate::device::DevicePost;
    use fake::Fake;
    use fake::faker::lorem::en::Word;

    pub struct DevicePostTestDataBuilder {
        caption: String,
        r#type: String,
    }

    impl Default for DevicePostTestDataBuilder {
        fn default() -> Self {
            Self::new()
        }
    }

    impl DevicePostTestDataBuilder {
        pub fn new() -> DevicePostTestDataBuilder {
            // A faked lorem `Word()` alone is drawn from a small vocabulary and
            // collides when several devices are built in one test, tripping the
            // `device_sync_groups` UNIQUE constraint. Append a UUID to guarantee
            // uniqueness while keeping a human-readable prefix.
            let word: String = Word().fake();
            DevicePostTestDataBuilder {
                r#type: "laptop".to_string(),
                caption: format!("{word}-{}", uuid::Uuid::new_v4()),
            }
        }

        pub fn build(self) -> DevicePost {
            DevicePost {
                caption: self.caption,
                kind: self.r#type,
            }
        }
    }

    #[test]
    fn build_produces_unique_captions() {
        // Devices created in the same test must not collide on their caption,
        // otherwise sync endpoints hit a UNIQUE constraint. A bare faked lorem
        // `Word()` is drawn from a small vocabulary and collides across a batch,
        // so the caption must carry a guaranteed-unique component.
        use std::collections::HashSet;
        let captions: HashSet<String> = (0..200)
            .map(|_| DevicePostTestDataBuilder::new().build().caption)
            .collect();
        assert_eq!(captions.len(), 200, "device captions must be unique");
    }
}
