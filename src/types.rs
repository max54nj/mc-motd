use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ServerListPingResponse {
    pub version: Version,
    pub players: Players,
    pub description: Description,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
    #[serde(rename(serialize = "enforcesSecureChat"))]
    pub enforces_secure_chat: bool,
    #[serde(rename(serialize = "previewsChat"))]
    pub previews_chat: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Version {
    pub name: String,
    pub protocol: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Players {
    pub max: i32,
    pub online: i32,
    pub sample: Vec<SamplePlayer>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SamplePlayer {
    pub name: String,
    pub id: String,
}

#[derive(Serialize)]
pub struct Description {
    pub text: String,
}

#[derive(Serialize)]
pub struct KickPayload {
    pub text: String,
}
