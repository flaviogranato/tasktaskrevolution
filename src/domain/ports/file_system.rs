//! File system port for domain file operations
//!
//! This module defines the file system interface that the domain layer
//! requires from the infrastructure layer.

use crate::domain::shared::errors::DomainResult;
use std::path::{Path, PathBuf};

/// File system port for file operations
pub trait FileSystemPort: Send + Sync {
    /// Read a file
    fn read_file(&self, path: &Path) -> DomainResult<Vec<u8>>;

    /// Write a file
    fn write_file(&self, path: &Path, content: &[u8]) -> DomainResult<()>;

    /// Check if a file exists
    fn file_exists(&self, path: &Path) -> bool;

    /// Check if a directory exists
    fn dir_exists(&self, path: &Path) -> bool;

    /// Create a directory
    fn create_dir(&self, path: &Path) -> DomainResult<()>;

    /// Create directories recursively
    fn create_dir_all(&self, path: &Path) -> DomainResult<()>;

    /// Remove a file
    fn remove_file(&self, path: &Path) -> DomainResult<()>;

    /// Remove a directory
    fn remove_dir(&self, path: &Path) -> DomainResult<()>;

    /// Remove a directory and all its contents
    fn remove_dir_all(&self, path: &Path) -> DomainResult<()>;

    /// List directory contents
    fn read_dir(&self, path: &Path) -> DomainResult<Vec<PathBuf>>;

    /// Get file metadata
    fn metadata(&self, path: &Path) -> DomainResult<FileMetadata>;

    /// Copy a file
    fn copy_file(&self, from: &Path, to: &Path) -> DomainResult<()>;

    /// Move a file
    fn move_file(&self, from: &Path, to: &Path) -> DomainResult<()>;
}

/// File metadata
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub is_file: bool,
    pub is_dir: bool,
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    pub modified: Option<chrono::DateTime<chrono::Utc>>,
    pub permissions: FilePermissions,
}

/// File permissions
#[derive(Debug, Clone)]
pub struct FilePermissions {
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
}

impl Default for FilePermissions {
    fn default() -> Self {
        Self {
            readable: true,
            writable: true,
            executable: false,
        }
    }
}
