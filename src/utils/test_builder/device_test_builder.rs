#[cfg(test)]
pub mod tests {
    use crate::gpodder::device::dto::device_post::DevicePost;
    use fake::faker::lorem::en::Word;
    use fake::Fake;

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
            DevicePostTestDataBuilder {
                r#type: "laptop".to_string(),
                caption: Word().fake(),
            }
        }

        pub fn build(self) -> DevicePost {
            DevicePost {
                caption: self.caption,
                kind: self.r#type,
            }
        }
    }
}
