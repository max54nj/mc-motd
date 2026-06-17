use std::{fs, path::Path};

use color_eyre::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::{ChatComponent, Players, Version};

#[derive(Deserialize, Serialize, Clone, JsonSchema)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    #[serde(rename = "$schema")]
    _schema: String,
    pub port: u16,
    pub host: String,
    pub version: Version,
    pub players: Players,
    pub motd: String,
    pub motd_json: ChatComponent,
    pub favicon: Option<String>,
    pub kick_message: String,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_file(path: &Path) -> Result<Self> {
        let config_json = fs::read_to_string(path).unwrap();

        let parsed = serde_json::from_str(&config_json).unwrap();

        Ok(parsed)
    }

    pub fn write_file(&self, path: &Path) -> Result<()> {
        let config_json = self.to_json_string_pretty().unwrap();
        fs::write(path, config_json).unwrap();
        Ok(())
    }

    fn to_json_string_pretty(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self).unwrap())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            _schema: "./config.schema.json".to_string(),
            port: 25565,
            host: "0.0.0.0".to_string(),
            version: Version {
                name: "1.21.11".to_string(),
                protocol: 774,
            },
            players: Players {
                max: 20,
                online: 0,
                sample: vec![],
            },
            motd: "A Minecraft Server".to_string(),
            motd_json: ChatComponent {
                text: "A Minecraft Server".to_string(),
                color: None,
                bold: None,
                extra: Some(vec![]),
            },
            favicon: None,
            kick_message: "Disconnected".to_string(),
        }
    }
}
