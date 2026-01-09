//! Forge artifact and broadcast handling
//!
//! This module provides utilities for:
//! - Loading contract artifacts from forge build output
//! - Parsing broadcast outputs from forge script
//! - Extracting deployment information
//!
//! # Traits
//!
//! The module provides two main traits for dependency injection and testing:
//!
//! - [`ArtifactLoader`] - For loading contract artifacts from various sources
//! - [`BroadcastParser`] - For parsing broadcast outputs from deployment scripts
//!
//! # Implementations
//!
//! - [`FileSystemArtifactLoader`] - Loads artifacts from forge build output on disk
//! - [`ForgeBroadcastParser`] - Parses forge script broadcast files
//!
//! # Example
//!
//! ```ignore
//! use crate::forge::{ArtifactLoader, FileSystemArtifactLoader};
//!
//! let loader = FileSystemArtifactLoader::with_paths(Path::new("/my/project"));
//! let artifacts = loader.list()?;
//! let details = loader.get_details("MyContract")?;
//! ```

mod artifact;
mod broadcast;
mod types;

// Re-export traits
pub use artifact::ArtifactLoader;
pub use broadcast::BroadcastParser;

// Re-export implementations
pub use artifact::FileSystemArtifactLoader;
pub use broadcast::ForgeBroadcastParser;

// Re-export data types
pub use types::{ArtifactDetails, ArtifactInfo, BroadcastOutput};
