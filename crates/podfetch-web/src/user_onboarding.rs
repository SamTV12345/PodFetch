use serde::Deserialize;
use utoipa::ToSchema;

use crate::user_admin::UserSummary;

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserOnboardingModel {
    pub invite_id: String,
    pub username: String,
    pub password: String,
}

pub trait UserOnboardingApplicationService {
    type Error;

    fn onboard_user(&self, request: UserOnboardingModel) -> Result<UserSummary, Self::Error>;
}

pub fn onboard_user<S>(service: &S, request: UserOnboardingModel) -> Result<UserSummary, S::Error>
where
    S: UserOnboardingApplicationService,
{
    service.onboard_user(request)
}
