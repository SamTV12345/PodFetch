use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::utils::podcast_key_checker::check_permissions_for_files;
use actix_files::Files;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::middleware::from_fn;
use actix_web::{web, Scope};

pub fn get_podcast_serving() -> Scope<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse<impl MessageBody>,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    web::scope("/podcasts")
        .wrap(from_fn(check_permissions_for_files))
        .service(
            Files::new("/", ENVIRONMENT_SERVICE.default_podfetch_folder.to_string())
                .disable_content_disposition(),
        )
}
