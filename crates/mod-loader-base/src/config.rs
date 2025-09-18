use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub enable_console: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enable_console: false,
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
