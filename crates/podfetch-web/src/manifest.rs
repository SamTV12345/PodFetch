use serde::Serialize;

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct Icon {
    pub src: String,
    pub sizes: String,
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub id: String,
    pub name: String,
    pub short_name: String,
    pub description: String,
    pub start_url: String,
    pub scope: String,
    pub icons: Vec<Icon>,
    pub theme_color: String,
    pub background_color: String,
    pub display: String,
    pub orientation: String,
}

pub fn build_manifest(server_url: &str) -> Manifest {
    let ui_url = format!("{server_url}ui/");
    Manifest {
        id: ui_url.clone(),
        name: "PodFetch".to_string(),
        short_name: "PodFetch".to_string(),
        description: "Open source podcast manager and player.".to_string(),
        start_url: ui_url.clone(),
        scope: ui_url.clone(),
        icons: vec![
            Icon {
                src: format!("{ui_url}pwa-192x192.png"),
                sizes: "192x192".to_string(),
                r#type: "image/png".to_string(),
                purpose: None,
            },
            Icon {
                src: format!("{ui_url}pwa-512x512.png"),
                sizes: "512x512".to_string(),
                r#type: "image/png".to_string(),
                purpose: None,
            },
            Icon {
                src: format!("{ui_url}pwa-512x512.png"),
                sizes: "512x512".to_string(),
                r#type: "image/png".to_string(),
                purpose: Some("maskable any".to_string()),
            },
        ],
        orientation: "portrait-primary".to_string(),
        theme_color: "#101010".to_string(),
        display: "standalone".to_string(),
        background_color: "#101010".to_string(),
    }
}
