use crate::constants::inner_constants::WATCH_TOGETHER_ID;
use crate::controllers::watch_together_dto::{WatchTogetherDto, WatchTogetherDtoCreate, WatchTogetherDtoDelete};
use crate::models::user::User;
use crate::models::watch_together_users::WatchTogetherUser;
use crate::models::watch_togethers::WatchTogether;
use crate::service::WatchTogetherService;
use crate::utils::error::{map_r2d2_error, CustomError};
use crate::utils::jwt_watch_together::generate_watch_together_id;
use crate::DbPool;
use actix_web::cookie::Cookie;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, Scope};
use std::ops::DerefMut;
use crate::models::watch_together_users_to_room_mappings::{WatchTogetherStatus, WatchTogetherUsersToRoomMapping};

#[get("/{watch_id}")]
pub async fn get_watch_together(
    watch_id: Path<String>,
    conn: Data<DbPool>,
) -> Result<HttpResponse, CustomError> {
    let watch_together: Option<WatchTogetherDto> = WatchTogether::get_watch_together_by_id(
        &watch_id.into_inner(),
        conn.get().map_err(map_r2d2_error)?.deref_mut(),
    )
    .map(|watch_together| watch_together.map(|watch_together| watch_together.into()))
    .map_err(|_| CustomError::Unknown)?;
    Ok(HttpResponse::Ok().json(watch_together))
}

#[get("")]
pub async fn get_available_watch_togethers(
    requester: Option<web::ReqData<User>>,
    conn: Data<DbPool>,
) -> Result<HttpResponse, CustomError> {
    WatchTogetherUsersToRoomMapping::get_watch_together_by_admin(
        requester.unwrap().id,
        conn.get().map_err(map_r2d2_error)?.deref_mut(),
    )
    .map(|watch_together| {
        watch_together
            .into_iter()
            .map(Into::<WatchTogetherDto>::into)
            .collect()
    })
    .map_err(|_| CustomError::Unknown)
    .map(|watch_together: Vec<WatchTogetherDto>| HttpResponse::Ok().json(watch_together))
}

#[post("")]
pub async fn create_watch_together(
    data: Json<WatchTogetherDtoCreate>,
    req: HttpRequest,
    requester: Option<web::ReqData<User>>,
    conn: Data<DbPool>,
) -> Result<HttpResponse, CustomError> {
    let cookie = req.cookie(WATCH_TOGETHER_ID);
    let unwrapped_requester = requester.unwrap();
    let cookie_to_send: Option<Cookie>;

    // If the user has never created a watch together room
    if cookie.is_none() {
        // Check if this user uses a new device which does not have the cookie
        let watch_together = WatchTogetherService::create_watch_together(
            &data.into_inner(),
            conn.get().map_err(map_r2d2_error)?.deref_mut(),
            &unwrapped_requester,
        )?;

        let id = generate_watch_together_id(
            Some(unwrapped_requester.username.clone()),
            None,
            conn.get().map_err(map_r2d2_error)?.deref_mut(),
        );
        cookie_to_send = Some(
            Cookie::build(WATCH_TOGETHER_ID, id)
                .http_only(true)
                .finish(),
        );
        Ok(HttpResponse::Ok()
            .cookie(cookie_to_send.unwrap())
            .json(watch_together))
    } else {
        // Cookie is already present for this user
        let watch_together = WatchTogetherService::create_watch_together(
            &data.into_inner(),
            conn.get().map_err(map_r2d2_error)?.deref_mut(),
            &unwrapped_requester,
        )?;

        Ok(HttpResponse::Ok().json(watch_together))
    }
}

#[delete("/{room_id}")]
pub async fn delete_watch_together(
    data: Path<String>,
    requester: Option<web::ReqData<User>>,
    conn: Data<DbPool>,
) -> Result<HttpResponse, CustomError> {
    if data.is_empty() {
        return Ok(HttpResponse::BadRequest().finish());
    }

    let unwrapped_requester = requester.unwrap();

    let opt_watch_together_user = WatchTogetherUser::get_watch_together_users_by_user_id(
        unwrapped_requester.id,
        conn.get().map_err(map_r2d2_error)?.deref_mut(),
    )?;

    if opt_watch_together_user.is_none() {
        return Ok(HttpResponse::BadRequest().finish());
    }

    let watch_together_users = opt_watch_together_user.unwrap();

    let room_id_to_delete = data.into_inner();

    let watch_together = WatchTogetherUsersToRoomMapping::get_by_user_and_room_id(
        &watch_together_users.subject,
        &room_id_to_delete,
        conn.get().map_err(map_r2d2_error)?.deref_mut(),
    )?;

    if watch_together.is_none() {
        return Ok(HttpResponse::BadRequest().finish());
    }


    let watch_together_room_mapping = watch_together.unwrap();
    if watch_together_room_mapping.role == WatchTogetherStatus::Admin.to_string() {
        return Ok(HttpResponse::BadRequest().finish());
    }

    WatchTogether::delete_watch_together(
        &room_id_to_delete,
        conn.get().map_err(map_r2d2_error)?.deref_mut(),
    )?;

    Ok(HttpResponse::Ok().json(WatchTogetherDtoDelete::new(room_id_to_delete)))
}

pub fn watch_together_routes() -> Scope {
    Scope::new("/watch-together")
        .service(get_watch_together)
        .service(create_watch_together)
        .service(get_available_watch_togethers)
        .service(delete_watch_together)
}
