use podfetch_domain::audiobookshelf::library::Library;
use serde::Serialize;

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LibraryFolderDto {
    pub id: String,
    pub full_path: String,
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LibrarySettingsDto {
    pub metadata_precedence: Vec<String>,
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LibraryDto {
    pub id: String,
    pub name: String,
    pub folders: Vec<LibraryFolderDto>,
    pub display_order: i32,
    pub icon: String,
    pub media_type: String,
    pub provider: String,
    pub settings: LibrarySettingsDto,
    pub created_at: i64,
    pub last_update: i64,
}

impl From<&Library> for LibraryDto {
    fn from(value: &Library) -> Self {
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            folders: value
                .folder_paths
                .iter()
                .map(|path| LibraryFolderDto {
                    id: format!("fol_{}", path_hash(path)),
                    full_path: path.clone(),
                })
                .collect(),
            display_order: value.display_order,
            icon: value.icon.clone(),
            media_type: value.media_type.as_str().to_string(),
            provider: "audible".to_string(),
            settings: LibrarySettingsDto {
                metadata_precedence: value.metadata_precedence.clone(),
            },
            created_at: value.created_at.and_utc().timestamp_millis(),
            last_update: value.updated_at.and_utc().timestamp_millis(),
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LibrariesListResponse {
    pub libraries: Vec<LibraryDto>,
}

fn path_hash(path: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
