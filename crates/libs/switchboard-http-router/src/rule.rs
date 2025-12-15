use std::{collections::BTreeMap, sync::Arc};

use http::{HeaderName, HeaderValue, request::Parts};
use regex::bytes::Regex;

use crate::utils::query_kv::QueryKvIter;

#[derive(Debug, Clone, Default)]
pub struct RuleBucket<T: Clone> {
    pub rules: Vec<(RuleMatch, T)>,
}

impl<T: Clone> RuleBucket<T> {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }
    pub fn new_single(data: T) -> Self {
        Self {
            rules: vec![(RuleMatch::fallback_rule(), data)],
        }
    }
    pub fn sort(&mut self) {
        self.rules
            .sort_by_key(|(rule_match, _)| rule_match.priority());
    }
    pub fn add_rule(&mut self, rule: RuleMatch, data: T) {
        // check if is fallback rule
        if rule.is_fallback_rule() {
            self.rules.push((rule, data));
            return;
        }
        self.rules.push((rule, data));
        self.sort();
    }
    pub fn match_request_part<'c>(
        &self,
        parts: &'c http::request::Parts,
    ) -> Option<RuleBucketMatched<'c, T>> {
        for (rule, data) in &self.rules {
            if let Some(matched) = rule.is_match_request_part(parts) {
                return Some(RuleBucketMatched {
                    data: data.clone(),
                    matched,
                });
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct RuleBucketMatched<'c, T> {
    pub data: T,
    pub matched: RuleMatched<'c>,
}

impl<'c, T> RuleBucketMatched<'c, T> {
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
    pub fn is_match_request_part<'c>(
        &self,
        parts: &'c http::request::Parts,
    ) -> Option<RuleMatched<'c>> {
        let mut rule_matched = RuleMatched {
            method_matched: false,
            header_matches: Vec::new(),
            query_matches: Vec::new(),
        };
        // check method
        if let Some(method) = &self.method {
            if parts.method != *method {
                return None;
            } else {
                rule_matched.method_matched = true;
            }
        }
        // check headers
        for header_match in &self.headers {
            let matched = header_match.match_headers(parts)?;
            rule_matched.header_matches.push(matched);
        }
        // if queries is empty, return early
        if self.queries.is_empty() {
            return Some(rule_matched);
        }
        // otherwise, we should expect uri has a query
        let query = parts.uri.query()?;
        let query_iter = QueryKvIter::new(query).collect::<BTreeMap<&str, Option<&str>>>();
        for query in &self.queries {
            let matched = query.match_query(&query_iter)?;
            rule_matched.query_matches.push(matched);
        }
        Some(rule_matched)
    }
}

#[derive(Debug)]
pub struct RuleMatched<'c> {
    pub method_matched: bool,
    pub header_matches: Vec<RegexOrExactMatched<'c, HeaderValue>>,
    pub query_matches: Vec<RegexOrExactMatched<'c, Arc<str>>>,
}
#[derive(Debug, Clone)]
pub enum RegexOrExact<T> {
    Regex(Arc<Regex>),
    Exact(T),
}

#[derive(Debug)]
pub enum RegexOrExactMatched<'c, T> {
    Regex {
        regex: Arc<Regex>,
        captures: regex::bytes::Captures<'c>,
    },
    Exact(T),
}

impl RegexOrExact<HeaderValue> {
    pub fn match_header_value<'c>(
        &self,
        value: &'c HeaderValue,
    ) -> Option<RegexOrExactMatched<'c, HeaderValue>> {
        match self {
            RegexOrExact::Regex(regex) => {
                regex
                    .captures(value.as_bytes())
                    .map(|captures| RegexOrExactMatched::Regex {
                        regex: regex.clone(),
                        captures,
                    })
            }
            RegexOrExact::Exact(exact_value) => {
                (exact_value == value).then_some(RegexOrExactMatched::Exact(value.clone()))
            }
        }
    }
}

impl RegexOrExact<Arc<str>> {
    pub fn match_str_value<'c>(&self, value: &'c str) -> Option<RegexOrExactMatched<'c, Arc<str>>> {
        match self {
            RegexOrExact::Regex(regex) => {
                regex
                    .captures(value.as_bytes())
                    .map(|captures| RegexOrExactMatched::Regex {
                        regex: regex.clone(),
                        captures,
                    })
            }
            RegexOrExact::Exact(exact_value) => (exact_value.as_ref() == value)
                .then_some(RegexOrExactMatched::Exact(exact_value.clone())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HeaderMatch {
    pub header_name: HeaderName,
    pub header_value: RegexOrExact<HeaderValue>,
}

impl HeaderMatch {
    /// this will return matched when first header pair is matched
    pub fn match_headers<'c>(
        &self,
        parts: &'c Parts,
    ) -> Option<RegexOrExactMatched<'c, HeaderValue>> {
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
    pub fn match_query<'c>(
        &self,
        queries: &BTreeMap<&'c str, Option<&'c str>>,
    ) -> Option<RegexOrExactMatched<'c, Arc<str>>> {
        let value = queries.get(self.query_name.as_ref())?.unwrap_or_default();
        self.query_value.match_str_value(value)
    }
}
