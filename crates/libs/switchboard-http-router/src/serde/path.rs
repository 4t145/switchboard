use std::{collections::BTreeMap, sync::Arc};

use crate::{
    path::{PathTree, PathTreeRegexMatch},
    rule::RuleBucket,
    serde::rule::RuleBucketSerde,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
#[serde(rename_all = "camelCase")]
pub struct PathTreeSerde<T> {
    #[serde(default = "BTreeMap::new")]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub matchit: BTreeMap<String, RuleBucketSerde<T>>,
    #[serde(default = "Vec::new")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub regex_matches: Vec<PathTreeRegexMatchSerde<T>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<T>,
}

impl<T> PathTreeSerde<T> {
    pub fn add_matchit_route(&mut self, route: String, target: RuleBucketSerde<T>) {
        self.matchit.insert(route, target);
    }
    pub fn add_regex_route(&mut self, regex: String, target: RuleBucketSerde<T>) {
        self.regex_matches.push(PathTreeRegexMatchSerde { regex, target });
    }
}

impl<T> Default for PathTreeSerde<T> {
    fn default() -> Self {
        Self {
            matchit: BTreeMap::new(),
            regex_matches: Vec::new(),
            fallback: None,
        }
    }
}

impl<T: Clone> TryInto<PathTree<T>> for PathTreeSerde<T> {
    type Error = crate::error::BuildError;

    fn try_into(self) -> Result<PathTree<T>, Self::Error> {
        let mut path_tree = PathTree::new();
        for (route, bucket_serde) in self.matchit {
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

#[derive(
    Debug, Default, Clone, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode,
)]
#[serde(rename_all = "camelCase")]
pub struct PathTreeRegexMatchSerde<T> {
    pub regex: String,
    #[serde(default = "RuleBucketSerde::new")]
    #[serde(skip_serializing_if = "RuleBucketSerde::is_empty")]
    pub target: RuleBucketSerde<T>,
}

impl<T: Clone> TryInto<crate::path::PathTreeRegexMatch<T>> for PathTreeRegexMatchSerde<T> {
    type Error = crate::error::BuildError;

    fn try_into(self) -> Result<crate::path::PathTreeRegexMatch<T>, Self::Error> {
        let regex = Arc::new(regex::Regex::new(&self.regex)?);
        let target: RuleBucket<T> = self.target.try_into()?;
        Ok(crate::path::PathTreeRegexMatch { regex, target })
    }
}
