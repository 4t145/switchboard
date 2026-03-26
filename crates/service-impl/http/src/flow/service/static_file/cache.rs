use std::fmt::Write;
use std::time::SystemTime;

use http::header::{IF_MODIFIED_SINCE, IF_NONE_MATCH};
use http::request::Parts;

use super::{CacheConfig, CacheMode, CachePolicy, CacheRule};

const IMMUTABLE_DEFAULT_MAX_AGE_SECONDS: u32 = 31_536_000;

fn cache_rule_matches(
    rule: &CacheRule,
    request_path: &str,
    ext: Option<&str>,
    content_type: Option<&str>,
) -> bool {
    if let Some(prefix) = &rule.path_prefix
        && !request_path.starts_with(prefix)
    {
        return false;
    }

    if let Some(exts) = &rule.ext {
        let Some(ext) = ext else {
            return false;
        };
        if !exts.iter().any(|item| item.eq_ignore_ascii_case(ext)) {
            return false;
        }
    }

    if let Some(prefix) = &rule.content_type_prefix {
        let Some(content_type) = content_type else {
            return false;
        };
        if !content_type
            .to_ascii_lowercase()
            .starts_with(&prefix.to_ascii_lowercase())
        {
            return false;
        }
    }

    true
}

pub fn select_cache_policy<'a>(
    cache: &'a CacheConfig,
    request_path: &str,
    ext: Option<&str>,
    content_type: Option<&str>,
) -> &'a CachePolicy {
    for rule in &cache.rules {
        if cache_rule_matches(rule, request_path, ext, content_type) {
            return &rule.policy;
        }
    }
    &cache.default_policy
}

fn append_cc_token(buf: &mut String, first: &mut bool, token: &str) {
    if !*first {
        let _ = write!(buf, ", ");
    }
    let _ = write!(buf, "{token}");
    *first = false;
}

fn append_cc_kv(buf: &mut String, first: &mut bool, key: &str, value: u32) {
    if !*first {
        let _ = write!(buf, ", ");
    }
    let _ = write!(buf, "{key}={value}");
    *first = false;
}

pub fn build_cache_control_value(policy: &CachePolicy) -> String {
    let mut out = String::with_capacity(96);
    let mut first = true;

    match policy.mode {
        CacheMode::NoStore => append_cc_token(&mut out, &mut first, "no-store"),
        CacheMode::NoCache => append_cc_token(&mut out, &mut first, "no-cache"),
        CacheMode::Public => append_cc_token(&mut out, &mut first, "public"),
        CacheMode::Private => append_cc_token(&mut out, &mut first, "private"),
        CacheMode::Immutable => {
            append_cc_token(&mut out, &mut first, "public");
            append_cc_token(&mut out, &mut first, "immutable");
            if policy.max_age_seconds.is_none() {
                append_cc_kv(
                    &mut out,
                    &mut first,
                    "max-age",
                    IMMUTABLE_DEFAULT_MAX_AGE_SECONDS,
                );
            }
        }
    }

    if let Some(v) = policy.max_age_seconds {
        append_cc_kv(&mut out, &mut first, "max-age", v);
    }
    if let Some(v) = policy.s_maxage_seconds {
        append_cc_kv(&mut out, &mut first, "s-maxage", v);
    }
    if let Some(v) = policy.stale_while_revalidate_seconds {
        append_cc_kv(&mut out, &mut first, "stale-while-revalidate", v);
    }
    if let Some(v) = policy.stale_if_error_seconds {
        append_cc_kv(&mut out, &mut first, "stale-if-error", v);
    }
    if policy.must_revalidate {
        append_cc_token(&mut out, &mut first, "must-revalidate");
    }
    if policy.proxy_revalidate {
        append_cc_token(&mut out, &mut first, "proxy-revalidate");
    }

    out
}

pub fn make_etag(weak: bool, modified: SystemTime, len: u64) -> String {
    let dur = modified
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let raw = format!("{len:x}-{:x}-{:x}", dur.as_secs(), dur.subsec_nanos());
    if weak {
        format!("W/\"{raw}\"")
    } else {
        format!("\"{raw}\"")
    }
}

fn normalize_etag_for_weak_cmp(value: &str) -> &str {
    let v = value.trim();
    v.strip_prefix("W/").unwrap_or(v).trim()
}

pub fn if_none_match_hit(parts: &Parts, etag: &str) -> bool {
    let Some(value) = parts.headers.get(IF_NONE_MATCH) else {
        return false;
    };
    let Ok(raw) = value.to_str() else {
        return false;
    };

    raw.split(',').map(str::trim).any(|candidate| {
        candidate == "*"
            || normalize_etag_for_weak_cmp(candidate) == normalize_etag_for_weak_cmp(etag)
    })
}

fn unix_secs(t: SystemTime) -> u64 {
    t.duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn if_modified_since_hit(parts: &Parts, modified: SystemTime) -> bool {
    let Some(value) = parts.headers.get(IF_MODIFIED_SINCE) else {
        return false;
    };
    let Ok(raw) = value.to_str() else {
        return false;
    };
    let Ok(since) = httpdate::parse_http_date(raw) else {
        return false;
    };

    unix_secs(modified) <= unix_secs(since)
}
