mod cache;
mod index;
mod path;
mod response;

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::dynamic::stream_body;
use crate::utils::one_or_many::OneOrMany;
use crate::{DynRequest, DynResponse, bytes_body, empty_body};
use crate::{
    consts::ERR_STATIC_FILE,
    flow::{FlowContext, node::NodeClass, service::ServiceNode},
    utils::error_response,
};
use http::header::{
    ALLOW, CACHE_CONTROL, CONTENT_DISPOSITION, CONTENT_TYPE, ETAG, IF_NONE_MATCH, LAST_MODIFIED,
};

use http::{HeaderValue, Method, StatusCode};
use switchboard_model::services::http::{ClassId, consts::STATIC_FILE_CLASS_ID};
use tokio::{fs, io};

use self::cache::{
    build_cache_control_value, if_modified_since_hit, if_none_match_hit, make_etag,
    select_cache_policy,
};
use self::index::GeneratedIndex;
use self::path::{ensure_within_root, resolve_relative_path_from_uri_path};
use self::response::{empty_response, forbidden, not_found, not_modified_from};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum DirWithoutIndexPolicy {
    #[default]
    Forbidden,
    NotFound,
}

impl DirWithoutIndexPolicy {
    pub fn as_response(&self) -> DynResponse {
        match self {
            Self::Forbidden => forbidden(),
            Self::NotFound => not_found(),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum FileResponseMode {
    #[default]
    Inline,
    Attachment,
    Deny,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ResponseConfig {
    pub index: OneOrMany<PathBuf>,
    pub allow_symlink: bool,
    pub auto_index: bool,
    pub auto_index_exact_size: bool,
    pub auto_index_localtime: bool,
    pub dir_without_index_policy: DirWithoutIndexPolicy,
    pub mode_by_extension: BTreeMap<String, FileResponseMode>,
    pub mode_default: FileResponseMode,
}

impl Default for ResponseConfig {
    fn default() -> Self {
        Self {
            index: OneOrMany::One(PathBuf::from("index.html")),
            allow_symlink: false,
            auto_index: false,
            auto_index_exact_size: false,
            auto_index_localtime: false,
            dir_without_index_policy: DirWithoutIndexPolicy::Forbidden,
            mode_by_extension: BTreeMap::new(),
            mode_default: FileResponseMode::Inline,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct CacheConfig {
    pub enabled: bool,
    pub etag: EtagConfig,
    pub last_modified: bool,
    pub conditional_get: bool,
    pub default_policy: CachePolicy,
    pub rules: Vec<CacheRule>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            etag: EtagConfig::default(),
            last_modified: true,
            conditional_get: true,
            default_policy: CachePolicy {
                mode: CacheMode::NoCache,
                ..CachePolicy::default()
            },
            rules: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct EtagConfig {
    pub enabled: bool,
    pub weak: bool,
}

impl Default for EtagConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            weak: true,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct CachePolicy {
    pub mode: CacheMode,
    pub max_age_seconds: Option<u32>,
    pub s_maxage_seconds: Option<u32>,
    pub stale_while_revalidate_seconds: Option<u32>,
    pub stale_if_error_seconds: Option<u32>,
    pub must_revalidate: bool,
    pub proxy_revalidate: bool,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum CacheMode {
    #[default]
    Public,
    Private,
    NoCache,
    NoStore,
    Immutable,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct CacheRule {
    pub ext: Option<Vec<String>>,
    pub path_prefix: Option<String>,
    pub content_type_prefix: Option<String>,
    pub policy: CachePolicy,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct MimeSniffConfig {
    pub default_type: String,
    pub by_extension: BTreeMap<String, String>,
}

impl Default for MimeSniffConfig {
    fn default() -> Self {
        Self {
            default_type: "application/octet-stream".to_string(),
            by_extension: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct StaticFileServiceConfig {
    pub root: PathBuf,
    pub mime_sniff: Option<MimeSniffConfig>,
    pub response: ResponseConfig,
    pub cache: CacheConfig,
}

#[derive(Debug, Clone)]
pub struct MimeSniff {
    pub default_type: mime::Mime,
    pub by_extension: BTreeMap<String, mime::Mime>,
}

#[derive(Debug, Clone)]
pub struct StaticFileService {
    pub root: Arc<Path>,
    pub mime_sniff: Option<MimeSniff>,
    pub response: ResponseConfig,
    pub cache: CacheConfig,
}

#[derive(Debug, thiserror::Error)]
pub enum StaticFileServiceConfigError {
    #[error("root path must not be empty")]
    EmptyRoot,
    #[error("invalid default mime type: {0}")]
    InvalidDefaultMime(#[source] mime::FromStrError),
    #[error("invalid mime type for extension `{ext}`: {source}")]
    InvalidExtensionMime {
        ext: String,
        #[source]
        source: mime::FromStrError,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum StaticFileServiceError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("invalid header value: {0}")]
    HeaderValue(#[from] http::header::InvalidHeaderValue),
}

impl StaticFileService {
    fn file_mode_for_extension(&self, ext: Option<&str>) -> FileResponseMode {
        let Some(ext) = ext else {
            return self.response.mode_default.clone();
        };
        let key = ext.to_ascii_lowercase();
        self.response
            .mode_by_extension
            .get(&key)
            .cloned()
            .unwrap_or_else(|| self.response.mode_default.clone())
    }

    fn make_content_disposition_attachment(path: &Path) -> Option<String> {
        let name = path.file_name()?.to_str()?;
        let escaped = name.replace('"', "\\\"");
        Some(format!("attachment; filename=\"{escaped}\""))
    }

    pub async fn fetch_file(
        &self,
        relative_path: PathBuf,
        parts: &http::request::Parts,
    ) -> Result<DynResponse, StaticFileServiceError> {
        use tokio_util::io::ReaderStream;

        let return_body = parts.method == Method::GET;
        let candidate_path = self.root.join(&relative_path);
        let Some(resource_path) = ensure_within_root(self.root.as_ref(), &candidate_path).await?
        else {
            return Ok(not_found());
        };

        let symlink_meta = fs::symlink_metadata(&resource_path).await?;
        if symlink_meta.file_type().is_symlink() && !self.response.allow_symlink {
            return Ok(not_found());
        }

        let metadata = fs::metadata(&resource_path).await?;
        let file_type = metadata.file_type();
        let modified = metadata.modified()?;

        let mut response = if file_type.is_file() {
            let ext = resource_path
                .extension()
                .and_then(OsStr::to_str)
                .map(str::to_ascii_lowercase);
            match self.file_mode_for_extension(ext.as_deref()) {
                FileResponseMode::Deny => {
                    return Ok(self.response.dir_without_index_policy.as_response());
                }
                FileResponseMode::Inline | FileResponseMode::Attachment => {
                    let body = if return_body {
                        let file = fs::OpenOptions::new()
                            .read(true)
                            .open(&resource_path)
                            .await?;
                        stream_body(ReaderStream::new(file))
                    } else {
                        empty_body()
                    };
                    let mut response = DynResponse::new(body);

                    if let Some(mime_sniff) = &self.mime_sniff {
                        let mime = ext
                            .as_deref()
                            .and_then(|name| mime_sniff.by_extension.get(name))
                            .unwrap_or(&mime_sniff.default_type);
                        response
                            .headers_mut()
                            .insert(CONTENT_TYPE, HeaderValue::from_str(mime.essence_str())?);
                    }

                    if matches!(
                        self.file_mode_for_extension(ext.as_deref()),
                        FileResponseMode::Attachment
                    ) && let Some(content_disposition) =
                        Self::make_content_disposition_attachment(&resource_path)
                    {
                        response.headers_mut().insert(
                            CONTENT_DISPOSITION,
                            HeaderValue::from_str(&content_disposition)?,
                        );
                    }

                    response
                }
            }
        } else if file_type.is_dir() {
            for index in &self.response.index {
                let index_rel = relative_path.join(index);
                let index_abs = self.root.join(&index_rel);
                if fs::try_exists(&index_abs).await? {
                    return Box::pin(self.fetch_file(index_rel, parts)).await;
                }
            }
            if self.response.auto_index {
                let entry = GeneratedIndex::from_dir(
                    fs::read_dir(&resource_path).await?,
                    &relative_path,
                    self.response.auto_index_exact_size,
                    self.response.auto_index_localtime,
                )
                .await?;
                let body = if return_body {
                    bytes_body(entry.render_html())
                } else {
                    empty_body()
                };
                let mut response = DynResponse::new(body);
                response.headers_mut().insert(
                    CONTENT_TYPE,
                    HeaderValue::from_static("text/html; charset=utf-8"),
                );
                response
            } else {
                self.response.dir_without_index_policy.as_response()
            }
        } else {
            not_found()
        };

        if self.cache.enabled {
            let request_path = parts.uri.path();
            let ext = resource_path
                .extension()
                .and_then(OsStr::to_str)
                .map(str::to_ascii_lowercase);
            let content_type = response
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|v| v.to_str().ok());
            let policy =
                select_cache_policy(&self.cache, request_path, ext.as_deref(), content_type);

            let cache_control = build_cache_control_value(policy);
            if !cache_control.is_empty() {
                response
                    .headers_mut()
                    .insert(CACHE_CONTROL, HeaderValue::from_str(&cache_control)?);
            }

            let etag_value = if self.cache.etag.enabled {
                let etag = make_etag(self.cache.etag.weak, modified, metadata.len());
                response
                    .headers_mut()
                    .insert(ETAG, HeaderValue::from_str(&etag)?);
                Some(etag)
            } else {
                None
            };

            if self.cache.last_modified {
                let last_modified = httpdate::fmt_http_date(modified);
                response
                    .headers_mut()
                    .insert(LAST_MODIFIED, HeaderValue::from_str(&last_modified)?);
            }

            if self.cache.conditional_get && file_type.is_file() {
                let has_inm = parts.headers.contains_key(IF_NONE_MATCH);
                let inm_hit = etag_value
                    .as_deref()
                    .is_some_and(|etag| if_none_match_hit(parts, etag));
                let ims_hit = if_modified_since_hit(parts, modified);

                if if has_inm { inm_hit } else { ims_hit } {
                    return Ok(not_modified_from(&response));
                }
            }
        }

        Ok(response)
    }

    pub async fn call_inner(&self, req: DynRequest) -> Result<DynResponse, StaticFileServiceError> {
        let (parts, _body) = req.into_parts();
        if parts.method == Method::GET || parts.method == Method::HEAD {
            let Some(relative_path) = resolve_relative_path_from_uri_path(parts.uri.path()) else {
                return Ok(not_found());
            };
            self.fetch_file(relative_path, &parts).await
        } else {
            let mut response = if parts.method == Method::OPTIONS {
                empty_response(StatusCode::NO_CONTENT)
            } else if [
                Method::CONNECT,
                Method::DELETE,
                Method::PATCH,
                Method::POST,
                Method::PUT,
                Method::TRACE,
            ]
            .contains(&parts.method)
            {
                empty_response(StatusCode::METHOD_NOT_ALLOWED)
            } else {
                empty_response(StatusCode::NOT_IMPLEMENTED)
            };
            response
                .headers_mut()
                .insert(ALLOW, HeaderValue::from_static("GET, HEAD, OPTIONS"));
            Ok(response)
        }
    }
}

impl super::Service for StaticFileService {
    fn call<'c>(
        &self,
        req: DynRequest,
        _: &'c mut FlowContext,
    ) -> impl Future<Output = DynResponse> + Send + 'c {
        let this = self.clone();
        async move {
            match this.call_inner(req).await {
                Ok(response) => response,
                Err(e) => {
                    if let StaticFileServiceError::IoError(e) = &e {
                        if e.kind() == io::ErrorKind::NotFound {
                            return not_found();
                        }
                    }
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, e, ERR_STATIC_FILE)
                }
            }
        }
    }
}

pub struct StaticFileClass;

impl NodeClass for StaticFileClass {
    type Config = StaticFileServiceConfig;
    type Error = StaticFileServiceConfigError;
    type Node = ServiceNode<StaticFileService>;

    fn construct(&self, config: Self::Config) -> Result<Self::Node, Self::Error> {
        if config.root.as_os_str().is_empty() {
            return Err(StaticFileServiceConfigError::EmptyRoot);
        }

        let mime_sniff = if let Some(raw) = config.mime_sniff {
            let default_type = raw
                .default_type
                .parse()
                .map_err(StaticFileServiceConfigError::InvalidDefaultMime)?;
            let mut by_extension = BTreeMap::new();
            for (ext, mime_raw) in raw.by_extension {
                let mime = mime_raw.parse().map_err(|source| {
                    StaticFileServiceConfigError::InvalidExtensionMime {
                        ext: ext.clone(),
                        source,
                    }
                })?;
                by_extension.insert(ext.to_ascii_lowercase(), mime);
            }
            Some(MimeSniff {
                default_type,
                by_extension,
            })
        } else {
            None
        };

        Ok(ServiceNode::new(StaticFileService {
            root: Arc::<Path>::from(config.root.into_boxed_path()),
            mime_sniff,
            response: config.response,
            cache: config.cache,
        }))
    }

    fn id(&self) -> ClassId {
        ClassId::std(STATIC_FILE_CLASS_ID)
    }
}
