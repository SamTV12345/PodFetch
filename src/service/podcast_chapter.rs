use chrono::Duration;

pub type Image = url::Url;

/// Represents a web link for the [chapter](crate::Chapter).
#[derive(Debug, PartialEq, Clone)]
pub struct Link {
    /// The URL of the link.
    pub url: url::Url,
    /// The title of the link.
    pub title: Option<String>,
}

#[derive(Debug, PartialEq, Default)]
pub struct Chapter {
    /// The starting time of the chapter.
    pub start: Duration,
    /// The end time of the chapter.
    pub end: Option<Duration>,
    /// The title of this chapter.
    pub title: Option<String>,
    /// The image to use as chapter art.
    pub image: Option<Image>,
    /// Web page or supporting document related to the topic of this chapter.
    pub link: Option<Link>,
    /// If this property is set to true, this chapter should not display visibly to the user in either the table of contents or as a jump-to point in the user interface. In the original spec, the inverse of this is called `toc`.
    pub hidden: bool,
}
