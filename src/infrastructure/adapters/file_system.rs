//! File system adapter implementation
//!
//! This module provides a concrete implementation of the FileSystemPort
//! using the standard library file system operations.

use crate::domain::ports::file_system::{FileSystemPort, FileMetadata, FilePermissions};
use crate::domain::shared::errors::{DomainError, DomainResult};
use std::path::{Path, PathBuf};
use std::fs;

/// Standard file system adapter
pub struct StandardFileSystemAdapter;

impl StandardFileSystemAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StandardFileSystemAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystemPort for StandardFileSystemAdapter {
    fn read_file(&self, path: &Path) -> DomainResult<Vec<u8>> {
        fs::read(path).map_err(|e| DomainError::IoErrorWithPath {
            operation: "read file".to_string(),
            path: path.to_string_lossy().to_string(),
            details: e.to_string(),
        })
    }

    fn write_file(&self, path: &Path, content: &[u8]) -> DomainResult<()> {
        fs::write(path, content).map_err(|e| DomainError::IoErrorWithPath {
            operation: "write file".to_string(),
            path: path.to_string_lossy().to_string(),
            details: e.to_string(),
        })
    }

    fn file_exists(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn dir_exists(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn create_dir(&self, path: &Path) -> DomainResult<()> {
        fs::create_dir(path).map_err(|e| DomainError::IoErrorWithPath {
            operation: "create directory".to_string(),
            path: path.to_string_lossy().to_string(),
            details: e.to_string(),
        })
    }

    fn create_dir_all(&self, path: &Path) -> DomainResult<()> {
        fs::create_dir_all(path).map_err(|e| DomainError::IoErrorWithPath {
            operation: "create directories".to_string(),
            path: path.to_string_lossy().to_string(),
            details: e.to_string(),
        })
    }

    fn remove_file(&self, path: &Path) -> DomainResult<()> {
        fs::remove_file(path).map_err(|e| DomainError::IoErrorWithPath {
            operation: "remove file".to_string(),
            path: path.to_string_lossy().to_string(),
            details: e.to_string(),
        })
    }

    fn remove_dir(&self, path: &Path) -> DomainResult<()> {
        fs::remove_dir(path).map_err(|e| DomainError::IoErrorWithPath {
            operation: "remove directory".to_string(),
            path: path.to_string_lossy().to_string(),
            details: e.to_string(),
        })
    }

    fn remove_dir_all(&self, path: &Path) -> DomainResult<()> {
        fs::remove_dir_all(path).map_err(|e| DomainError::IoErrorWithPath {
            operation: "remove directory tree".to_string(),
            path: path.to_string_lossy().to_string(),
            details: e.to_string(),
        })
    }

    fn read_dir(&self, path: &Path) -> DomainResult<Vec<PathBuf>> {
        let entries = fs::read_dir(path).map_err(|e| DomainError::IoErrorWithPath {
            operation: "read directory".to_string(),
            path: path.to_string_lossy().to_string(),
            details: e.to_string(),
        })?;

        let paths: Result<Vec<PathBuf>, _> = entries
            .map(|entry| entry.map(|e| e.path()))
            .collect();

        paths.map_err(|e| DomainError::IoErrorWithPath {
            operation: "read directory entries".to_string(),
            path: path.to_string_lossy().to_string(),
            details: e.to_string(),
        })
    }

    fn metadata(&self, path: &Path) -> DomainResult<FileMetadata> {
        let metadata = fs::metadata(path).map_err(|e| DomainError::IoErrorWithPath {
            operation: "get metadata".to_string(),
            path: path.to_string_lossy().to_string(),
            details: e.to_string(),
        })?;

        let permissions = FilePermissions {
            readable: true, // Simplified - would need more complex logic
            writable: !metadata.permissions().readonly(),
            executable: false, // Simplified - would need more complex logic
        };

        Ok(FileMetadata {
            size: metadata.len(),
            is_file: metadata.is_file(),
            is_dir: metadata.is_dir(),
            created: metadata.created().ok().and_then(|t| {
                chrono::DateTime::from_timestamp(
                    t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i64,
                    0,
                ).map(|dt| dt.with_timezone(&chrono::Utc))
            }),
            modified: metadata.modified().ok().and_then(|t| {
                chrono::DateTime::from_timestamp(
                    t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i64,
                    0,
                ).map(|dt| dt.with_timezone(&chrono::Utc))
            }),
            permissions,
        })
    }

    fn copy_file(&self, from: &Path, to: &Path) -> DomainResult<()> {
        fs::copy(from, to).map_err(|e| DomainError::IoErrorWithPath {
            operation: "copy file".to_string(),
            path: from.to_string_lossy().to_string(),
            details: e.to_string(),
        })?;
        Ok(())
    }

    fn move_file(&self, from: &Path, to: &Path) -> DomainResult<()> {
        fs::rename(from, to).map_err(|e| DomainError::IoErrorWithPath {
            operation: "move file".to_string(),
            path: from.to_string_lossy().to_string(),
            details: e.to_string(),
        })
    }
}
