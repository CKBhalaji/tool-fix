//! Local filesystem storage backend.

use async_trait::async_trait;
use bytes::Bytes;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use super::{validate_key, StorageBackend, StorageError, StoredObject};

/// Stores objects under `<root>/<key>` after sanitizing the key. Parent
/// directories are created lazily.
pub struct LocalFsBackend {
    root: std::path::PathBuf,
}

impl LocalFsBackend {
    pub fn new(root: impl Into<std::path::PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn resolve(&self, key: &str) -> Result<std::path::PathBuf, StorageError> {
        validate_key(key)?;
        let path = self.root.join(key);
        // Defense in depth: the canonical path must stay under root.
        let root_canonical =
            std::fs::canonicalize(&self.root).unwrap_or_else(|_| self.root.clone());
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let Ok(canonical) = std::fs::canonicalize(&path) else {
            return Ok(path);
        };
        if !canonical.starts_with(&root_canonical) {
            return Err(StorageError::InvalidKey(key.to_string()));
        }
        Ok(canonical)
    }
}

#[async_trait]
impl StorageBackend for LocalFsBackend {
    async fn put(
        &self,
        key: &str,
        content_type: Option<&str>,
        data: Bytes,
    ) -> Result<StoredObject, StorageError> {
        validate_key(key)?;
        let path = self.root.join(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| StorageError::Backend(e.to_string()))?;
        }
        let mut file = fs::File::create(&path)
            .await
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        file.write_all(&data)
            .await
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        file.flush()
            .await
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        tracing::debug!(key = %key, size = data.len(), "stored object");
        Ok(StoredObject {
            key: key.to_string(),
            content_type: content_type.map(str::to_string),
            size_bytes: data.len() as u64,
        })
    }

    async fn get(&self, key: &str) -> Result<Bytes, StorageError> {
        let path = self.resolve(key)?;
        match fs::read(&path).await {
            Ok(data) => Ok(Bytes::from(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Err(StorageError::NotFound(key.to_string()))
            }
            Err(e) => Err(StorageError::Backend(e.to_string())),
        }
    }

    async fn delete(&self, key: &str) -> Result<(), StorageError> {
        let path = self.resolve(key)?;
        match fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(StorageError::Backend(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn put_get_delete_roundtrip() {
        let backend = LocalFsBackend::new(std::env::temp_dir().join(format!(
            "toolfix-storage-test-{}",
            uuid::Uuid::now_v7()
        )));
        let key = "breakdowns/abc/photo.jpg";
        backend
            .put(key, Some("image/jpeg"), Bytes::from_static(b"hello"))
            .await
            .unwrap();
        let data = backend.get(key).await.unwrap();
        assert_eq!(&data[..], b"hello");

        backend.delete(key).await.unwrap();
        assert!(matches!(
            backend.get(key).await,
            Err(StorageError::NotFound(_))
        ));
        let _ = fs::remove_dir_all(&backend.root).await;
    }

    #[tokio::test]
    async fn rejects_traversal_keys() {
        let backend = LocalFsBackend::new(std::env::temp_dir().join("toolfix-storage-test"));
        assert!(matches!(
            backend.get("../escape").await,
            Err(StorageError::InvalidKey(_))
        ));
        assert!(matches!(
            backend.put("/abs", None, Bytes::new()).await,
            Err(StorageError::InvalidKey(_))
        ));
    }
}
