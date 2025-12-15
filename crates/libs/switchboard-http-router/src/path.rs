use std::sync::Arc;

use crate::rule::{RuleBucket, RuleBucketMatched};

#[derive(Debug, Clone, Default)]
pub struct PathTree<T: Clone> {
    pub matchit: matchit::Router<RuleBucket<T>>,
    pub regex_matches: Vec<PathTreeRegexMatch<T>>,
    pub fallback: Option<T>,
}

#[derive(Debug, Clone)]
pub struct PathTreeRegexMatch<T: Clone> {
    pub regex: Arc<regex::Regex>,
    pub target: RuleBucket<T>,
}

#[derive(Debug)]
pub enum PathTreeMatched<'c, T: Clone> {
    Matchit {
        matched: RuleBucketMatched<'c, T>,
    },
    Regex {
        regex: Arc<regex::Regex>,
        captures: regex::Captures<'c>,
        data: RuleBucketMatched<'c, T>,
    },
    Fallback {
        data: T,
    },
}

impl<'c, T: Clone> PathTreeMatched<'c, T> {
    pub fn get_data(&self) -> &T {
        match self {
            PathTreeMatched::Matchit { matched } => matched.get_data(),
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
        self.matchit
            .insert(route, target.into())
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
    pub fn match_request_parts<'c>(
        &self,
        parts: &'c http::request::Parts,
    ) -> Option<PathTreeMatched<'c, T>> {
        let path = parts.uri.path();
        if let Some(matched) = self.matchit.at(path).ok()
            && let Some(matched) = matched.value.match_request_part(parts)
        {
            return Some(PathTreeMatched::Matchit { matched });
        }
        // try regex matches
        for regex_match in &self.regex_matches {
            if let Some(captures) = regex_match.regex.captures(path)
                && let Some(data) = regex_match.target.match_request_part(parts)
            {
                return Some(PathTreeMatched::Regex {
                    regex: regex_match.regex.clone(),
                    captures,
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
