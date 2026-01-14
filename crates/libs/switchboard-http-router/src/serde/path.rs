use std::{collections::BTreeMap, sync::Arc};

use crate::{
    path::{PathTree, PathTreeRegexMatch},
    rule::RuleBucket,
    serde::rule::{RuleBucketSerde, RuleBucketSimplifiedSerde},
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]

pub struct PathTreeSerde<T> {
    #[serde(default = "BTreeMap::new")]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub route: BTreeMap<String, RuleBucketSimplifiedSerde<T>>,
    #[serde(default = "Vec::new")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub regex_matches: Vec<PathTreeRegexMatchSerde<T>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<T>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
#[serde(transparent)]
pub struct PathTreeSerdeMapStyle<T> {
    /// 1. match routes: "/get/users/*"
    /// 2. regex routes: "re:^/get/users/([0-9]+)$"
    /// 3. fallback route: "fallback"
    pub route: BTreeMap<String, RuleBucketSimplifiedSerde<T>>,
}

impl<T> std::ops::Deref for PathTreeSerdeMapStyle<T> {
    type Target = BTreeMap<String, RuleBucketSimplifiedSerde<T>>;
    fn deref(&self) -> &Self::Target {
        &self.route
    }
}
impl<T> std::ops::DerefMut for PathTreeSerdeMapStyle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.route
    }
}

impl<T> Default for PathTreeSerdeMapStyle<T> {
    fn default() -> Self {
        Self {
            route: BTreeMap::new(),
        }
    }
}

impl<T: Clone> PathTreeSerdeMapStyle<T> {
    pub fn merge(&mut self, other: PathTreeSerdeMapStyle<T>) {
        self.route.extend(other.route);
    }
    pub fn into_regular_style(self) -> PathTreeSerde<T> {
        let mut path_tree = PathTreeSerde::default();
        for (mut route, bucket) in self.route {
            if route == "fallback" {
                path_tree.fallback = Some(bucket.into_target());
            } else if let Some(regex_str) = route.strip_prefix("re:") {
                let regex = regex_str.to_string();
                path_tree.regex_matches.push(PathTreeRegexMatchSerde {
                    regex,
                    target: RuleBucketSimplifiedSerde::from(bucket),
                });
            } else {
                // check if route ends with '/*' for matchit route
                if let Some(prefix) = route.strip_suffix("/*") {
                    route = format!("{}/{{*rest}}", prefix);
                }
                path_tree
                    .route
                    .insert(route, RuleBucketSimplifiedSerde::from(bucket));
            }
        }
        path_tree
    }
}

impl<T> PathTreeSerde<T> {
    pub fn add_matchit_route(&mut self, route: String, target: RuleBucketSimplifiedSerde<T>) {
        self.route.insert(route, target);
    }
    pub fn add_regex_route(&mut self, regex: String, target: RuleBucketSimplifiedSerde<T>) {
        self.regex_matches
            .push(PathTreeRegexMatchSerde { regex, target });
    }
}

impl<T> Default for PathTreeSerde<T> {
    fn default() -> Self {
        Self {
            route: BTreeMap::new(),
            regex_matches: Vec::new(),
            fallback: None,
        }
    }
}
impl<T: Clone> TryInto<PathTree<T>> for PathTreeSerde<T> {
    type Error = crate::error::BuildError;

    fn try_into(self) -> Result<PathTree<T>, Self::Error> {
        let mut path_tree = PathTree::new();
        for (route, bucket_serde) in self.route {
            let bucket: RuleBucket<T> = bucket_serde.try_into()?;
            path_tree.add_matchit_route(route, bucket)?;
        }
        for regex_match_serde in self.regex_matches {
            let regex_match: PathTreeRegexMatch<T> = regex_match_serde.try_into()?;
            path_tree.regex_matches.push(regex_match);
        }
        path_tree.fallback = self.fallback;
        Ok(path_tree)
    }
}

impl<T: Clone> TryInto<PathTree<T>> for PathTreeSerdeMapStyle<T> {
    type Error = crate::error::BuildError;

    fn try_into(self) -> Result<PathTree<T>, Self::Error> {
        let regular_style = self.into_regular_style();
        regular_style.try_into()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]

pub struct PathTreeRegexMatchSerde<T> {
    pub regex: String,
    pub target: RuleBucketSimplifiedSerde<T>,
}

impl<T: Clone> TryInto<crate::path::PathTreeRegexMatch<T>> for PathTreeRegexMatchSerde<T> {
    type Error = crate::error::BuildError;

    fn try_into(self) -> Result<crate::path::PathTreeRegexMatch<T>, Self::Error> {
        let regex = Arc::new(regex::Regex::new(&self.regex)?);
        let target: RuleBucket<T> = self.target.try_into()?;
        Ok(crate::path::PathTreeRegexMatch { regex, target })
    }
}
