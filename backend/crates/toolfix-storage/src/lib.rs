//! Object/file storage abstraction.
//!
//! Postgres stores metadata; actual bytes live behind [`StorageBackend`].
//! The local filesystem backend ships first; an S3-compatible backend can
//! implement the same trait later without touching callers.

use async_trait::async_trait;
use bytes::Bytes;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("invalid storage key: {0}")]
    InvalidKey(String),
    #[error("object not found: {0}")]
    NotFound(String),
    #[error("storage backend error: {0}")]
    Backend(String),
}

#[derive(Debug, Clone)]
pub struct StoredObject {
    pub key: String,
    pub content_type: Option<String>,
    pub size_bytes: u64,
}

#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Store bytes under a logical key; returns the canonical stored key.
    async fn put(
        &self,
        key: &str,
        content_type: Option<&str>,
        data: Bytes,
    ) -> Result<StoredObject, StorageError>;

    async fn get(&self, key: &str) -> Result<Bytes, StorageError>;

    async fn delete(&self, key: &str) -> Result<(), StorageError>;
}

/// Byte buffer alias used by callers of the storage layer.
pub type Blob = Bytes;

/// Keys must be relative, normalized, and stay inside the bucket: no
/// absolute paths, no `..`, no backslashes.
pub fn validate_key(key: &str) -> Result<(), StorageError> {
    if key.is_empty()
        || key.starts_with('/')
        || key.contains("..")
        || key.contains('\\')
        || key.contains('\0')
    {
        return Err(StorageError::InvalidKey(key.to_string()));
    }
    Ok(())
}

pub mod local;
pub use local::LocalFsBackend;
