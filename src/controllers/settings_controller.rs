use std::str::FromStr;
use crate::models::settings::Setting;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use actix_web::web::{Data, Path};
use actix_web::{get, put};
use actix_web::{web, HttpResponse, Responder};
use std::sync::{Mutex, MutexGuard};
use chrono::Local;
use xml_builder::{XMLBuilder, XMLElement, XMLVersion};
use crate::db::DB;
use crate::DbPool;
use crate::models::itunes_models::{Podcast};
use crate::models::user::User;
use crate::mutex::LockResultExt;
use crate::service::environment_service::EnvironmentService;
use crate::service::settings_service::SettingsService;

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets the current settings")),
tag="podcast_episodes"
)]
#[get("/settings")]
pub async fn get_settings(db: Data<Mutex<SettingsService>>) -> impl Responder {
    let mut db = db.lock().ignore_poison();

    let settings = db.get_settings();
    match settings {
        Some(settings) => HttpResponse::Ok().json(settings),
        None => HttpResponse::NotFound().finish(),
    }
}

#[utoipa::path(
context_path="/api/v1",
request_body=Setting,
responses(
(status = 200, description = "Updates the current settings")),
tag="settings"
)]
#[put("/settings")]
pub async fn update_settings(db: Data<Mutex<SettingsService>>, settings: web::Json<Setting>,
                             requester: Option<web::ReqData<User>>) -> impl Responder {
    if !requester.unwrap().is_admin() {
        return HttpResponse::Unauthorized().finish();
    }

    let mut db = db.lock().ignore_poison();

    let settings = db.update_settings(settings.into_inner());
    HttpResponse::Ok().json(settings)
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Runs a cleanup of old episodes")),
tag="settings"
)]
#[put("/settings/runcleanup")]
pub async fn run_cleanup(
    pdservice: Data<Mutex<PodcastEpisodeService>>,
    db: Data<Mutex<SettingsService>>,
    conn: Data<DbPool>,
    requester: Option<web::ReqData<User>>
) -> impl Responder {

    if !requester.unwrap().is_admin() {
        return HttpResponse::Unauthorized().finish();
    }

    let settings = db.lock().ignore_poison().get_settings();
    match settings {
        Some(settings) => {
            pdservice
                .lock()
                .ignore_poison()
                .cleanup_old_episodes(settings.auto_cleanup_days,&mut conn.get().unwrap());
            HttpResponse::Ok().finish()
        }
        None => {
            log::error!("Error getting settings");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode{
    LOCAL,ONLINE
}

#[get("/settings/opml/{type_of}")]
pub async fn get_opml(conn: Data<DbPool>, type_of: Path<Mode>, env_service: Data<Mutex<EnvironmentService>>) ->
                                                                                             impl
Responder {
    let env_service = env_service.lock().ignore_poison();
    let podcasts_found = DB::get_all_podcasts(&mut conn.get().unwrap()).unwrap();

    let mut xml = XMLBuilder::new().version(XMLVersion::XML1_1)
        .encoding("UTF-8".to_string())
        .build();
    let mut opml = XMLElement::new("opml");
    opml.add_attribute("version", "2.0");
    opml.add_child(add_header()).expect("TODO: panic message");
    opml.add_child(add_podcasts(podcasts_found, env_service,type_of.into_inner() )).expect("TODO: panic \
    message");

    xml.set_root_element(opml);


    let mut writer: Vec<u8> = Vec::new();
    xml.generate(&mut writer).unwrap();
    HttpResponse::Ok().body(writer)
}


fn add_header()->XMLElement {
    let mut head = XMLElement::new("head");
    let mut title = XMLElement::new("title");
    title.add_text("PodFetch Feed Export".to_string()).expect("Error creating title");
    head.add_child(title).expect("TODO: panic message");
    let mut date_created = XMLElement::new("dateCreated");
    date_created.add_text(Local::now().to_rfc3339()).expect("Error creating dateCreated");

    head.add_child(date_created).expect("TODO: panic message");
    head
}


fn add_body()->XMLElement {
    XMLElement::new("body")
}


fn add_podcasts(podcasts_found: Vec<Podcast>, env_service: MutexGuard<EnvironmentService>,
                type_of: Mode) -> XMLElement {
    let mut body = add_body();
    for podcast in podcasts_found {
        let mut outline = XMLElement::new("outline");
        if podcast.summary.is_some(){
            outline.add_attribute("text", &*podcast.summary.unwrap());
        }
        outline.add_attribute("title", &*podcast.name);
        outline.add_attribute("type", "rss");
        match type_of {
            Mode::LOCAL => outline.add_attribute("xmlUrl", &*format!("{}rss/{}", &*env_service
                .get_server_url(), podcast.id)),
            Mode::ONLINE => outline.add_attribute("xmlUrl", &*podcast.rssfeed),
        }
        body.add_child(outline).expect("TODO: panic message");
    }
    body
}


#[put("/settings/name")]
pub async fn update_name(db: Data<Mutex<SettingsService>>, update_information: web::Json<UpdateNameSettings>,
                             requester: Option<web::ReqData<User>>) -> impl
Responder {
    if !requester.unwrap().is_admin() {
        return HttpResponse::Unauthorized().finish();
    }

    let mut db = db.lock().ignore_poison();

    let settings = db.update_name(update_information.into_inner());
    HttpResponse::Ok().json(settings)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNameSettings{
    pub use_existing_filenames: bool,
    pub replace_invalid_characters: bool,
    pub replacement_strategy: ReplacementStrategy
}
#[derive(Deserialize)]
pub enum ReplacementStrategy{
    ReplaceWithUnderscoreAndDash,
    REMOVE,
    ReplaceWithDash
}

impl ToString for ReplacementStrategy{
    fn to_string(&self) -> String {
        match self {
            ReplacementStrategy::ReplaceWithUnderscoreAndDash => "replace-with-dash-and-underscore".to_string(),
            ReplacementStrategy::REMOVE => "remove".to_string(),
            ReplacementStrategy::ReplaceWithDash => "replace-with-dash".to_string()
        }
    }
}

impl FromStr for ReplacementStrategy{
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "replace-with-dash-and-underscore" => Ok(ReplacementStrategy::ReplaceWithUnderscoreAndDash),
            "remove" => Ok(ReplacementStrategy::REMOVE),
            "replace-with-dash" => Ok(ReplacementStrategy::ReplaceWithDash),
            _ => Err(())
        }
    }
}