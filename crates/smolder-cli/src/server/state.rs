//! Server application state

use std::sync::Arc;

use crate::db::Database;
use crate::forge::{ArtifactLoader, FileSystemArtifactLoader};

/// Application state shared across handlers
///
/// Uses the Database which implements all repository traits,
/// providing type-safe data access through the repository pattern.
#[derive(Clone)]
pub struct AppState {
    db: Arc<Database>,
    artifact_loader: Arc<dyn ArtifactLoader>,
}

impl AppState {
    /// Create a new AppState with the given database
    pub fn new(db: Database) -> Self {
        Self {
            db: Arc::new(db),
            artifact_loader: Arc::new(FileSystemArtifactLoader::new()),
        }
    }

    /// Get a reference to the database
    pub fn db(&self) -> &Database {
        &self.db
    }

    /// Get a reference to the artifact loader
    pub fn artifacts(&self) -> &dyn ArtifactLoader {
        self.artifact_loader.as_ref()
    }
}
