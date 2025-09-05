use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

mod io;
mod track;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FileStateCache {
    files: HashMap<PathBuf, u64>,
}

impl FileStateCache {

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

impl PartialEq for FileStateCache {

    fn eq(&self, other: &Self) -> bool {
        let mut changed = false;

        for (path, timestamp) in &other.files {
            match self.files.get(path) {
                Some(cached_timestamp) => {
                    if cached_timestamp != timestamp {
                        changed = true;
                    }
                },
                None => {
                    changed = true;
                }
            }
        }

        for path in self.files.keys() {
            if !other.files.contains_key(path) {
                changed = true;
            }
        }

        !changed
    }

}
