use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

mod io;
mod track;

const BUILD_ID: &str = env!("BUILD_ID");

#[derive(Debug, Serialize, Deserialize)]
pub struct FileStateCache {
    build_id: String,
    files: HashMap<PathBuf, u64>,
}

impl Default for FileStateCache {
    fn default() -> Self {
        Self {
            build_id: BUILD_ID.to_string(),
            files: HashMap::new(),
        }
    }
}

impl FileStateCache {

    pub fn error() -> Self {
        Self {
            build_id: String::new(),
            files: HashMap::new(),
        }
    }

    pub fn track(
        &mut self,
        path: impl Into<PathBuf>,
    ) {
        let path = path.into();
        let timestamp = path.metadata()
            .map_or(0, |m| m.modified()
                .map_or(0, |t| t.duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |d| d.as_secs())));

        self.files.insert(path, timestamp);
    }

}

pub struct CacheDiff {
    build_id_changed: bool,
    files_changed: bool,
}

#[allow(dead_code)]
impl CacheDiff {

    pub fn new_dirty() -> Self {
        Self {
            build_id_changed: true,
            files_changed: true,
        }
    }

    pub fn build_id_changed(&self) -> bool {
        self.build_id_changed
    }

    pub fn files_changed(&self) -> bool {
        self.files_changed
    }

    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    pub fn is_some(&self) -> bool {
        self.build_id_changed || self.files_changed
    }

}

impl FileStateCache {

    pub fn diff(&self, other: &Self) -> CacheDiff {
        CacheDiff {
            build_id_changed: !self.eq_build_id(other),
            files_changed: !self.eq_files(other),
        }
    }

    fn eq_build_id(&self, other: &Self) -> bool {
        self.build_id == other.build_id
    }

    fn eq_files(&self, other: &Self) -> bool {
        for (path, timestamp) in &other.files {
            match self.files.get(path) {
                Some(cached_timestamp) => {
                    if cached_timestamp != timestamp {
                        return false;
                    }
                },
                None => {
                    return false;
                }
            }
        }

        for path in self.files.keys() {
            if !other.files.contains_key(path) {
                return false;
            }
        }

        true
    }

}
