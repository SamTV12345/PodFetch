use crate::constants::inner_constants::WATCH_TOGETHER_ID;
use crate::controllers::watch_together_dto::{
    WatchTogetherDto, WatchTogetherDtoCreate, WatchTogetherDtoDelete,
};
use crate::models::user::User;
use crate::models::watch_together_users::WatchTogetherUser;
use crate::models::watch_togethers::WatchTogether;
use crate::service::WatchTogetherService;
use crate::utils::error::{map_r2d2_error, CustomError};
use crate::DbPool;
use actix::ActorStreamExt;
use actix_web::cookie::Cookie;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, Responder, Scope};
use std::ops::DerefMut;

#[get("/{watch_id}")]
pub async fn get_watch_together(
    watch_id: Path<String>,
    conn: Data<DbPool>,
) -> Result<HttpResponse, CustomError> {
    let watch_together: Option<WatchTogetherDto> = WatchTogether::get_watch_together_by_id(
        watch_id.into_inner(),
        conn.get().map_err(map_r2d2_error)?.deref_mut(),
    )
    .map(|watch_together| watch_together.map(|watch_together| watch_together.into()))
    .map_err(|e| CustomError::Unknown)?;
    Ok(HttpResponse::Ok().json(watch_together))
}

#[get("")]
pub async fn get_available_watch_togethers(
    requester: Option<web::ReqData<User>>,
    conn: Data<DbPool>,
) -> Result<HttpResponse, CustomError> {
    WatchTogether::get_watch_together_by_admin(
        requester.unwrap().username.clone(),
        conn.get().map_err(map_r2d2_error)?.deref_mut(),
    )
    .map(|watch_together| {
        watch_together
            .into_iter()
            .map(|watch_together| Into::<WatchTogetherDto>::into(watch_together))
            .collect()
    })
    .map_err(|e| CustomError::Unknown)
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
        let watch_together = WatchTogetherUser::get_watch_together_by_username(
            &unwrapped_requester.username,
            conn.get().map_err(map_r2d2_error)?.deref_mut(),
        )?;
        // Check if this user uses a new device which does not have the cookie
        match watch_together {
            Some(w) => {
                let watch_together = WatchTogetherService::create_watch_together(
                    &data.into_inner(),
                    conn.get().map_err(map_r2d2_error)?.deref_mut(),
                    &unwrapped_requester,
                )?;

                return Ok(HttpResponse::Ok()
                    .cookie(
                        Cookie::build(WATCH_TOGETHER_ID, w.user)
                            .http_only(true)
                            .finish(),
                    )
                    .json(watch_together));
            }
            None => {
                let watch_together = WatchTogetherService::create_watch_together(
                    &data.into_inner(),
                    conn.get().map_err(map_r2d2_error)?.deref_mut(),
                    &unwrapped_requester,
                )?;

                cookie_to_send = Some(
                    Cookie::build(WATCH_TOGETHER_ID, unwrapped_requester.username.clone())
                        .http_only(true)
                        .finish(),
                );
                Ok(HttpResponse::Ok()
                    .cookie(cookie_to_send.unwrap())
                    .json(watch_together))
            }
        }
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

#[delete("")]
pub async fn delete_watch_together(
    data: Json<WatchTogetherDtoDelete>,
    requester: Option<web::ReqData<User>>,
    conn: Data<DbPool>,
) -> Result<HttpResponse, CustomError> {
    if data.room_id.is_empty() {
        return Ok(HttpResponse::BadRequest().finish());
    }

    let unwrapped_requester = requester.unwrap();
    let watch_together = WatchTogetherUser::get_watch_together_by_username(
        &unwrapped_requester.username,
        conn.get().map_err(map_r2d2_error)?.deref_mut(),
    )?;

    if watch_together.is_none() {
        return Ok(HttpResponse::BadRequest().finish());
    }

    let watch_together = watch_together.unwrap();
    if watch_together.user != unwrapped_requester.username {
        return Ok(HttpResponse::BadRequest().finish());
    }

    WatchTogether::delete_watch_together(
        unwrapped_requester.id,
        data.room_id.clone(),
        conn.get().map_err(map_r2d2_error)?.deref_mut(),
    )?;

    Ok(HttpResponse::Ok().finish())
}

pub fn watch_together_routes() -> Scope {
    Scope::new("/watch-together")
        .service(get_watch_together)
        .service(create_watch_together)
        .service(get_available_watch_togethers)
        .service(delete_watch_together)
}
