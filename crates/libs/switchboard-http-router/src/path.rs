use std::{collections::HashMap, sync::Arc};

use crate::{
    rule::{RuleBucket, RuleBucketMatched},
    utils::regex_match::{REGEX_CAPTURE_GROUPS_LIMIT, RegexMatched, capture_key},
};

#[derive(Debug, Clone, Default)]
pub struct PathTree<T: Clone> {
    pub matchit: matchit::Router<(Arc<str>, RuleBucket<T>)>,
    pub regex_matches: Vec<PathTreeRegexMatch<T>>,
    pub fallback: Option<T>,
}

#[derive(Debug, Clone)]
pub struct PathTreeRegexMatch<T: Clone> {
    pub regex: Arc<regex::Regex>,
    pub target: RuleBucket<T>,
}

#[derive(Debug, Clone)]
pub enum PathTreeMatched<T: Clone> {
    Matchit {
        route: Arc<str>,
        captures: HashMap<Arc<str>, Arc<str>>,
        matched: RuleBucketMatched<T>,
    },
    Regex {
        regex: Arc<str>,
        captures: RegexMatched,
        data: RuleBucketMatched<T>,
    },
    Fallback {
        data: T,
    },
}

pub enum PathTreeMatchedCapturesIterator<'a> {
    MatchIt(std::collections::hash_map::Iter<'a, Arc<str>, Arc<str>>),
    Regex(std::iter::Enumerate<std::slice::Iter<'a, Option<Arc<str>>>>),
    None,
}

impl<'a> Iterator for PathTreeMatchedCapturesIterator<'a> {
    type Item = (&'a str, &'a str);
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            PathTreeMatchedCapturesIterator::MatchIt(iter) => {
                iter.next().map(|(k, v)| (k.as_ref(), v.as_ref()))
            }
            PathTreeMatchedCapturesIterator::Regex(iter) => {
                iter.next().and_then(|(index, value)| {
                    if index >= REGEX_CAPTURE_GROUPS_LIMIT {
                        return None;
                    }
                    value.as_ref().map(|v| (capture_key(index), v.as_ref()))
                })
            }
            PathTreeMatchedCapturesIterator::None => None,
        }
    }
}

impl<T: Clone> PathTreeMatched<T> {
    pub fn captures_iter(&self) -> PathTreeMatchedCapturesIterator<'_> {
        match self {
            PathTreeMatched::Matchit { captures, .. } => {
                PathTreeMatchedCapturesIterator::MatchIt(captures.iter())
            }
            PathTreeMatched::Regex { captures, .. } => {
                PathTreeMatchedCapturesIterator::Regex(captures.matched.iter().enumerate())
            }
            PathTreeMatched::Fallback { .. } => PathTreeMatchedCapturesIterator::None,
        }
    }
    pub fn get_data(&self) -> &T {
        match self {
            PathTreeMatched::Matchit { matched, .. } => matched.get_data(),
            PathTreeMatched::Regex { data, .. } => data.get_data(),
            PathTreeMatched::Fallback { data } => data,
        }
    }
}

impl<T: Clone> PathTree<T> {
    pub fn new() -> Self {
        Self {
            matchit: matchit::Router::new(),
            regex_matches: Vec::new(),
            fallback: None,
        }
    }
    pub fn add_matchit_route(
        &mut self,
        route: impl Into<String>,
        target: impl Into<RuleBucket<T>>,
    ) -> Result<(), crate::error::BuildError> {
        let route = route.into();
        self.matchit
            .insert(route.clone(), (Arc::from(route), target.into()))
            .map_err(Into::into)
    }
    pub fn add_regex_route(
        &mut self,
        regex: impl Into<Arc<regex::Regex>>,
        target: impl Into<RuleBucket<T>>,
    ) {
        self.regex_matches.push(PathTreeRegexMatch {
            regex: regex.into(),
            target: target.into(),
        });
    }
    pub fn add_str_regex_route(
        &mut self,
        regex: &str,
        target: impl Into<RuleBucket<T>>,
    ) -> Result<(), crate::error::BuildError> {
        let regex = regex::Regex::new(regex)?;
        self.regex_matches.push(PathTreeRegexMatch {
            regex: regex.into(),
            target: target.into(),
        });
        Ok(())
    }
    pub fn set_fallback(&mut self, data: T) {
        self.fallback = Some(data);
    }
    pub fn match_request_parts(&self, parts: &http::request::Parts) -> Option<PathTreeMatched<T>> {
        let path = parts.uri.path();
        if let Some(matched) = self.matchit.at(path).ok()
            && let Some(rule_matched) = matched.value.1.match_request_part(parts)
        {
            let captures = matched
                .params
                .iter()
                .map(|(k, v)| (Arc::from(k), Arc::from(v)))
                .collect();
            return Some(PathTreeMatched::Matchit {
                route: matched.value.0.clone(),
                matched: rule_matched,
                captures,
            });
        }
        // try regex matches
        for regex_match in &self.regex_matches {
            if let Some(captures) = regex_match.regex.captures(path)
                && let Some(data) = regex_match.target.match_request_part(parts)
            {
                return Some(PathTreeMatched::Regex {
                    regex: regex_match.regex.as_str().into(),
                    captures: RegexMatched::from_captures(&regex_match.regex, &captures),
                    data,
                });
            }
        }
        // all missed, check fallback
        if let Some(data) = &self.fallback {
            return Some(PathTreeMatched::Fallback { data: data.clone() });
        }
        // otherwise return None
        None
    }
}
