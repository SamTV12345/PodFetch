use rss::extension::itunes::ITunesCategory;

pub struct PodcastBuilder {
    id: i32,
    description: String,
    language: String,
    keywords: String,
    last_build_date: String,
    explicit: bool,
    author: String,
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
}

impl PodcastBuilder {
    pub fn new(podcast_id: i32) -> PodcastBuilder {
        return PodcastBuilder {
            id: podcast_id,
            description: "".to_string(),
            language: "".to_string(),
            keywords: "".to_string(),
            last_build_date: "".to_string(),
            explicit: false,
            author: "".to_string(),
        };
    }

    pub fn description(&mut self, description: String) -> &mut PodcastBuilder {
        self.description = description;
        return self;
    }

    pub fn language(&mut self, language: Option<String>) -> &mut PodcastBuilder {
        if language.is_some() {
            self.language = language.unwrap();
        }
        return self;
    }

    pub fn keywords(&mut self, keywords: Vec<ITunesCategory>) -> &mut PodcastBuilder {
        self.keywords = keywords
            .iter()
            .map(|x| x.text.clone())
            .collect::<Vec<String>>()
            .join(",");
        return self;
    }

    pub fn last_build_date(&mut self, last_build_date: Option<String>) -> &mut PodcastBuilder {
        if last_build_date.is_some() {
            self.last_build_date = last_build_date.unwrap();
        }
        return self;
    }

    pub fn explicit(&mut self, explicit: Option<String>) -> &mut PodcastBuilder {
        match explicit {
            Some(explicit) => {
                self.explicit = explicit == "yes";
            }
            None => {}
        }
        return self;
    }

    pub fn author(&mut self, author: Option<String>) -> &mut PodcastBuilder {
        match author {
            Some(author) => {
                self.author = author;
            }
            None => {}
        }
        return self;
    }

    pub fn build(&self) -> PodcastExtra {
        return PodcastExtra {
            id: self.id.clone(),
            explicit: self.explicit.clone(),
            description: self.description.clone(),
            language: self.language.clone(),
            keywords: self.keywords.clone(),
            last_build_date: self.last_build_date.clone(),
            author: self.author.clone(),
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::podcast_builder::PodcastBuilder;

    #[test]
    fn test_raw_podcast_builder(){
        let podcast_builder = PodcastBuilder::new(1).build();

        assert_eq!(podcast_builder.id,1);
        assert_eq!(podcast_builder.description,"");
        assert_eq!(podcast_builder.language,"");
        assert_eq!(podcast_builder.keywords,"");
        assert_eq!(podcast_builder.last_build_date,"");
        assert_eq!(podcast_builder.explicit,false);
        assert_eq!(podcast_builder.author,"");
    }

    #[test]
    fn test_only_description(){
        let podcast_builder = PodcastBuilder::new(2).description("test".to_string()).build();

        assert_eq!(podcast_builder.id,2);
        assert_eq!(podcast_builder.description,"test");
        assert_eq!(podcast_builder.language,"");
        assert_eq!(podcast_builder.keywords,"");
        assert_eq!(podcast_builder.last_build_date,"");
        assert_eq!(podcast_builder.explicit,false);
        assert_eq!(podcast_builder.author,"");
    }

    #[test]
    fn test_only_language(){
        let podcast_builder = PodcastBuilder::new(3).language(Some("en".to_string())).build();

        assert_eq!(podcast_builder.id,3);
        assert_eq!(podcast_builder.description,"");
        assert_eq!(podcast_builder.language,"en");
        assert_eq!(podcast_builder.keywords,"");
        assert_eq!(podcast_builder.last_build_date,"");
        assert_eq!(podcast_builder.explicit,false);
        assert_eq!(podcast_builder.author,"");
    }

    #[test]
    fn test_only_keywords(){
        let keywords = vec![rss::extension::itunes::ITunesCategory{
            text: "test".to_string(),
            subcategory: None,
        },
        rss::extension::itunes::ITunesCategory{
            text: "test2".to_string(),
            subcategory: None,
        }];
        let podcast_builder = PodcastBuilder::new(4).keywords(keywords).build();

        assert_eq!(podcast_builder.id,4);
        assert_eq!(podcast_builder.description,"");
        assert_eq!(podcast_builder.language,"");
        assert_eq!(podcast_builder.keywords,"test,test2");
        assert_eq!(podcast_builder.last_build_date,"");
        assert_eq!(podcast_builder.explicit,false);
        assert_eq!(podcast_builder.author,"");
    }

    #[test]
    fn test_only_last_build_date(){
        let podcast_builder = PodcastBuilder::new(5).last_build_date(Some("22.03.2023".to_string()))
            .build();

        assert_eq!(podcast_builder.id,5);
        assert_eq!(podcast_builder.description,"");
        assert_eq!(podcast_builder.language,"");
        assert_eq!(podcast_builder.keywords,"");
        assert_eq!(podcast_builder.last_build_date,"22.03.2023");
        assert_eq!(podcast_builder.explicit,false);
        assert_eq!(podcast_builder.author,"");
    }

    #[test]
    fn test_only_explicit(){
        let podcast_builder = PodcastBuilder::new(6).explicit(Some("yes".to_string())).build();

        assert_eq!(podcast_builder.id,6);
        assert_eq!(podcast_builder.description,"");
        assert_eq!(podcast_builder.language,"");
        assert_eq!(podcast_builder.keywords,"");
        assert_eq!(podcast_builder.last_build_date,"");
        assert_eq!(podcast_builder.explicit,true);
        assert_eq!(podcast_builder.author,"");
    }

    #[test]
    fn test_only_author(){
        let podcast_builder = PodcastBuilder::new(6).author(Some("yes".to_string())).build();

        assert_eq!(podcast_builder.id,6);
        assert_eq!(podcast_builder.description,"");
        assert_eq!(podcast_builder.language,"");
        assert_eq!(podcast_builder.keywords,"");
        assert_eq!(podcast_builder.last_build_date,"");
        assert_eq!(podcast_builder.explicit,false);
        assert_eq!(podcast_builder.author,"yes");
    }

    #[test]
    fn test_combined(){
        let podcast_builder = PodcastBuilder::new(7).description("test".to_string())
            .language(Some("en".to_string()))
            .keywords(vec![rss::extension::itunes::ITunesCategory{
                text: "test".to_string(),
                subcategory: None,
            },
            rss::extension::itunes::ITunesCategory{
                text: "test2".to_string(),
                subcategory: None,
            }])
            .last_build_date(Some("22.03.2023".to_string()))
            .explicit(Some("yes".to_string()))
            .author(Some("yes".to_string()))
            .build();

        assert_eq!(podcast_builder.id,7);
        assert_eq!(podcast_builder.description,"test");
        assert_eq!(podcast_builder.language,"en");
        assert_eq!(podcast_builder.keywords,"test,test2");
        assert_eq!(podcast_builder.last_build_date,"22.03.2023");
        assert_eq!(podcast_builder.explicit,true);
        assert_eq!(podcast_builder.author,"yes");
    }
}
