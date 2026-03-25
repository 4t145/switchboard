use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use axum::{
    extract::{Query, State},
    response::Response,
};

use crate::interface::http::HttpState;

#[derive(Debug, serde::Deserialize)]
pub struct GetEntryQuery {
    pub root: PathBuf,
    pub relative_path: Option<PathBuf>,
    #[serde(default)]
    pub list_children: bool,
    #[serde(default)]
    pub include_hidden: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct GetRootsResponse {
    pub roots: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryType {
    File,
    Directory,
    Symlink,
    Other,
}

#[derive(Debug, serde::Serialize)]
pub struct FsEntry {
    pub name: String,
    pub path: String,
    pub entry_type: EntryType,
    pub size: Option<u64>,
    pub modified_unix_ms: Option<u64>,
    pub readonly: Option<bool>,
    pub has_children: Option<bool>,
}

#[derive(Debug, serde::Serialize)]
pub struct GetEntryResponse {
    pub entry: FsEntry,
    pub children: Option<Vec<FsEntry>>,
}

pub async fn get_entry(
    State(state): State<HttpState>,
    Query(query): Query<GetEntryQuery>,
) -> Response {
    let process = async move {
        let checked_path =
            checked_path_in_allowed_roots(&state, &query.root, query.relative_path.as_deref())
                .await?;
        let entry = build_entry(&checked_path, query.include_hidden).await?;
        let children = if query.list_children && matches!(entry.entry_type, EntryType::Directory) {
            Some(list_children(&checked_path, query.include_hidden).await?)
        } else {
            None
        };
        Ok::<_, crate::Error>(GetEntryResponse { entry, children })
    };

    super::result_to_json_response(process.await)
}

pub async fn get_roots(State(state): State<HttpState>) -> Response {
    let process = async move {
        let roots = canonical_allowed_roots(&state)
            .await?
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect();
        Ok::<_, crate::Error>(GetRootsResponse { roots })
    };

    super::result_to_json_response(process.await)
}

/// # Errors
/// Returns an error when any configured allowed root cannot be canonicalized.
async fn canonical_allowed_roots(state: &HttpState) -> crate::Result<Vec<PathBuf>> {
    let mut roots = BTreeSet::new();
    for root in &state
        .controller_context
        .controller_config
        .file_browser
        .allowed_roots
    {
        let resolved_root = tokio::fs::canonicalize(root)
            .await
            .map_err(crate::Error::FileBrowserIoError)?;
        roots.insert(resolved_root);
    }
    Ok(roots.into_iter().collect())
}

/// # Errors
/// Returns an error when root/target paths cannot be canonicalized,
/// when `relative_path` is absolute, or when target path is outside allowed roots.
async fn checked_path_in_allowed_roots(
    state: &HttpState,
    root: &Path,
    relative_path: Option<&Path>,
) -> crate::Result<PathBuf> {
    if relative_path.is_some_and(Path::is_absolute) {
        return Err(crate::Error::FileBrowserPathNotAllowed(root.to_path_buf()));
    }
    let resolved_root = tokio::fs::canonicalize(root)
        .await
        .map_err(crate::Error::FileBrowserIoError)?;
    let target_path = match relative_path {
        Some(relative) => resolved_root.join(relative),
        None => resolved_root.clone(),
    };
    let resolved_path = tokio::fs::canonicalize(&target_path)
        .await
        .map_err(crate::Error::FileBrowserIoError)?;
    let mut has_allowed_root = false;
    for allowed_root in canonical_allowed_roots(state).await? {
        if resolved_path.starts_with(&allowed_root) && resolved_root.starts_with(&allowed_root) {
            has_allowed_root = true;
            break;
        }
    }
    if has_allowed_root {
        Ok(resolved_path)
    } else {
        Err(crate::Error::FileBrowserPathNotAllowed(resolved_path))
    }
}

/// # Errors
/// Returns an error when file metadata cannot be read, or when directory child probing fails.
async fn build_entry(path: &Path, include_hidden: bool) -> crate::Result<FsEntry> {
    let metadata = tokio::fs::symlink_metadata(path)
        .await
        .map_err(crate::Error::FileBrowserIoError)?;
    let entry_type = map_entry_type(&metadata.file_type());
    let file_name = path.file_name().map_or_else(
        || path.to_string_lossy().to_string(),
        |name| name.to_string_lossy().to_string(),
    );
    let has_children = if matches!(entry_type, EntryType::Directory) {
        Some(directory_has_children(path, include_hidden).await?)
    } else {
        None
    };
    let size = if matches!(entry_type, EntryType::File) {
        Some(metadata.len())
    } else {
        None
    };
    let modified_unix_ms = metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as u64);

    Ok(FsEntry {
        name: file_name,
        path: path.to_string_lossy().to_string(),
        entry_type,
        size,
        modified_unix_ms,
        readonly: Some(metadata.permissions().readonly()),
        has_children,
    })
}

/// # Errors
/// Returns an error when reading the directory or any child metadata fails.
async fn list_children(path: &Path, include_hidden: bool) -> crate::Result<Vec<FsEntry>> {
    let mut children = Vec::new();
    let mut read_dir = tokio::fs::read_dir(path)
        .await
        .map_err(crate::Error::FileBrowserIoError)?;
    while let Some(entry) = read_dir
        .next_entry()
        .await
        .map_err(crate::Error::FileBrowserIoError)?
    {
        let child_path = entry.path();
        if !include_hidden && is_hidden(&child_path) {
            continue;
        }
        children.push(build_entry(&child_path, include_hidden).await?);
    }
    children.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(children)
}

/// # Errors
/// Returns an error when reading the directory entries fails.
async fn directory_has_children(path: &Path, include_hidden: bool) -> crate::Result<bool> {
    let mut read_dir = tokio::fs::read_dir(path)
        .await
        .map_err(crate::Error::FileBrowserIoError)?;
    while let Some(entry) = read_dir
        .next_entry()
        .await
        .map_err(crate::Error::FileBrowserIoError)?
    {
        if include_hidden || !is_hidden(&entry.path()) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| name.to_string_lossy().starts_with('.'))
}

fn map_entry_type(file_type: &std::fs::FileType) -> EntryType {
    if file_type.is_dir() {
        EntryType::Directory
    } else if file_type.is_file() {
        EntryType::File
    } else if file_type.is_symlink() {
        EntryType::Symlink
    } else {
        EntryType::Other
    }
}

pub fn router() -> axum::Router<HttpState> {
    axum::Router::new()
        .route("/entry", axum::routing::get(get_entry))
        .route("/roots", axum::routing::get(get_roots))
}
