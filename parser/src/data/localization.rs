use serde::{Deserialize, Serialize};

use super::HashKey32;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocaTagCollectionResourceData {
    pub tags: Vec<LocaTagResource>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocaTagResource {
    pub id: HashKey32,
    pub text: String,
    pub arguments: Vec<LocaTagArgument>,
    pub generic_arguments: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocaTagArgument {
    pub id: u32,
    pub r#type: LocaArgumentType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LocaArgumentType {
    Generic,
    Input,
    Config,
    Balancing,
}
