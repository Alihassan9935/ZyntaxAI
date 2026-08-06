use directories::ProjectDirs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PathsError {
    #[error("could not determine this platform's application directories")]
    NoHomeDirectory,
    #[error("could not create {path}: {source}")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Clone)]
pub struct Paths {
    config_dir: PathBuf,
    data_dir: PathBuf,
}

impl Paths {
    pub fn resolve() -> Result<Self, PathsError> {
        let dirs = ProjectDirs::from("dev", "TheHolyOneZ", "ZyntaxAI")
            .ok_or(PathsError::NoHomeDirectory)?;

        let paths = Self {
            config_dir: dirs.config_dir().to_path_buf(),
            data_dir: dirs.data_dir().to_path_buf(),
        };
        paths.ensure_dirs()?;
        Ok(paths)
    }

    pub fn rooted_at(root: impl AsRef<Path>) -> Result<Self, PathsError> {
        let root = root.as_ref();
        let paths = Self {
            config_dir: root.join("config"),
            data_dir: root.join("data"),
        };
        paths.ensure_dirs()?;
        Ok(paths)
    }

    pub fn from_env() -> Result<Self, PathsError> {
        match std::env::var_os("ZYNTAX_DATA_DIR") {
            Some(root) => Self::rooted_at(root),
            None => Self::resolve(),
        }
    }

    fn ensure_dirs(&self) -> Result<(), PathsError> {
        for dir in [&self.config_dir, &self.data_dir, &self.logs_dir()] {
            std::fs::create_dir_all(dir).map_err(|source| PathsError::CreateDir {
                path: dir.clone(),
                source,
            })?;
        }
        Ok(())
    }

    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.data_dir.join("logs")
    }

    pub fn settings_file(&self) -> PathBuf {
        self.config_dir.join("settings.json")
    }

    pub fn history_db(&self) -> PathBuf {
        self.data_dir.join("history.db")
    }

    pub fn fallback_key_file(&self) -> PathBuf {
        self.data_dir.join("secrets.key")
    }

    pub fn fallback_secrets_file(&self) -> PathBuf {
        self.data_dir.join("secrets.enc")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rooted_paths_are_created_and_distinct() {
        let temp = tempfile::tempdir().expect("temp dir");
        let paths = Paths::rooted_at(temp.path()).expect("resolve");

        assert!(paths.config_dir().is_dir());
        assert!(paths.data_dir().is_dir());
        assert!(paths.logs_dir().is_dir());
        assert_ne!(paths.config_dir(), paths.data_dir());
    }

    #[test]
    fn resolving_twice_is_idempotent() {
        let temp = tempfile::tempdir().expect("temp dir");
        let first = Paths::rooted_at(temp.path()).expect("resolve");
        let second = Paths::rooted_at(temp.path()).expect("resolve again");
        assert_eq!(first.settings_file(), second.settings_file());
    }

    #[test]
    fn file_paths_live_under_the_right_directories() {
        let temp = tempfile::tempdir().expect("temp dir");
        let paths = Paths::rooted_at(temp.path()).expect("resolve");

        assert!(paths.settings_file().starts_with(paths.config_dir()));
        assert!(paths.history_db().starts_with(paths.data_dir()));
        assert!(paths.fallback_key_file().starts_with(paths.data_dir()));
    }
}
