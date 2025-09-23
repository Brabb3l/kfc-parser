use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub enable_console: bool,
    #[serde(default)]
    pub use_export_flag: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub export_directory: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enable_console: false,
            use_export_flag: false,
            export_directory: None,
        }
    }
}

impl Config {

    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self, Box<dyn std::error::Error>> {
        if !path.as_ref().exists() {
            let default = Self::default();
            let json = serde_json::to_string_pretty(&default)?;

            std::fs::write(path, json)?;

            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)?;
        let config = serde_json::from_str::<Self>(&content)?;

        Ok(config)
    }

}
