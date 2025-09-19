#[cfg(test)]
pub mod tests {
    use crate::models::podcasts::Podcast;
    use derive_builder::Builder;
    use fake::Fake;
    use fake::Faker;
    use fake::faker::lorem::de_de::Word;

    #[derive(Default, Builder, Debug)]
    #[builder(setter(into), default)]
    pub struct PodcastTestData {
        pub id: i32,
        pub name: String,
        pub directory_id: String,
        pub rssfeed: String,
        pub image_url: String,
        pub summary: Option<String>,
        pub language: Option<String>,
        pub explicit: Option<String>,
        pub keywords: Option<String>,
        pub last_build_date: Option<String>,
        pub author: Option<String>,
        pub active: bool,
        pub original_image_url: String,
        pub directory_name: String,
        pub download_location: Option<String>,
        pub guid: Option<String>,
    }

    impl PodcastTestData {
        pub fn new() -> PodcastTestData {
            let num_of_keywords: i32 = Faker.fake();
            let keywords: String = (0..num_of_keywords)
                .map(|_| Word().fake())
                .collect::<Vec<String>>()
                .join(",");

            PodcastTestData {
                id: Faker.fake(),
                name: Faker.fake(),
                directory_id: Faker.fake(),
                rssfeed: Faker.fake(),
                image_url: Faker.fake(),
                summary: Some(Faker.fake()),
                language: Some(Faker.fake()),
                explicit: Some(Faker.fake()),
                keywords: Some(keywords),
                last_build_date: Faker.fake(),
                author: Faker.fake(),
                active: Faker.fake(),
                original_image_url: Faker.fake(),
                directory_name: Faker.fake(),
                download_location: None,
                guid: Some(Faker.fake()),
            }
        }

        pub fn build(self) -> Podcast {
            Podcast {
                id: self.id,
                name: self.name,
                directory_id: self.directory_id,
                rssfeed: self.rssfeed,
                image_url: self.image_url,
                summary: self.summary,
                author: self.author,
                active: self.active,
                original_image_url: self.original_image_url,
                directory_name: self.directory_name,
                download_location: self.download_location,
                explicit: self.explicit,
                last_build_date: self.last_build_date,
                keywords: self.keywords,
                language: self.language,
                guid: self.guid,
            }
        }
    }

    impl From<PodcastTestData> for Podcast {
        fn from(value: PodcastTestData) -> Self {
            Podcast {
                id: value.id,
                name: value.name,
                directory_id: value.directory_id,
                keywords: value.keywords,
                language: value.language,
                active: value.active,
                original_image_url: value.original_image_url,
                directory_name: value.directory_name,
                download_location: value.download_location,
                explicit: value.explicit,
                last_build_date: value.last_build_date,
                author: value.author,
                summary: value.summary,
                image_url: value.image_url,
                rssfeed: value.rssfeed,
                guid: value.guid,
            }
        }
    }
}
