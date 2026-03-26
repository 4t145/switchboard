use std::path::{Path, PathBuf};
use std::time::SystemTime;

use tokio::{fs, io};

pub enum GeneratedIndexItemType {
    Dir,
    Link,
    File,
}

pub struct GeneratedIndexItem {
    pub item_type: GeneratedIndexItemType,
    pub relative_path: PathBuf,
    pub modified_at: Option<SystemTime>,
    pub created_at: Option<SystemTime>,
    pub size: Option<usize>,
    pub file_name: String,
}

pub struct GeneratedIndex {
    items: Vec<GeneratedIndexItem>,
}

impl GeneratedIndex {
    pub async fn from_dir(
        mut rd: fs::ReadDir,
        current_relative_path: &Path,
        display_size: bool,
        display_time: bool,
    ) -> io::Result<Self> {
        let mut items = Vec::new();

        while let Some(child) = rd.next_entry().await? {
            let file_type = child.file_type().await?;
            let metadata = child.metadata().await?;

            let item_type = if file_type.is_dir() {
                GeneratedIndexItemType::Dir
            } else if file_type.is_symlink() {
                GeneratedIndexItemType::Link
            } else {
                GeneratedIndexItemType::File
            };

            let size = if file_type.is_file() && display_size {
                usize::try_from(metadata.len()).ok()
            } else {
                None
            };

            let file_name = child.file_name().to_string_lossy().into_owned();
            let (modified_at, created_at) = if display_time {
                (metadata.modified().ok(), metadata.created().ok())
            } else {
                (None, None)
            };
            let relative_path = current_relative_path.join(&file_name);

            items.push(GeneratedIndexItem {
                item_type,
                relative_path,
                modified_at,
                created_at,
                size,
                file_name,
            });
        }

        items.sort_by(|a, b| {
            let rank = |t: &GeneratedIndexItemType| match t {
                GeneratedIndexItemType::Dir => 0u8,
                GeneratedIndexItemType::Link => 1u8,
                GeneratedIndexItemType::File => 2u8,
            };
            rank(&a.item_type)
                .cmp(&rank(&b.item_type))
                .then_with(|| a.file_name.cmp(&b.file_name))
        });

        Ok(Self { items })
    }

    pub fn render_html(&self) -> String {
        let mut html = String::from(
            "<!doctype html><html><head><meta charset=\"utf-8\"><title>Index</title></head><body><h1>Index</h1><table><thead><tr><th>Name</th><th>Last Modified</th><th>Created At</th><th>Size</th></tr></thead><tbody>",
        );
        for item in &self.items {
            let suffix = match item.item_type {
                GeneratedIndexItemType::Dir => "/",
                _ => "",
            };
            let href = Self::to_href_path(
                &item.relative_path,
                matches!(item.item_type, GeneratedIndexItemType::Dir),
            );
            let modified = item
                .modified_at
                .map(httpdate::fmt_http_date)
                .unwrap_or_else(|| "-".to_string());
            let created = item
                .created_at
                .map(httpdate::fmt_http_date)
                .unwrap_or_else(|| "-".to_string());
            let size = item
                .size
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string());
            let _ = std::fmt::Write::write_fmt(
                &mut html,
                format_args!(
                    "<tr><td><a href=\"{}\">{}{}</a></td><td>{}</td><td>{}</td><td>{}</td></tr>",
                    href,
                    Self::escape_html(&item.file_name),
                    suffix,
                    modified,
                    created,
                    size,
                ),
            );
        }
        html.push_str("</tbody></table></body></html>");
        html
    }

    fn to_href_path(relative_path: &Path, is_dir: bool) -> String {
        let mut href = String::from("/");
        let mut first = true;
        for segment in relative_path {
            if !first {
                href.push('/');
            }
            href.push_str(&segment.to_string_lossy());
            first = false;
        }
        if is_dir {
            href.push('/');
        }
        href
    }

    fn escape_html(input: &str) -> String {
        input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }
}
