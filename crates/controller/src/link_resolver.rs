use std::path::PathBuf;

use http::Uri;

use crate::storage::StorageObjectDescriptor;

pub struct SuperResolver;

pub enum Link {
    FilePath(PathBuf),
    Http(Uri),
    Storage(StorageObjectDescriptor),
}
