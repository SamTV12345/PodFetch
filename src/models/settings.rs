use crate::constants::inner_constants::DEFAULT_SETTINGS;
use crate::dbconfig::schema::*;
use crate::service::environment_service::OidcConfig;
use crate::utils::do_retry::do_retry;
use crate::utils::error::{map_db_error, CustomError};
use crate::DBType as DbConnection;
use diesel::insert_into;
use diesel::prelude::{AsChangeset, Identifiable, Insertable, Queryable};
use diesel::{OptionalExtension, RunQueryDsl};
use jwt_simple::algorithms::HS256Key;
use jwt_simple::prelude::RS256KeyPair;
use utoipa::ToSchema;

#[derive(
    Serialize,
    Deserialize,
    Queryable,
    Insertable,
    Debug,
    Clone,
    Identifiable,
    AsChangeset,
    ToSchema,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct Setting {
    pub id: i32,
    pub auto_download: bool,
    pub auto_update: bool,
    pub auto_cleanup: bool,
    pub auto_cleanup_days: i32,
    pub podcast_prefill: i32,
    pub replace_invalid_characters: bool,
    pub use_existing_filename: bool,
    pub replacement_strategy: String,
    pub episode_format: String,
    pub podcast_format: String,
    pub direct_paths: bool,
    pub jwt_key: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigModel {
    pub podindex_configured: bool,
    pub rss_feed: String,
    pub server_url: String,
    pub basic_auth: bool,
    pub oidc_configured: bool,
    pub oidc_config: Option<OidcConfig>,
    pub reverse_proxy: bool,
}

impl Setting {
    pub fn get_settings(conn: &mut DbConnection) -> Result<Option<Setting>, CustomError> {
        use crate::dbconfig::schema::settings::dsl::*;

        settings
            .first::<Setting>(conn)
            .optional()
            .map_err(map_db_error)
    }

    pub fn update_settings(
        setting: Setting,
        conn: &mut DbConnection,
    ) -> Result<Setting, CustomError> {
        use crate::dbconfig::schema::settings::dsl::*;
        let setting_to_update = settings
            .first::<Setting>(conn)
            .expect("Error loading settings");
        do_retry(|| {
            diesel::update(&setting_to_update)
                .set(setting.clone())
                .get_result::<Setting>(conn)
        })
        .map_err(map_db_error)
    }

    pub fn update_jwt_key(conn: &mut DbConnection) -> Result<(), CustomError> {
        use crate::dbconfig::schema::settings::dsl::*;
        let setting_to_update = settings
            .first::<Setting>(conn)
            .expect("Error loading settings");

        if setting_to_update.jwt_key.is_none() {
            use diesel::ExpressionMethods;
            // Safe because moduluis 2^8 is 256
            let new_jwt_key = RS256KeyPair::generate(2048).unwrap();
            let der_to_safe = new_jwt_key.to_der().unwrap();
            diesel::update(&setting_to_update)
                .set(jwt_key.eq(der_to_safe))
                .get_result::<Setting>(conn)
                .map_err(map_db_error)?;
        }
        Ok(())
    }

    pub fn get_jwt_key(conn: &mut DbConnection) -> Result<RS256KeyPair, CustomError> {
        use crate::dbconfig::schema::settings::dsl::*;
        let jwt_bytes = settings
            .first::<Setting>(conn)
            .map(|setting| setting.jwt_key.unwrap())
            .map_err(map_db_error)?;
        // unwrap: Safe as the key is generated at startup and can't be altered later on
        Ok(RS256KeyPair::from_der(&jwt_bytes).unwrap())
    }

    pub fn insert_default_settings(conn: &mut DbConnection) -> Result<(), CustomError> {
        use crate::dbconfig::schema::settings::dsl::*;
        use diesel::ExpressionMethods;
        do_retry(|| {
            insert_into(settings)
                .values((
                    id.eq(DEFAULT_SETTINGS.id),
                    auto_update.eq(DEFAULT_SETTINGS.auto_update),
                    auto_download.eq(DEFAULT_SETTINGS.auto_download),
                    auto_cleanup.eq(DEFAULT_SETTINGS.auto_cleanup),
                    auto_cleanup_days.eq(DEFAULT_SETTINGS.auto_cleanup_days),
                    podcast_prefill.eq(DEFAULT_SETTINGS.podcast_prefill),
                ))
                .execute(conn)
        })
        .map_err(map_db_error)?;
        Ok(())
    }
}
