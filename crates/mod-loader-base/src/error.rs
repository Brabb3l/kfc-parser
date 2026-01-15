use thiserror::Error;

use crate::{alias::{Path, PathBuf}, ModRegistry};

#[derive(Debug, Error)]
#[error("Error loading mod environment")]
pub struct ModEnvironmentErrorReport {
    pub error: Option<IoError>,
    pub mods: Vec<ModErrorReport>,
    pub mod_registry: ModRegistry,
}

impl ModEnvironmentErrorReport {

    pub(crate) fn base_io(
        error: IoError,
    ) -> Self {
        Self {
            error: Some(error),
            mods: Default::default(),
            mod_registry: Default::default(),
        }
    }

    pub(crate) fn with_mod_errors(
        mod_errors: Vec<ModErrorReport>,
        mod_registry: ModRegistry,
    ) -> Self {
        Self {
            error: None,
            mods: mod_errors,
            mod_registry,
        }
    }

}

#[derive(Debug, Error)]
#[error("Error in mod at {path}")]
pub struct ModErrorReport {
    pub path: PathBuf,
    pub id: Option<String>,
    pub error: ModError,
}

impl ModErrorReport {

    pub(crate) fn new(
        path: impl AsRef<Path>,
        error: ModError,
    ) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            id: None,
            error,
        }
    }

    pub(crate) fn with_id(
        mut self,
        id: String,
    ) -> Self {
        self.id = Some(id);
        self
    }

}


#[derive(Debug, Error)]
pub enum ModError {
    #[error("{0}")]
    Io(IoError),
    #[error("Path is not valid UTF-8: {0}")]
    Utf8(std::path::PathBuf),
    #[error("JSON error in {path}: {source}")]
    Json {
        path: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("Duplicate mod ID: {0}")]
    DuplicateModId(String),
}

#[derive(Debug, Error)]
#[error("IO error at {path}: {source}")]
pub struct IoError {
    pub path: String,
    #[source]
    pub source: std::io::Error,
}
