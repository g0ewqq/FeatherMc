use std::path::{Path, PathBuf};

use crate::error::Result;

#[derive(Debug, Clone)]
pub struct RuntimeDirs {
    pub base: PathBuf,
    pub config: PathBuf,
    pub logs: PathBuf,
    pub worlds: PathBuf,
    pub players: PathBuf,
    pub plugins: PathBuf,
}

impl RuntimeDirs {
    #[must_use]
    pub fn new(base: &Path) -> Self {
        Self {
            base: base.to_path_buf(),
            config: base.join("config"),
            logs: base.join("logs"),
            worlds: base.join("worlds"),
            players: base.join("players"),
            plugins: base.join("plugins"),
        }
    }

    pub fn ensure_all(&self) -> Result<()> {
        for dir in [
            &self.config,
            &self.logs,
            &self.worlds,
            &self.players,
            &self.plugins,
        ] {
            std::fs::create_dir_all(dir)?;
        }
        Ok(())
    }

    #[must_use]
    pub fn config_file(&self) -> PathBuf {
        self.config.join("server.toml")
    }
}

pub fn load_or_ensure_dirs(base: &Path) -> Result<RuntimeDirs> {
    let dirs = RuntimeDirs::new(base);
    dirs.ensure_all()?;
    Ok(dirs)
}

pub fn default_config_path(base: &Path) -> PathBuf {
    RuntimeDirs::new(base).config_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_missing_directories() {
        let tmp = tempfile::tempdir().unwrap();
        let dirs = RuntimeDirs::new(tmp.path());

        for dir in [&dirs.config, &dirs.logs, &dirs.worlds, &dirs.plugins] {
            assert!(!dir.exists());
        }

        dirs.ensure_all().unwrap();

        for dir in [&dirs.config, &dirs.logs, &dirs.worlds, &dirs.plugins] {
            assert!(dir.is_dir());
        }
    }

    #[test]
    fn ensure_is_idempotent_and_preserves_files() {
        let tmp = tempfile::tempdir().unwrap();
        let dirs = RuntimeDirs::new(tmp.path());
        dirs.ensure_all().unwrap();

        let marker = dirs.plugins.join("keep.txt");
        std::fs::write(&marker, "data").unwrap();

        dirs.ensure_all().unwrap();
        assert_eq!(std::fs::read_to_string(&marker).unwrap(), "data");
    }

    #[test]
    fn config_file_lives_under_config_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let dirs = RuntimeDirs::new(tmp.path());
        assert_eq!(dirs.config_file(), dirs.config.join("server.toml"));
    }
}
