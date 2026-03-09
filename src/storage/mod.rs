// These types are the public API contract of the storage layer.
// They will be actively used starting from issue #7 (FilesystemBackend)
// and issues #13–#32 (HTTP handlers).
pub mod backend;

#[allow(unused_imports)]
pub use backend::{
    BucketInfo, ETag, ListObjectsParams, ListObjectsResult, NullBackend, ObjectInfo,
    ObjectMetadata, StorageBackend,
};
