use crate::controllers::watch_together_dto::{WatchTogetherDto, WatchTogetherDtoCreate};
use crate::dbconfig::DBType;
use crate::models::user::User;
use crate::models::watch_together_users::WatchTogetherUser;
use crate::models::watch_togethers::WatchTogether;
use crate::utils::error::CustomError;

pub struct WatchTogetherService {}

impl WatchTogetherService {
    pub fn create_watch_together(
        watch_together_create: &WatchTogetherDtoCreate,
        conn: &mut DBType,
        unwrapped_requester: &User,
    ) -> Result<WatchTogetherDto, CustomError> {
        let mut random_room_id = WatchTogether::random_room_id();
        // Check if the room id is already in use
        while WatchTogether::get_watch_together_by_id(&random_room_id.clone(), conn)?.is_some() {
            random_room_id = WatchTogether::random_room_id();
        }

        let watch_together = WatchTogether::new(
            None,
            &random_room_id,
            unwrapped_requester.username.clone(),
            watch_together_create.room_name.clone(),
        );
        let saved_watch_together = watch_together.save_watch_together(conn)?;
        let watch_together_user = WatchTogetherUser::new(
            None,
            saved_watch_together.id.unwrap(),
            unwrapped_requester.username.clone(),
            "admin".to_string(),
            Some(unwrapped_requester.username.clone()),
        );
        watch_together_user.save_watch_together_users(conn)?;
        Ok(saved_watch_together.into())
    }
}
