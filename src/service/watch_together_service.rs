use crate::controllers::watch_together_dto::{WatchTogetherDto, WatchTogetherDtoCreate};
use crate::dbconfig::DBType;
use crate::models::user::User;
use crate::models::watch_together_users::WatchTogetherUser;
use crate::models::watch_together_users_to_room_mappings::{WatchTogetherRole, WatchTogetherStatus, WatchTogetherUsersToRoomMapping};
use crate::models::watch_togethers::WatchTogether;
use crate::utils::error::CustomError;

pub struct WatchTogetherService {}

impl WatchTogetherService {
    pub fn create_watch_together(
        watch_together_create: &WatchTogetherDtoCreate,
        conn: &mut DBType,
        unwrapped_requester: &User,
    ) -> Result<(WatchTogetherDto, WatchTogetherUser), CustomError> {
        let mut random_room_id = WatchTogether::random_room_id();
        // Check if the room id is already in use
        while WatchTogether::get_watch_together_by_id(&random_room_id.clone(), conn)?.is_some() {
            random_room_id = WatchTogether::random_room_id();
        }

        let watch_together = WatchTogether::new(
            None,
            &random_room_id,
            watch_together_create.room_name.clone(),
        );
        let saved_watch_together = watch_together.save_watch_together(conn)?;

        let mut possible_found_user = WatchTogetherUser::get_watch_together_users_by_user_id
        (unwrapped_requester.id, conn)?;


        if possible_found_user.is_none() {
            let watch_together_user = WatchTogetherUser::new(
                uuid::Uuid::new_v4().to_string(),
                Some(unwrapped_requester.username.clone()),
                Some(unwrapped_requester.id),
            );
            watch_together_user.save_watch_together_users(conn)?;

            possible_found_user = Some(watch_together_user);
        }

        // unwrap: Safe as we have just checked if the user exists and create it if it does not
        let unwrapped_user = possible_found_user.clone().unwrap();

        let watch_together_mapping = WatchTogetherUsersToRoomMapping::new(
            saved_watch_together.id.unwrap(),
            &unwrapped_user.subject,
            WatchTogetherStatus::Accepted,
            WatchTogetherRole::Admin,
        );

        watch_together_mapping.insert_watch_together_user_to_room_mapping(conn)?;

        Ok((saved_watch_together.into(), unwrapped_user))
    }
}
