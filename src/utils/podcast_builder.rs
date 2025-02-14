use rss::extension::itunes::ITunesCategory;

pub struct PodcastBuilder {
    id: i32,
    description: String,
    language: String,
    keywords: String,
    last_build_date: String,
    explicit: bool,
    author: String,
    guid: Option<String>
}

#[derive(Clone)]
pub struct PodcastExtra {
    pub id: i32,
    pub description: String,
    pub language: String,
    pub keywords: String,
    pub last_build_date: String,
    pub explicit: bool,
    pub author: String,
    pub guid: Option<String>
}

impl PodcastBuilder {
    pub fn new(podcast_id: i32) -> PodcastBuilder {
        PodcastBuilder {
            id: podcast_id,
            description: "".to_string(),
            language: "".to_string(),
            keywords: "".to_string(),
            last_build_date: "".to_string(),
            explicit: false,
            author: "".to_string(),
            guid: None
        }
    }

    pub fn description(&mut self, description: String) -> &mut PodcastBuilder {
        self.description = description;
        self
    }

    pub fn language(&mut self, language: Option<String>) -> &mut PodcastBuilder {
        if let Some(language) = language {
            self.language = language;
        }
        self
    }

    pub fn keywords(&mut self, keywords: Vec<ITunesCategory>) -> &mut PodcastBuilder {
        self.keywords = keywords
            .iter()
            .map(|x| x.text.clone())
            .collect::<Vec<String>>()
            .join(",");
        self
    }

    pub fn guid(&mut self, guid: Option<String>) -> &mut PodcastBuilder {
        if let Some(guid) = guid {
            self.guid = Some(guid);
        }
        self
    }

    pub fn last_build_date(&mut self, last_build_date: Option<String>) -> &mut PodcastBuilder {
        if let Some(last_build_date) = last_build_date {
            self.last_build_date = last_build_date;
        }
        self
    }

    pub fn explicit(&mut self, explicit: Option<String>) -> &mut PodcastBuilder {
        if let Some(explicit) = explicit {
            self.explicit = explicit == "yes";
        }
        self
    }

    pub fn author(&mut self, author: Option<String>) -> &mut PodcastBuilder {
        if let Some(author) = author {
            self.author = author;
        }

        self
    }

    pub fn build(&self) -> PodcastExtra {
        PodcastExtra {
            id: self.id,
            explicit: self.explicit,
            description: self.description.clone(),
            language: self.language.clone(),
            keywords: self.keywords.clone(),
            last_build_date: self.last_build_date.clone(),
            author: self.author.clone(),
            guid: self.guid.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::podcast_builder::PodcastBuilder;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_raw_podcast_builder() {
        let podcast_builder = PodcastBuilder::new(1).build();

        assert_eq!(podcast_builder.id, 1);
        assert_eq!(podcast_builder.description, "");
        assert_eq!(podcast_builder.language, "");
        assert_eq!(podcast_builder.keywords, "");
        assert_eq!(podcast_builder.last_build_date, "");
        assert!(!podcast_builder.explicit);
        assert_eq!(podcast_builder.author, "");
    }

    #[test]
    #[serial]
    fn test_only_description() {
        let podcast_builder = PodcastBuilder::new(2)
            .description("test".to_string())
            .build();

        assert_eq!(podcast_builder.id, 2);
        assert_eq!(podcast_builder.description, "test");
        assert_eq!(podcast_builder.language, "");
        assert_eq!(podcast_builder.keywords, "");
        assert_eq!(podcast_builder.last_build_date, "");
        assert!(!podcast_builder.explicit);
        assert_eq!(podcast_builder.author, "");
    }

    #[test]
    #[serial]
    fn test_only_language() {
        let podcast_builder = PodcastBuilder::new(3)
            .language(Some("en".to_string()))
            .build();

        assert_eq!(podcast_builder.id, 3);
        assert_eq!(podcast_builder.description, "");
        assert_eq!(podcast_builder.language, "en");
        assert_eq!(podcast_builder.keywords, "");
        assert_eq!(podcast_builder.last_build_date, "");
        assert!(!podcast_builder.explicit);
        assert_eq!(podcast_builder.author, "");
    }

    #[test]
    #[serial]
    fn test_only_keywords() {
        let keywords = vec![
            rss::extension::itunes::ITunesCategory {
                text: "test".to_string(),
                subcategory: None,
            },
            rss::extension::itunes::ITunesCategory {
                text: "test2".to_string(),
                subcategory: None,
            },
        ];
        let podcast_builder = PodcastBuilder::new(4).keywords(keywords).build();

        assert_eq!(podcast_builder.id, 4);
        assert_eq!(podcast_builder.description, "");
        assert_eq!(podcast_builder.language, "");
        assert_eq!(podcast_builder.keywords, "test,test2");
        assert_eq!(podcast_builder.last_build_date, "");
        assert!(!podcast_builder.explicit);
        assert_eq!(podcast_builder.author, "");
    }

    #[test]
    #[serial]
    fn test_only_last_build_date() {
        let podcast_builder = PodcastBuilder::new(5)
            .last_build_date(Some("22.03.2023".to_string()))
            .build();

        assert_eq!(podcast_builder.id, 5);
        assert_eq!(podcast_builder.description, "");
        assert_eq!(podcast_builder.language, "");
        assert_eq!(podcast_builder.keywords, "");
        assert_eq!(podcast_builder.last_build_date, "22.03.2023");
        assert!(!podcast_builder.explicit);
        assert_eq!(podcast_builder.author, "");
    }

    #[test]
    #[serial]
    fn test_only_explicit() {
        let podcast_builder = PodcastBuilder::new(6)
            .explicit(Some("yes".to_string()))
            .build();

        assert_eq!(podcast_builder.id, 6);
        assert_eq!(podcast_builder.description, "");
        assert_eq!(podcast_builder.language, "");
        assert_eq!(podcast_builder.keywords, "");
        assert_eq!(podcast_builder.last_build_date, "");
        assert!(podcast_builder.explicit);
        assert_eq!(podcast_builder.author, "");
    }

    #[test]
    #[serial]
    fn test_only_author() {
        let podcast_builder = PodcastBuilder::new(6)
            .author(Some("yes".to_string()))
            .build();

        assert_eq!(podcast_builder.id, 6);
        assert_eq!(podcast_builder.description, "");
        assert_eq!(podcast_builder.language, "");
        assert_eq!(podcast_builder.keywords, "");
        assert_eq!(podcast_builder.last_build_date, "");
        assert!(!podcast_builder.explicit);
        assert_eq!(podcast_builder.author, "yes");
    }

    #[test]
    #[serial]
    fn test_combined() {
        let podcast_builder = PodcastBuilder::new(7)
            .description("test".to_string())
            .language(Some("en".to_string()))
            .keywords(vec![
                rss::extension::itunes::ITunesCategory {
                    text: "test".to_string(),
                    subcategory: None,
                },
                rss::extension::itunes::ITunesCategory {
                    text: "test2".to_string(),
                    subcategory: None,
                },
            ])
            .last_build_date(Some("22.03.2023".to_string()))
            .explicit(Some("yes".to_string()))
            .author(Some("yes".to_string()))
            .build();

        assert_eq!(podcast_builder.id, 7);
        assert_eq!(podcast_builder.description, "test");
        assert_eq!(podcast_builder.language, "en");
        assert_eq!(podcast_builder.keywords, "test,test2");
        assert_eq!(podcast_builder.last_build_date, "22.03.2023");
        assert!(podcast_builder.explicit);
        assert_eq!(podcast_builder.author, "yes");
    }
}
