//! Smolder directory management
//!
//! The [`SmolderDir`] struct manages the `.smolder/` directory where all
//! project-local smolder data is stored.

use std::path::{Path, PathBuf};

/// Manages the `.smolder/` directory for project-local data storage.
///
/// All smolder data (database, config, cache, etc.) lives under this directory,
/// keeping the project root clean and requiring only a single `.gitignore` entry.
#[derive(Debug, Clone)]
pub struct SmolderDir {
    path: PathBuf,
}

impl SmolderDir {
    /// The directory name used for smolder data
    pub const NAME: &str = ".smolder";

    /// Create a new `SmolderDir` pointing to `.smolder/` in the current directory.
    pub fn new() -> Self {
        Self {
            path: PathBuf::from(Self::NAME),
        }
    }

    /// Create a `SmolderDir` at a custom location.
    pub fn at<P: Into<PathBuf>>(path: P) -> Self {
        Self { path: path.into() }
    }

    /// Get the path to the smolder directory.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Join a relative path to the smolder directory.
    pub fn join<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.path.join(path)
    }

    /// Check if the smolder directory exists.
    pub fn exists(&self) -> bool {
        self.path.is_dir()
    }

    /// Create the smolder directory if it doesn't exist.
    pub fn create(&self) -> std::io::Result<()> {
        if !self.exists() {
            std::fs::create_dir_all(&self.path)?;
        }
        Ok(())
    }
}

impl Default for SmolderDir {
    fn default() -> Self {
        Self::new()
    }
}

impl AsRef<Path> for SmolderDir {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let dir = SmolderDir::new();
        assert_eq!(dir.path(), Path::new(".smolder"));
    }

    #[test]
    fn test_at() {
        let dir = SmolderDir::at("/custom/path/.smolder");
        assert_eq!(dir.path(), Path::new("/custom/path/.smolder"));
    }

    #[test]
    fn test_join() {
        let dir = SmolderDir::new();
        assert_eq!(dir.join("smolder.db"), PathBuf::from(".smolder/smolder.db"));
        assert_eq!(
            dir.join("cache/artifacts"),
            PathBuf::from(".smolder/cache/artifacts")
        );
    }

    #[test]
    fn test_default() {
        let dir = SmolderDir::default();
        assert_eq!(dir.path(), Path::new(".smolder"));
    }
}
