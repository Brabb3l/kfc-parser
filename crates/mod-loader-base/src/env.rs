use std::sync::Arc;

use crate::{alias::{Path, PathBuf}, ModEnvironmentErrorReport, ModRegistry};

struct ModEnvironmentInner {
    game_dir: PathBuf,
    cache_dir: PathBuf,
    mods_dir: PathBuf,

    registry: ModRegistry,
}

#[derive(Clone)]
pub struct ModEnvironment {
    inner: Arc<ModEnvironmentInner>,
}

impl ModEnvironment {

    pub fn load(
        game_dir: impl AsRef<Path>,
    ) -> Result<Self, ModEnvironmentErrorReport> {
        let game_dir = game_dir.as_ref().to_path_buf();
        let cache_dir = game_dir.join(".cache");
        let mods_dir = game_dir.join("mods");

        let registry = ModRegistry::load(&mods_dir)?;

        Ok(Self {
            inner: Arc::new(ModEnvironmentInner {
                game_dir,
                cache_dir,
                mods_dir,
                registry,
            }),
        })
    }

    pub fn game_dir(&self) -> &Path {
        &self.inner.game_dir
    }

    pub fn cache_dir(&self) -> &Path {
        &self.inner.cache_dir
    }

    pub fn mods_dir(&self) -> &Path {
        &self.inner.mods_dir
    }

    pub fn mod_registry(&self) -> &ModRegistry {
        &self.inner.registry
    }

}
