use crate::app_state::AppState;
use crate::controllers::id_resolver::{ResolvedId, parse_resolved_id};
use axum::extract::Path;
use axum::{Extension, Json};
use common_infrastructure::error::ErrorSeverity::Warning;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use podfetch_domain::user::User;
use podfetch_persistence::sponsorblock::{SponsorblockRepository, SponsorblockUserSettingsEntity};
use crate::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use serde::{Deserialize, Serialize};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Serialize, Deserialize, Debug, Clone, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SponsorSegmentDto {
    pub uuid: String,
    pub category: String,
    pub action_type: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub votes: i32,
    pub locked: bool,
    pub duration_mismatch: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SponsorblockUserSettingsDto {
    pub enabled: bool,
    pub skip_sponsor: bool,
    pub skip_selfpromo: bool,
    pub skip_interaction: bool,
    pub skip_intro: bool,
    pub skip_outro: bool,
    pub skip_preview: bool,
    pub skip_filler: bool,
    pub skip_music_offtopic: bool,
}

impl SponsorblockUserSettingsDto {
    fn defaults() -> Self {
        Self {
            enabled: true,
            skip_sponsor: true,
            skip_selfpromo: true,
            skip_interaction: false,
            skip_intro: false,
            skip_outro: false,
            skip_preview: false,
            skip_filler: false,
            skip_music_offtopic: false,
        }
    }

    fn into_entity(self, user_id: String) -> SponsorblockUserSettingsEntity {
        SponsorblockUserSettingsEntity {
            user_id,
            enabled: self.enabled,
            skip_sponsor: self.skip_sponsor,
            skip_selfpromo: self.skip_selfpromo,
            skip_interaction: self.skip_interaction,
            skip_intro: self.skip_intro,
            skip_outro: self.skip_outro,
            skip_preview: self.skip_preview,
            skip_filler: self.skip_filler,
            skip_music_offtopic: self.skip_music_offtopic,
        }
    }
}

impl From<SponsorblockUserSettingsEntity> for SponsorblockUserSettingsDto {
    fn from(e: SponsorblockUserSettingsEntity) -> Self {
        Self {
            enabled: e.enabled,
            skip_sponsor: e.skip_sponsor,
            skip_selfpromo: e.skip_selfpromo,
            skip_interaction: e.skip_interaction,
            skip_intro: e.skip_intro,
            skip_outro: e.skip_outro,
            skip_preview: e.skip_preview,
            skip_filler: e.skip_filler,
            skip_music_offtopic: e.skip_music_offtopic,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SponsorblockEpisodeResponse {
    pub segments: Vec<SponsorSegmentDto>,
    pub preferences: SponsorblockUserSettingsDto,
}

/// Resolve an episode `{id}` path segment (UUID or legacy integer) to the
/// canonical episode `Uuid`. Replicates the logic from
/// `podcast_episode_controller::resolve_episode_uuid`.
fn resolve_episode_uuid(id: &str) -> Result<uuid::Uuid, CustomError> {
    match parse_resolved_id(id)? {
        ResolvedId::Uuid(uuid) => Ok(uuid),
        ResolvedId::Legacy(legacy) => {
            let episode = PodcastEpisodeService::get_podcast_episode_by_legacy_id(legacy)?
                .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Warning)))?;
            uuid::Uuid::parse_str(&episode.id)
                .map_err(|_| CustomErrorInner::NotFound(Warning).into())
        }
    }
}

#[utoipa::path(
    get,
    path = "/podcasts/episodes/{id}/sponsorblock",
    responses(
        (status = 200, description = "SponsorBlock segments + caller preferences", body = SponsorblockEpisodeResponse)
    ),
    tag = "sponsorblock"
)]
pub async fn get_episode_sponsorblock(
    Path(id): Path<String>,
    Extension(requester): Extension<User>,
) -> Result<Json<SponsorblockEpisodeResponse>, CustomError> {
    let episode_id = resolve_episode_uuid(&id)?.to_string();

    let segments = SponsorblockRepository::get_segments_for_episode(&episode_id)?
        .into_iter()
        .map(|e| SponsorSegmentDto {
            uuid: e.uuid,
            category: e.category,
            action_type: e.action_type,
            start_ms: e.start_ms,
            end_ms: e.end_ms,
            votes: e.votes,
            locked: e.locked,
            duration_mismatch: e.duration_mismatch,
        })
        .collect();

    let preferences =
        match SponsorblockRepository::get_user_settings(&requester.id.to_string())? {
            Some(e) => e.into(),
            None => SponsorblockUserSettingsDto::defaults(),
        };

    Ok(Json(SponsorblockEpisodeResponse {
        segments,
        preferences,
    }))
}

#[utoipa::path(
    get,
    path = "/settings/sponsorblock",
    responses(
        (status = 200, description = "Current user's SponsorBlock preferences", body = SponsorblockUserSettingsDto)
    ),
    tag = "sponsorblock"
)]
pub async fn get_sponsorblock_settings(
    Extension(requester): Extension<User>,
) -> Result<Json<SponsorblockUserSettingsDto>, CustomError> {
    let prefs = match SponsorblockRepository::get_user_settings(&requester.id.to_string())? {
        Some(e) => e.into(),
        None => SponsorblockUserSettingsDto::defaults(),
    };
    Ok(Json(prefs))
}

#[utoipa::path(
    put,
    path = "/settings/sponsorblock",
    request_body = SponsorblockUserSettingsDto,
    responses(
        (status = 200, description = "Updated SponsorBlock preferences", body = SponsorblockUserSettingsDto)
    ),
    tag = "sponsorblock"
)]
pub async fn update_sponsorblock_settings(
    Extension(requester): Extension<User>,
    Json(body): Json<SponsorblockUserSettingsDto>,
) -> Result<Json<SponsorblockUserSettingsDto>, CustomError> {
    SponsorblockRepository::upsert_user_settings(
        body.clone().into_entity(requester.id.to_string()),
    )?;
    Ok(Json(body))
}

pub fn get_sponsorblock_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_episode_sponsorblock))
        .routes(routes!(get_sponsorblock_settings))
        .routes(routes!(update_sponsorblock_settings))
}

#[cfg(test)]
mod tests {
    use crate::test_support::tests::handle_test_startup;
    use serde_json::{Value, json};
    use serial_test::serial;
    use uuid::Uuid;

    #[tokio::test]
    #[serial]
    async fn sponsorblock_settings_default_then_roundtrip() {
        let server = handle_test_startup().await;

        // 1. GET /api/v1/settings/sponsorblock → defaults
        let get_resp = server
            .test_server
            .get("/api/v1/settings/sponsorblock")
            .await;
        assert_eq!(get_resp.status_code(), 200);
        let defaults = get_resp.json::<Value>();
        assert_eq!(defaults["enabled"], json!(true));
        assert_eq!(defaults["skipSponsor"], json!(true));
        assert_eq!(defaults["skipSelfpromo"], json!(true));
        assert_eq!(defaults["skipInteraction"], json!(false));

        // 2. PUT /api/v1/settings/sponsorblock → update prefs
        let put_resp = server
            .test_server
            .put("/api/v1/settings/sponsorblock")
            .json(&json!({
                "enabled": true,
                "skipSponsor": false,
                "skipSelfpromo": true,
                "skipInteraction": true,
                "skipIntro": false,
                "skipOutro": false,
                "skipPreview": false,
                "skipFiller": false,
                "skipMusicOfftopic": false
            }))
            .await;
        assert_eq!(put_resp.status_code(), 200);

        // 3. GET again → changes persisted
        let get_resp2 = server
            .test_server
            .get("/api/v1/settings/sponsorblock")
            .await;
        assert_eq!(get_resp2.status_code(), 200);
        let updated = get_resp2.json::<Value>();
        assert_eq!(updated["skipSponsor"], json!(false));
        assert_eq!(updated["skipInteraction"], json!(true));
        assert_eq!(updated["enabled"], json!(true));

        // 4. Optional: GET /api/v1/podcasts/episodes/{random-uuid}/sponsorblock
        //    → empty segments + default-like preferences (the user now has stored prefs)
        let episode_uuid = Uuid::new_v4().to_string();
        let ep_resp = server
            .test_server
            .get(&format!(
                "/api/v1/podcasts/episodes/{episode_uuid}/sponsorblock"
            ))
            .await;
        assert_eq!(ep_resp.status_code(), 200);
        let ep_body = ep_resp.json::<Value>();
        assert!(
            ep_body["segments"].as_array().unwrap().is_empty(),
            "no segments for a fresh episode"
        );
        // preferences reflect what we PUT above
        assert_eq!(ep_body["preferences"]["skipSponsor"], json!(false));
        assert_eq!(ep_body["preferences"]["skipInteraction"], json!(true));
    }
}
