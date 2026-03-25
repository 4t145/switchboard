use std::{
    collections::HashMap,
    sync::{Arc, atomic::{AtomicU64, Ordering}},
    time::Duration,
};

use http::StatusCode;
use serde::{Deserialize, Serialize};
use switchboard_model::services::http::ClassId;
use tokio::sync::RwLock;

use crate::{
    DynRequest, DynResponse,
    consts::ERR_FILTER_REQUEST_RATE_LIMIT,
    flow::filter::{FilterClass, FilterLike},
    utils::{error_response, token_bucket::TokenBucket},
};

const DEFAULT_CAPACITY: usize = 100;
const DEFAULT_RATE: Duration = Duration::from_millis(10);
const DEFAULT_IDLE_TTL: Duration = Duration::from_secs(300);
const DEFAULT_CLEANUP_INTERVAL: Duration = Duration::from_secs(60);
const DEFAULT_STATUS_CODE: u16 = 429;
const UNKNOWN_KEY: &str = "unknown";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct RequestRateLimitFilterConfig {
    pub capacity: usize,
    #[serde(with = "crate::utils::duration_expr")]
    pub rate: Duration,
    #[serde(with = "crate::utils::duration_expr")]
    pub idle_ttl: Duration,
    #[serde(with = "crate::utils::duration_expr")]
    pub cleanup_interval: Duration,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_template: Option<String>,
    pub status_code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl Default for RequestRateLimitFilterConfig {
    fn default() -> Self {
        Self {
            capacity: DEFAULT_CAPACITY,
            rate: DEFAULT_RATE,
            idle_ttl: DEFAULT_IDLE_TTL,
            cleanup_interval: DEFAULT_CLEANUP_INTERVAL,
            key_template: None,
            status_code: DEFAULT_STATUS_CODE,
            message: None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RequestRateLimitFilterConfigError {
    #[error("Invalid status code: {0}")]
    InvalidStatusCode(#[from] http::status::InvalidStatusCode),
    #[error("capacity must be greater than 0")]
    InvalidCapacity,
    #[error("rate must be greater than 0")]
    InvalidRate,
    #[error("cleanup_interval must be greater than 0")]
    InvalidCleanupInterval,
}

struct BucketEntry {
    bucket: TokenBucket,
    last_seen_ms: AtomicU64,
}

impl BucketEntry {
    fn new(now: tokio::time::Instant, capacity: usize, rate: Duration, now_ms: u64) -> Self {
        let _ = now;
        Self {
            bucket: TokenBucket::new(capacity, rate),
            last_seen_ms: AtomicU64::new(now_ms),
        }
    }

    async fn require(&self, now: tokio::time::Instant, now_ms: u64) -> bool {
        self.last_seen_ms.store(now_ms, Ordering::Relaxed);
        self.bucket.require(now).await.is_some()
    }
}

struct BucketTable {
    entries: HashMap<String, Arc<BucketEntry>>,
    last_cleanup_ms: u64,
}

pub struct RequestRateLimitFilter {
    pub capacity: usize,
    pub rate: Duration,
    pub idle_ttl: Duration,
    pub cleanup_interval: Duration,
    pub status_code: StatusCode,
    pub message: Option<String>,
    pub key_template: Option<String>,
    table: Arc<RwLock<BucketTable>>,
    started_at: tokio::time::Instant,
}

impl RequestRateLimitFilter {
    fn now_ms(&self, now: tokio::time::Instant) -> u64 {
        now.saturating_duration_since(self.started_at).as_millis() as u64
    }

    fn key_for_request(parts: &http::request::Parts, ctx: &crate::flow::FlowContext) -> String {
        let _ = parts;
        let _ = ctx;
        // TODO: support custom key template, currently fallback to client IP.
        ctx.connection_info
            .as_ref()
            .map(|info| info.peer_addr.ip().to_string())
            .unwrap_or_else(|| UNKNOWN_KEY.to_string())
    }

    async fn cleanup_if_needed(self: &Arc<Self>, now_ms: u64) {
        let need_cleanup = {
            let table = self.table.read().await;
            now_ms.saturating_sub(table.last_cleanup_ms) >= self.cleanup_interval.as_millis() as u64
        };
        if !need_cleanup {
            return;
        }

        let idle_ttl_ms = self.idle_ttl.as_millis() as u64;
        let mut table = self.table.write().await;
        if now_ms.saturating_sub(table.last_cleanup_ms) < self.cleanup_interval.as_millis() as u64 {
            return;
        }
        table.entries.retain(|_, entry| {
            let last_seen = entry.last_seen_ms.load(Ordering::Relaxed);
            now_ms.saturating_sub(last_seen) <= idle_ttl_ms
        });
        table.last_cleanup_ms = now_ms;
    }

    async fn bucket_for_key(self: &Arc<Self>, key: String, now: tokio::time::Instant, now_ms: u64) -> Arc<BucketEntry> {
        if let Some(entry) = self.table.read().await.entries.get(&key).cloned() {
            return entry;
        }
        let mut table = self.table.write().await;
        table
            .entries
            .entry(key)
            .or_insert_with(|| Arc::new(BucketEntry::new(now, self.capacity, self.rate, now_ms)))
            .clone()
    }

    async fn try_take_token(self: &Arc<Self>, key: String, now: tokio::time::Instant) -> bool {
        let now_ms = self.now_ms(now);
        self.cleanup_if_needed(now_ms).await;
        let entry = self.bucket_for_key(key, now, now_ms).await;
        entry.require(now, now_ms).await
    }
}

impl FilterLike for RequestRateLimitFilter {
    async fn call(
        self: Arc<Self>,
        req: DynRequest,
        ctx: &mut crate::flow::FlowContext,
        next: super::Next,
    ) -> DynResponse {
        let (parts, body) = req.into_parts();
        let key = Self::key_for_request(&parts, ctx);
        let allowed = self.try_take_token(key, tokio::time::Instant::now()).await;
        if !allowed {
            return error_response(
                self.status_code,
                self.message
                    .as_deref()
                    .unwrap_or("rate limit exceeded, please retry later"),
                ERR_FILTER_REQUEST_RATE_LIMIT,
            );
        }
        let req = DynRequest::from_parts(parts, body);
        next.call(req, ctx).await
    }
}

pub struct RequestRateLimitFilterClass;

impl FilterClass for RequestRateLimitFilterClass {
    type Filter = RequestRateLimitFilter;
    type Error = RequestRateLimitFilterConfigError;
    type Config = RequestRateLimitFilterConfig;

    fn id(&self) -> ClassId {
        ClassId::std("request-rate-limit")
    }

    fn construct(&self, config: Self::Config) -> Result<Self::Filter, Self::Error> {
        if config.capacity == 0 {
            return Err(RequestRateLimitFilterConfigError::InvalidCapacity);
        }
        if config.rate.is_zero() {
            return Err(RequestRateLimitFilterConfigError::InvalidRate);
        }
        if config.cleanup_interval.is_zero() {
            return Err(RequestRateLimitFilterConfigError::InvalidCleanupInterval);
        }
        let status_code = StatusCode::from_u16(config.status_code)?;
        let started_at = tokio::time::Instant::now();
        Ok(RequestRateLimitFilter {
            capacity: config.capacity,
            rate: config.rate,
            idle_ttl: config.idle_ttl,
            cleanup_interval: config.cleanup_interval,
            status_code,
            message: config.message,
            key_template: config.key_template,
            table: Arc::new(RwLock::new(BucketTable {
                entries: HashMap::new(),
                last_cleanup_ms: 0,
            })),
            started_at,
        })
    }
}
