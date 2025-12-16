use std::sync::Arc;

use http::HeaderValue;

use crate::rule::{BytesRegexOrExact, HeaderMatch, QueryMatch, RegexOrExact, RuleBucket, RuleMatch};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
#[serde(rename_all = "camelCase")]
pub struct RuleBucketSerde<T> {
    // we can use heap hear...
    #[serde(default = "Vec::new")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<(RuleMatchSerde, T)>,
}

impl<T: Clone> TryInto<RuleBucket<T>> for RuleBucketSerde<T> {
    type Error = crate::error::BuildError;

    fn try_into(self) -> Result<RuleBucket<T>, Self::Error> {
        let rules = self
            .rules
            .into_iter()
            .map(|(r, t)| r.try_into().map(|rm| (rm, t)))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(RuleBucket { rules })
    }
}

impl<T> Default for RuleBucketSerde<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> RuleBucketSerde<T> {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }
    pub fn sort(&mut self) {
        self.rules
            .sort_by_key(|(rule_match, _)| rule_match.priority());
    }
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
    pub fn add_rule(&mut self, rule: RuleMatchSerde, data: T) {
        self.rules.push((rule, data));
        self.sort();
    }
}

#[derive(
    Debug, Clone, Default, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode,
)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct RuleMatchSerde {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    // max: 256
    pub headers: Vec<HeaderMatchSerde>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    // max: 256
    pub queries: Vec<QueryMatchSerde>,
}

impl TryInto<RuleMatch> for RuleMatchSerde {
    type Error = crate::error::BuildError;

    fn try_into(self) -> Result<RuleMatch, Self::Error> {
        let method = match self.method {
            Some(m) => Some(m.parse()?),
            None => None,
        };
        let mut headers = Vec::with_capacity(self.headers.len());
        for h in self.headers {
            headers.push(h.try_into()?);
        }
        let mut queries = Vec::with_capacity(self.queries.len());
        for q in self.queries {
            queries.push(q.try_into()?);
        }
        Ok(RuleMatch {
            method,
            headers,
            queries,
        })
    }
}

impl RuleMatchSerde {
    pub fn priority(&self) -> u32 {
        let method_priority: u8 = if self.method.is_some() { 1 } else { 0 };
        let header_priority: u8 = self.headers.len().min(255) as u8;
        let query_priority: u8 = self.queries.len().min(255) as u8;
        ((method_priority as u32) << 16) | ((header_priority as u32) << 8) | (query_priority as u32)
    }
    pub fn fallback_rule() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "kind", content = "value")]
pub enum RegexOrExactSerde {
    Regex(String),
    Exact(String),
}

impl TryInto<BytesRegexOrExact<HeaderValue>> for RegexOrExactSerde {
    type Error = crate::error::BuildError;

    fn try_into(self) -> Result<BytesRegexOrExact<HeaderValue>, Self::Error> {
        match self {
            RegexOrExactSerde::Regex(s) => {
                let re = regex::bytes::Regex::new(&s)?;
                Ok(BytesRegexOrExact::Regex(re))
            }
            RegexOrExactSerde::Exact(s) => {
                let hv = HeaderValue::from_str(&s)?;
                Ok(BytesRegexOrExact::Exact(hv))
            }
        }
    }
}

impl TryInto<RegexOrExact<Arc<str>>> for RegexOrExactSerde {
    type Error = crate::error::BuildError;

    fn try_into(self) -> Result<RegexOrExact<Arc<str>>, Self::Error> {
        match self {
            RegexOrExactSerde::Regex(s) => {
                let re = regex::Regex::new(&s)?;
                Ok(RegexOrExact::Regex(re))
            }
            RegexOrExactSerde::Exact(s) => Ok(RegexOrExact::Exact(Arc::from(s.as_str()))),
        }
    }
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
#[serde(rename_all = "camelCase")]
pub struct HeaderMatchSerde {
    pub header_name: String,
    pub header_value: RegexOrExactSerde,
}

impl TryInto<HeaderMatch> for HeaderMatchSerde {
    type Error = crate::error::BuildError;

    fn try_into(self) -> Result<HeaderMatch, Self::Error> {
        Ok(HeaderMatch {
            header_name: self.header_name.parse()?,
            header_value: self.header_value.try_into()?,
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
#[serde(rename_all = "camelCase")]
pub struct QueryMatchSerde {
    pub query_name: String,
    pub query_value: RegexOrExactSerde,
}

impl TryInto<QueryMatch> for QueryMatchSerde {
    type Error = crate::error::BuildError;

    fn try_into(self) -> Result<QueryMatch, Self::Error> {
        Ok(QueryMatch {
            query_name: Arc::from(self.query_name.as_str()),
            query_value: self.query_value.try_into()?,
        })
    }
}
