use std::path::PathBuf;

/// Location of an installed client or dedicated server used by ignored tests.
pub struct GameFiles {
    pub dir: PathBuf,
    pub file_name: String,
}

impl GameFiles {
    pub fn from_env() -> Self {
        let dir = PathBuf::from(
            std::env::var_os("GAME_DIR").expect("GAME_DIR environment variable not set"),
        );
        assert!(
            dir.is_dir(),
            "GAME_DIR is not an existing directory: {}",
            dir.display()
        );

        // Set GAME_FILE_NAME=enshrouded_server for the dedicated server.
        let file_name = std::env::var("GAME_FILE_NAME").unwrap_or_else(|_| "enshrouded".to_owned());

        Self { dir, file_name }
    }

    pub fn path(&self, extension: &str) -> PathBuf {
        self.dir.join(format!("{}.{}", self.file_name, extension))
    }
}
