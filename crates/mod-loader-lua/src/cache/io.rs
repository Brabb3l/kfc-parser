use std::io::{BufReader, BufWriter};

use crate::{alias::Path, cache::FileStateCache, log::warn};

impl FileStateCache {

    pub fn read(
        cache_dir: impl AsRef<Path>
    ) -> Self {
        let cache_file = cache_dir.as_ref().join("files.json");

        if cache_file.exists() {
            match std::fs::File::open(&cache_file) {
                Ok(file) => {
                    let reader = BufReader::new(file);

                    match serde_json::from_reader::<_, Self>(reader) {
                        Ok(cache) => cache,
                        Err(e) => {
                            warn!(
                                error = %e,
                                path = cache_file.as_str(),
                                "Failed to parse cache file, skipping cache loading"
                            );
                            Self::error()
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        error = %e,
                        path = cache_file.as_str(),
                        "Failed to open cache file, skipping cache loading"
                    );

                    Self::error()
                }
            }
        } else {
            Self::error()
        }
    }

    pub fn write(
        &self,
        cache_dir: impl AsRef<Path>
    ) {
        let cache_dir = cache_dir.as_ref();

        if let Err(e) = std::fs::create_dir_all(cache_dir) {
            warn!(
                error = %e,
                path = cache_dir.as_str(),
                "Failed to create cache directory"
            );
        } else {
            let cache_file = cache_dir.join("files.json");

            match std::fs::File::create(&cache_file) {
                Ok(file) => {
                    let writer = BufWriter::new(file);

                    if let Err(e) = serde_json::to_writer(writer, self) {
                        warn!(
                            error = %e,
                            path = cache_file.as_str(),
                            "Failed to write cache file"
                        );
                    }
                }
                Err(e) => {
                    warn!(
                        error = %e,
                        path = cache_file.as_str(),
                        "Failed to create cache file"
                    );
                }
            }
        }
    }

}

