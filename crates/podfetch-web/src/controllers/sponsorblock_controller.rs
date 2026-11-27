use crate::app_state::AppState;
use crate::controllers::podcast_episode_controller::resolve_episode_uuid;
use axum::extract::{Path, State};
use axum::{Extension, Json};
use common_infrastructure::error::CustomError;
use podfetch_identity::api::Principal;
use podfetch_media::sponsorblock::{SkipPreferences, SponsorSegment};
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

impl From<SponsorSegment> for SponsorSegmentDto {
    fn from(s: SponsorSegment) -> Self {
        Self {
            uuid: s.uuid,
            category: s.category,
            action_type: s.action_type,
            start_ms: s.start_ms,
            end_ms: s.end_ms,
            votes: s.votes,
            locked: s.locked,
            duration_mismatch: s.duration_mismatch,
        }
    }
}

impl From<SkipPreferences> for SponsorblockUserSettingsDto {
    fn from(p: SkipPreferences) -> Self {
        Self {
            enabled: p.enabled,
            skip_sponsor: p.skip_sponsor,
            skip_selfpromo: p.skip_selfpromo,
            skip_interaction: p.skip_interaction,
            skip_intro: p.skip_intro,
            skip_outro: p.skip_outro,
            skip_preview: p.skip_preview,
            skip_filler: p.skip_filler,
            skip_music_offtopic: p.skip_music_offtopic,
        }
    }
}

impl From<SponsorblockUserSettingsDto> for SkipPreferences {
    fn from(d: SponsorblockUserSettingsDto) -> Self {
        Self {
            enabled: d.enabled,
            skip_sponsor: d.skip_sponsor,
            skip_selfpromo: d.skip_selfpromo,
            skip_interaction: d.skip_interaction,
            skip_intro: d.skip_intro,
            skip_outro: d.skip_outro,
            skip_preview: d.skip_preview,
            skip_filler: d.skip_filler,
            skip_music_offtopic: d.skip_music_offtopic,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SponsorblockEpisodeResponse {
    pub segments: Vec<SponsorSegmentDto>,
    pub preferences: SponsorblockUserSettingsDto,
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
    State(state): State<AppState>,
    Path(id): Path<String>,
    Extension(requester): Extension<Principal>,
) -> Result<Json<SponsorblockEpisodeResponse>, CustomError> {
    let episode_id = resolve_episode_uuid(&id)?.to_string();
    let sponsorblock = &state.sponsorblock_service;

    let segments = sponsorblock
        .segments(&episode_id)?
        .into_iter()
        .map(Into::into)
        .collect();
    let preferences = sponsorblock.preferences(requester.id)?.into();

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
    State(state): State<AppState>,
    Extension(requester): Extension<Principal>,
) -> Result<Json<SponsorblockUserSettingsDto>, CustomError> {
    let preferences = state.sponsorblock_service.preferences(requester.id)?;
    Ok(Json(preferences.into()))
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
    State(state): State<AppState>,
    Extension(requester): Extension<Principal>,
    Json(body): Json<SponsorblockUserSettingsDto>,
) -> Result<Json<SponsorblockUserSettingsDto>, CustomError> {
    state
        .sponsorblock_service
        .save_preferences(requester.id, &body.clone().into())?;
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
