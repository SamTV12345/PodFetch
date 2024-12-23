use crate::adapters::persistence::repositories::tag::tag::TagRepositoryImpl;
use crate::domain::models::tag::tag::Tag;
use crate::utils::error::CustomError;

pub struct TagService;

impl TagService {
    pub(crate) fn get_tags_of_podcast(p0: i32, p1: &String) -> Result<Vec<Tag>, CustomError> {
        TagRepositoryImpl::get_tags_of_podcast(p0, p1)
    }
}

impl TagService {
    pub fn delete_tags_by_podcast_id(podcast_id_to_delete: i32) -> Result<(), CustomError> {
        TagRepositoryImpl::get_tags
    }
}