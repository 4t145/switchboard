use std::{collections::BTreeMap, sync::Arc};

use http::{HeaderName, HeaderValue, request::Parts};

use crate::utils::{
    query_kv::QueryKvIter,
    regex_match::{BytesRegexMatched, RegexMatched},
};

#[derive(Debug, Clone, Default)]
pub struct RuleBucket<T: Clone> {
    pub rules: Vec<RuleMatch>,
    pub target: T,
}

impl<T: Clone> RuleBucket<T> {
    pub fn new(target: T) -> Self {
        Self {
            rules: Vec::new(),
            target,
        }
    }
    pub fn new_single(data: T) -> Self {
        Self {
            rules: vec![],
            target: data,
        }
    }
    pub fn sort(&mut self) {
        self.rules.sort_by_key(RuleMatch::priority);
    }
    pub fn add_rule(&mut self, rule: RuleMatch) {
        self.rules.push(rule);
        self.sort();
    }
    pub fn match_request_part(&self, parts: &http::request::Parts) -> Option<RuleBucketMatched<T>> {
        if self.rules.is_empty() {
            return Some(RuleBucketMatched {
                data: self.target.clone(),
                matched: None,
            });
        }
        for rule in &self.rules {
            if let Some(matched) = rule.is_match_request_part(parts) {
                return Some(RuleBucketMatched {
                    data: self.target.clone(),
                    matched: Some(matched),
                });
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct RuleBucketMatched<T> {
    pub data: T,
    pub matched: Option<RuleMatched>,
}

impl<T> RuleBucketMatched<T> {
    pub fn get_data(&self) -> &T
    where
        T: Clone,
    {
        &self.data
    }
}

#[derive(Debug, Clone, Default)]
pub struct RuleMatch {
    pub method: Option<http::Method>,
    // max: 256
    pub headers: Vec<HeaderMatch>,
    // max: 256
    pub queries: Vec<QueryMatch>,
}

impl RuleMatch {
    pub fn is_fallback_rule(&self) -> bool {
        self.method.is_none() && self.headers.is_empty() && self.queries.is_empty()
    }
    pub fn priority(&self) -> u32 {
        let method_priority: u8 = if self.method.is_some() { 1 } else { 0 };
        let header_priority: u8 = self.headers.len().min(255) as u8;
        let query_priority: u8 = self.queries.len().min(255) as u8;
        ((method_priority as u32) << 16) | ((header_priority as u32) << 8) | (query_priority as u32)
    }
    pub fn fallback_rule() -> Self {
        Self::default()
    }
    pub fn is_match_request_part(&self, parts: &http::request::Parts) -> Option<RuleMatched> {
        let mut method_matched = false;
        let mut header_matches = Vec::with_capacity(self.headers.len());
        let mut query_matches = Vec::with_capacity(self.queries.len());
        // check method
        if let Some(method) = &self.method {
            if parts.method != *method {
                return None;
            } else {
                method_matched = true;
            }
        }
        // check headers
        for header_match in &self.headers {
            let matched = header_match.match_headers(parts)?;
            header_matches.push(matched);
        }
        // if queries is empty, return early
        if self.queries.is_empty() {
            return Some(RuleMatched {
                method_matched,
                header_matches: header_matches.into(),
                query_matches: query_matches.into(),
            });
        }
        // otherwise, we should expect uri has a query
        let query = parts.uri.query()?;
        let query_iter = QueryKvIter::new(query).collect::<BTreeMap<&str, Option<&str>>>();
        for query in &self.queries {
            let matched = query.match_query(&query_iter)?;
            query_matches.push(matched);
        }
        Some(RuleMatched {
            method_matched,
            header_matches: header_matches.into(),
            query_matches: query_matches.into(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct RuleMatched {
    pub method_matched: bool,
    pub header_matches: Arc<[BytesRegexOrExactMatched<HeaderValue>]>,
    pub query_matches: Arc<[RegexOrExactMatched<Arc<str>>]>,
}

#[derive(Debug, Clone)]
pub enum BytesRegexOrExact<T> {
    Regex(regex::bytes::Regex),
    Exact(T),
}

#[derive(Debug, Clone)]
pub enum RegexOrExact<T> {
    Regex(regex::Regex),
    Exact(T),
}

#[derive(Debug, Clone)]
pub enum RegexOrExactMatched<T> {
    Regex(RegexMatched),
    Exact(T),
}

#[derive(Debug, Clone)]
pub enum BytesRegexOrExactMatched<T> {
    Regex(BytesRegexMatched),
    Exact(T),
}

impl BytesRegexOrExact<HeaderValue> {
    pub fn match_header_value(
        &self,
        value: &HeaderValue,
    ) -> Option<BytesRegexOrExactMatched<HeaderValue>> {
        match self {
            BytesRegexOrExact::Regex(regex) => regex
                .captures(value.as_bytes())
                .map(|c| BytesRegexMatched::from_captures(regex, c))
                .map(BytesRegexOrExactMatched::Regex),
            BytesRegexOrExact::Exact(exact_value) => {
                (exact_value == value).then_some(BytesRegexOrExactMatched::Exact(value.clone()))
            }
        }
    }
}

impl RegexOrExact<Arc<str>> {
    pub fn match_str_value(&self, value: &str) -> Option<RegexOrExactMatched<Arc<str>>> {
        match self {
            RegexOrExact::Regex(regex) => regex
                .captures(value)
                .map(|c| RegexMatched::from_captures(regex, &c))
                .map(RegexOrExactMatched::Regex),
            RegexOrExact::Exact(exact_value) => (exact_value.as_ref() == value)
                .then_some(RegexOrExactMatched::Exact(exact_value.clone())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HeaderMatch {
    pub header_name: HeaderName,
    pub header_value: BytesRegexOrExact<HeaderValue>,
}

impl HeaderMatch {
    /// this will return matched when first header pair is matched
    pub fn match_headers(&self, parts: &Parts) -> Option<BytesRegexOrExactMatched<HeaderValue>> {
        for header in parts.headers.get_all(&self.header_name) {
            if let Some(matched) = self.header_value.match_header_value(header) {
                return Some(matched);
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct QueryMatch {
    pub query_name: Arc<str>,
    pub query_value: RegexOrExact<Arc<str>>,
}

impl QueryMatch {
    /// this would treat none query value as empty string, for example, ?a=&b would treat b's value as ""
    pub fn match_query(
        &self,
        queries: &BTreeMap<&str, Option<&str>>,
    ) -> Option<RegexOrExactMatched<Arc<str>>> {
        let value = queries.get(self.query_name.as_ref())?.unwrap_or_default();
        self.query_value.match_str_value(value)
    }
}
