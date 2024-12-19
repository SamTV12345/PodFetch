use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct OpmlModel {
    pub content: String,
}
