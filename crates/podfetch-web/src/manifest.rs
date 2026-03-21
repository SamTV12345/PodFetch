use serde::Serialize;

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct Icon {
    pub src: String,
    pub sizes: String,
    pub r#type: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub name: String,
    pub short_name: String,
    pub start_url: String,
    pub icons: Vec<Icon>,
    pub theme_color: String,
    pub background_color: String,
    pub display: String,
    pub orientation: String,
}

pub fn build_manifest(server_url: &str) -> Manifest {
    Manifest {
        name: "PodFetch".to_string(),
        short_name: "PodFetch".to_string(),
        start_url: server_url.to_string(),
        icons: vec![Icon {
            src: format!("{server_url}ui/logo.png"),
            sizes: "512x512".to_string(),
            r#type: "image/png".to_string(),
        }],
        orientation: "landscape".to_string(),
        theme_color: "#ffffff".to_string(),
        display: "fullscreen".to_string(),
        background_color: "#ffffff".to_string(),
    }
}
