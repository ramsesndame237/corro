// All types and trait methods here are part of the public storage API contract.
// They will be fully exercised starting from issue #7 (FilesystemBackend)
// and issues #13–#32 (S3 HTTP handlers).
#![allow(dead_code)]

use async_trait::async_trait;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::CorroError;

pub type ETag = String;

/// Metadata associated with a stored object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMetadata {
    pub size: u64,
    pub etag: ETag,
    pub last_modified: DateTime<Utc>,
    pub content_type: String,
}

/// Summary of a bucket returned in list responses.
#[derive(Debug, Clone)]
pub struct BucketInfo {
    pub name: String,
    pub creation_date: DateTime<Utc>,
}

/// Parameters for listing objects in a bucket (ListObjectsV2 semantics).
#[derive(Debug, Default)]
pub struct ListObjectsParams {
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub max_keys: Option<usize>,
    pub continuation_token: Option<String>,
    pub start_after: Option<String>,
}

/// A single object entry in a list response.
#[derive(Debug, Clone)]
pub struct ObjectInfo {
    pub key: String,
    pub size: u64,
    pub etag: ETag,
    pub last_modified: DateTime<Utc>,
}

/// Result of a ListObjectsV2 operation.
#[derive(Debug, Default)]
pub struct ListObjectsResult {
    pub objects: Vec<ObjectInfo>,
    /// Virtual "folder" prefixes when a delimiter is used.
    pub common_prefixes: Vec<String>,
    pub is_truncated: bool,
    pub next_continuation_token: Option<String>,
    pub key_count: usize,
}

/// The core storage abstraction. All storage backends implement this trait.
///
/// Implementations must be `Send + Sync` to be shared across async tasks.
#[async_trait]
pub trait StorageBackend: Send + Sync {
    // ── Bucket operations ──────────────────────────────────────────────────

    async fn create_bucket(&self, bucket: &str) -> Result<(), CorroError>;

    async fn delete_bucket(&self, bucket: &str) -> Result<(), CorroError>;

    async fn bucket_exists(&self, bucket: &str) -> Result<bool, CorroError>;

    async fn list_buckets(&self) -> Result<Vec<BucketInfo>, CorroError>;

    // ── Object operations ──────────────────────────────────────────────────

    async fn put_object(
        &self,
        bucket: &str,
        key: &str,
        data: Bytes,
        content_type: String,
    ) -> Result<ETag, CorroError>;

    async fn get_object(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<(Bytes, ObjectMetadata), CorroError>;

    async fn delete_object(&self, bucket: &str, key: &str) -> Result<(), CorroError>;

    async fn head_object(&self, bucket: &str, key: &str) -> Result<ObjectMetadata, CorroError>;

    async fn list_objects(
        &self,
        bucket: &str,
        params: ListObjectsParams,
    ) -> Result<ListObjectsResult, CorroError>;
}

// ── Null backend (stub used until #7 lands) ───────────────────────────────

/// A no-op backend that returns `NotImplemented` for every operation.
/// Used as a placeholder until the filesystem backend (#7) is implemented.
pub struct NullBackend;

#[async_trait]
impl StorageBackend for NullBackend {
    async fn create_bucket(&self, _bucket: &str) -> Result<(), CorroError> {
        Err(CorroError::NotImplemented)
    }
    async fn delete_bucket(&self, _bucket: &str) -> Result<(), CorroError> {
        Err(CorroError::NotImplemented)
    }
    async fn bucket_exists(&self, _bucket: &str) -> Result<bool, CorroError> {
        Err(CorroError::NotImplemented)
    }
    async fn list_buckets(&self) -> Result<Vec<BucketInfo>, CorroError> {
        Err(CorroError::NotImplemented)
    }
    async fn put_object(
        &self,
        _bucket: &str,
        _key: &str,
        _data: Bytes,
        _content_type: String,
    ) -> Result<ETag, CorroError> {
        Err(CorroError::NotImplemented)
    }
    async fn get_object(
        &self,
        _bucket: &str,
        _key: &str,
    ) -> Result<(Bytes, ObjectMetadata), CorroError> {
        Err(CorroError::NotImplemented)
    }
    async fn delete_object(&self, _bucket: &str, _key: &str) -> Result<(), CorroError> {
        Err(CorroError::NotImplemented)
    }
    async fn head_object(&self, _bucket: &str, _key: &str) -> Result<ObjectMetadata, CorroError> {
        Err(CorroError::NotImplemented)
    }
    async fn list_objects(
        &self,
        _bucket: &str,
        _params: ListObjectsParams,
    ) -> Result<ListObjectsResult, CorroError> {
        Err(CorroError::NotImplemented)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn null_backend_returns_not_implemented() {
        let backend = NullBackend;
        let result = backend.list_buckets().await;
        assert!(matches!(result, Err(CorroError::NotImplemented)));
    }

    #[tokio::test]
    async fn null_backend_put_returns_not_implemented() {
        let backend = NullBackend;
        let result = backend
            .put_object("bucket", "key", Bytes::new(), "text/plain".into())
            .await;
        assert!(matches!(result, Err(CorroError::NotImplemented)));
    }
}
