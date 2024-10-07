use std::ops::DerefMut;
use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, Responder, Scope};
use actix_web::cookie::Cookie;
use actix_web::web::{Data, Path, Json};
use crate::constants::inner_constants::WATCH_TOGETHER_ID;
use crate::controllers::watch_together_dto::{WatchTogetherDto, WatchTogetherDtoCreate};
use crate::DbPool;
use crate::models::user::User;
use crate::models::watch_together_users::WatchTogetherUser;
use crate::models::watch_togethers::WatchTogether;
use crate::utils::error::{map_r2d2_error, CustomError};

//#[get("/{watch_id}")]
pub async fn get_watch_together(watch_id: Path<String>, conn: Data<DbPool>) ->
                                                                 Result<Option<WatchTogetherDto>,
    CustomError> {
    let watch_together: Option<WatchTogetherDto> = WatchTogether::get_watch_together_by_id(watch_id.into_inner
    (),
                                                                          conn.get()
                                                            .map_err
    (map_r2d2_error)?.deref_mut())
        .map(|watch_together| watch_together.map(|watch_together| watch_together.into()))
        .map_err(|e| CustomError::Unknown)?;
   Ok(watch_together)
}

//#[post("/")]
pub async fn create_watch_together(data: Json<WatchTogetherDtoCreate>, req: HttpRequest,
                                   requester: Option<web::ReqData<User>>, conn: Data<DbPool>) ->  impl Responder{
    let cookie = req.cookie(WATCH_TOGETHER_ID);
    let unwrapped_requester = requester.unwrap();
    let cookie_to_send: Option<Cookie>;

    // If the user has never created a watch together room
    if cookie.is_none() {
        let watch_together = WatchTogetherUser::get_watch_together_by_username(&unwrapped_requester
                                                                       .username,conn.get()
            .map_err
            (map_r2d2_error).unwrap().deref_mut()).unwrap();
        return match watch_together {
            Some(w)=>{
                return HttpResponse::Ok()
                    .cookie(Cookie::build(WATCH_TOGETHER_ID, w.user)
                    .http_only(true)
                        .finish())
                    .finish()
            }
            None=>{
                let mut random_room_id = WatchTogether::random_room_id();
                // Check if the room id is already in use
                while WatchTogether::get_watch_together_by_id(random_room_id.clone(), conn.get()
                    .map_err(map_r2d2_error).unwrap().deref_mut()).unwrap().is_some() {
                    random_room_id = WatchTogether::random_room_id();
                }


                let watch_together = WatchTogether::new(0, &random_room_id, unwrapped_requester
                    .username.clone(), data.room_name.clone());
                watch_together.save_watch_together(conn.get().map_err(map_r2d2_error)?.deref_mut())?;
                let watch_together_user = WatchTogetherUser::new(0, random_room_id,
                                                                 unwrapped_requester.username.clone(), "admin".to_string(), None);
                watch_together_user.save_watch_together_users(conn.get().map_err(map_r2d2_error)?.deref_mut())?;
                cookie_to_send = Some(Cookie::build(WATCH_TOGETHER_ID, unwrapped_requester.username.clone())
                    .http_only(true)
                    .finish());
                HttpResponse::Ok()
                    .cookie(cookie_to_send.unwrap())
                    .finish()
            }
        }
    }

    Ok(())
}

#[delete("/")]
pub async fn delete_watch_together() -> Result<(), CustomError> {
    Ok(())
}

pub fn watch_together_routes() -> Scope {
    Scope::new("/watch-together")
        .service(get_watch_together)
        .service(create_watch_together)
}