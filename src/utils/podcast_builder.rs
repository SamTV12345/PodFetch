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
