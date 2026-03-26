use std::path::{Component, Path, PathBuf};

use tokio::{fs, io};

pub fn resolve_relative_path_from_uri_path(uri_path: &str) -> Option<PathBuf> {
    let trimmed = uri_path.trim_start_matches('/');
    let mut rel = PathBuf::new();

    for comp in Path::new(trimmed).components() {
        match comp {
            Component::Normal(seg) => rel.push(seg),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }

    Some(rel)
}

pub async fn ensure_within_root(root: &Path, path: &Path) -> io::Result<Option<PathBuf>> {
    let root_canonical = fs::canonicalize(root).await?;
    let target_canonical = match fs::canonicalize(path).await {
        Ok(target) => target,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    if target_canonical.starts_with(&root_canonical) {
        Ok(Some(target_canonical))
    } else {
        Ok(None)
    }
}
