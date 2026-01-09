//! Artifact loading trait and filesystem implementation

use color_eyre::eyre::{eyre, Result};
use smolder_core::Abi;
use std::path::{Path, PathBuf};

use super::types::{ArtifactDetails, ArtifactInfo, ContractArtifact, ContractArtifactFull};

// =============================================================================
// Trait Definition
// =============================================================================

/// Trait for loading contract artifacts from various sources
pub trait ArtifactLoader: Send + Sync {
    /// List all available artifacts
    fn list(&self) -> Result<Vec<ArtifactInfo>>;

    /// Get detailed information about a specific artifact
    fn get_details(&self, name: &str) -> Result<ArtifactDetails>;

    /// Get the bytecode for a specific artifact
    fn get_bytecode(&self, name: &str) -> Result<String>;

    /// Load the raw contract artifact
    fn load(&self, name: &str) -> Result<ContractArtifact>;
}

// =============================================================================
// Filesystem Implementation
// =============================================================================

/// Artifact loader that reads from the filesystem (forge build output)
#[derive(Debug, Clone)]
pub struct FileSystemArtifactLoader {
    /// Directory containing compiled artifacts (typically "out")
    out_dir: PathBuf,
    /// Directory containing source files (typically "src")
    src_dir: PathBuf,
}

impl FileSystemArtifactLoader {
    /// Create a new loader with default paths relative to current directory
    pub fn new() -> Self {
        Self::with_paths(Path::new("."))
    }

    /// Create a new loader with paths relative to the given project root
    pub fn with_paths(project_root: &Path) -> Self {
        Self {
            out_dir: project_root.join("out"),
            src_dir: project_root.join("src"),
        }
    }

    /// Create a new loader with explicit out and src directories
    #[allow(dead_code)]
    pub fn with_dirs(out_dir: PathBuf, src_dir: PathBuf) -> Self {
        Self { out_dir, src_dir }
    }

    /// Check if a source file exists in the src directory (including subdirectories)
    fn source_exists(&self, filename: &str) -> bool {
        self.source_exists_in_dir(&self.src_dir, filename)
    }

    fn source_exists_in_dir(&self, dir: &Path, filename: &str) -> bool {
        if dir.join(filename).exists() {
            return true;
        }

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && self.source_exists_in_dir(&path, filename) {
                    return true;
                }
            }
        }

        false
    }

    /// Find the source path for an artifact
    fn find_source_path(&self, name: &str) -> Option<String> {
        if !self.out_dir.exists() {
            return None;
        }

        for entry in std::fs::read_dir(&self.out_dir).ok()? {
            let entry = entry.ok()?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let dir_name = path.file_name()?.to_str()?;
            let json_path = path.join(format!("{}.json", name));

            if json_path.exists() {
                return Some(dir_name.to_string());
            }
        }

        None
    }
}

impl Default for FileSystemArtifactLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ArtifactLoader for FileSystemArtifactLoader {
    fn list(&self) -> Result<Vec<ArtifactInfo>> {
        if !self.out_dir.exists() {
            return Ok(Vec::new());
        }

        let mut artifacts = Vec::new();

        for entry in std::fs::read_dir(&self.out_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let dir_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };

            // Skip build-info and other special directories
            if dir_name.starts_with('.') || dir_name == "build-info" {
                continue;
            }

            // Only include if the source file exists in src directory
            if !self.source_exists(&dir_name) {
                continue;
            }

            // Look for JSON files in this directory
            for json_entry in std::fs::read_dir(&path)? {
                let json_entry = json_entry?;
                let json_path = json_entry.path();

                if json_path.extension().is_none_or(|e| e != "json") {
                    continue;
                }

                let contract_name = match json_path.file_stem().and_then(|n| n.to_str()) {
                    Some(name) => name.to_string(),
                    None => continue,
                };

                // Skip metadata files
                if contract_name.ends_with(".metadata") {
                    continue;
                }

                if let Ok(content) = std::fs::read_to_string(&json_path) {
                    if let Ok(artifact) = serde_json::from_str::<ContractArtifactFull>(&content) {
                        let has_constructor = Abi::from_value(&artifact.abi)
                            .map(|abi| abi.has_constructor_with_args())
                            .unwrap_or(false);
                        let has_bytecode = artifact.bytecode.is_valid();

                        // Skip artifacts without bytecode (interfaces, abstract contracts)
                        if !has_bytecode {
                            continue;
                        }

                        artifacts.push(ArtifactInfo {
                            name: contract_name,
                            source_path: dir_name.clone(),
                            has_constructor,
                            has_bytecode,
                        });
                    }
                }
            }
        }

        artifacts.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(artifacts)
    }

    fn get_details(&self, name: &str) -> Result<ArtifactDetails> {
        let artifact = self.load(name)?;
        let has_bytecode = artifact.bytecode.is_valid();
        let constructor = Abi::from_value(&artifact.abi)
            .ok()
            .and_then(|abi| abi.constructor());
        let source_path = self
            .find_source_path(name)
            .unwrap_or_else(|| format!("{}.sol", name));

        Ok(ArtifactDetails {
            name: name.to_string(),
            source_path,
            abi: artifact.abi,
            constructor,
            has_bytecode,
        })
    }

    fn get_bytecode(&self, name: &str) -> Result<String> {
        let artifact = self.load(name)?;
        let bytecode = artifact.bytecode.without_prefix().to_string();

        if bytecode.is_empty() {
            return Err(eyre!(
                "Artifact '{}' has no bytecode (may be an interface or abstract contract)",
                name
            ));
        }

        Ok(bytecode)
    }

    fn load(&self, contract_name: &str) -> Result<ContractArtifact> {
        let possible_paths = [
            self.out_dir
                .join(format!("{}.sol", contract_name))
                .join(format!("{}.json", contract_name)),
            self.out_dir
                .join(contract_name)
                .join(format!("{}.json", contract_name)),
        ];

        for path in &possible_paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                let artifact: ContractArtifact = serde_json::from_str(&content)?;
                return Ok(artifact);
            }
        }

        Err(eyre!(
            "Could not find artifact for contract '{}'. Make sure `forge build` was run.",
            contract_name
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_contract_artifact() {
        let json = r#"{
            "abi": [
                {
                    "type": "function",
                    "name": "transfer",
                    "inputs": [
                        {"name": "to", "type": "address"},
                        {"name": "amount", "type": "uint256"}
                    ],
                    "outputs": [{"type": "bool"}]
                }
            ],
            "bytecode": {
                "object": "0x6080604052348015600f57600080fd5b50"
            },
            "deployedBytecode": {
                "object": "0x6080604052"
            }
        }"#;

        let artifact: ContractArtifact = serde_json::from_str(json).unwrap();

        assert!(artifact.abi.is_array());
        assert_eq!(artifact.abi.as_array().unwrap().len(), 1);
        assert!(artifact.bytecode.is_valid());
        assert_eq!(
            artifact.bytecode.without_prefix(),
            "6080604052348015600f57600080fd5b50"
        );
    }

    #[test]
    fn test_bytecode_object_is_valid() {
        let valid = super::super::types::BytecodeObject {
            object: "0x6080604052".to_string(),
        };
        assert!(valid.is_valid());

        let empty = super::super::types::BytecodeObject {
            object: "".to_string(),
        };
        assert!(!empty.is_valid());

        let just_prefix = super::super::types::BytecodeObject {
            object: "0x".to_string(),
        };
        assert!(!just_prefix.is_valid());
    }

    #[test]
    fn test_loader_with_custom_paths() {
        let loader = FileSystemArtifactLoader::with_dirs(
            PathBuf::from("/custom/out"),
            PathBuf::from("/custom/src"),
        );

        assert_eq!(loader.out_dir, PathBuf::from("/custom/out"));
        assert_eq!(loader.src_dir, PathBuf::from("/custom/src"));
    }
}
