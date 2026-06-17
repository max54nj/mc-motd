use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ServerListPingResponse {
    pub version: Version,
    pub players: Players,
    pub description: ChatComponent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
    #[serde(rename(serialize = "enforcesSecureChat"))]
    pub enforces_secure_chat: bool,
    #[serde(rename(serialize = "previewsChat"))]
    pub previews_chat: bool,
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct Version {
    pub name: String,
    pub protocol: i32,
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct Players {
    pub max: i32,
    pub online: i32,
    pub sample: Vec<SamplePlayer>,
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct SamplePlayer {
    pub name: String,
    pub id: String,
}

#[derive(Serialize)]
pub struct KickPayload {
    pub text: String,
}

#[derive(Deserialize, Serialize, Clone, JsonSchema)]
pub struct ChatComponent {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<Vec<ChatComponent>>,
}
