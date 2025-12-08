use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModManifest {
    // mandatory
    pub id: String,
    pub name: String,
    pub version: Version,
    #[serde(default)]
    pub capabilities: Vec<Capability>,

    // optional
    #[serde(default)]
    pub dependencies: Vec<Dependency>,

    // optional descriptive
    #[serde(default)]
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub license: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub id: String,
    pub version: VersionReq,
    pub optional: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Capability {
    Export,
    Patch,
    Runtime,
}
