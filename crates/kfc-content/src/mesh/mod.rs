mod pbr;

pub use pbr::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum RenderEffectId {
    Pbr,
}
