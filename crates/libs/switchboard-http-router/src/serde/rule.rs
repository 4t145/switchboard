use std::{convert::Infallible, str::FromStr, sync::Arc};

use http::HeaderValue;

use crate::rule::{
    BytesRegexOrExact, HeaderMatch, QueryMatch, RegexOrExact, RuleBucket, RuleMatch,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
pub struct RuleBucketSerde<T> {
    // we can use heap hear...
    pub rules: Vec<RuleMatchSerde>,
    pub target: T,
}

impl<T: Clone> TryInto<RuleBucket<T>> for RuleBucketSerde<T> {
    type Error = crate::error::BuildError;

    fn try_into(self) -> Result<RuleBucket<T>, Self::Error> {
        let rules = self
            .rules
            .into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(RuleBucket {
            rules,
            target: self.target,
        })
    }
}

impl<T> RuleBucketSerde<T> {
    pub fn new(target: T) -> Self {
        Self {
            rules: Vec::new(),
            target,
        }
    }
    pub fn sort(&mut self) {
        self.rules.sort_by_key(RuleMatchSerde::priority);
    }
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
    pub fn add_rule(&mut self, rule: RuleMatchSerde) {
        self.rules.push(rule);
        self.sort();
    }
}

#[derive(Debug, Clone, bincode::Encode, bincode::Decode, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
/// T should be something string like, otherwise serde would cause error
pub enum RuleBucketSimplifiedSerde<T> {
    JustTarget(T),
    Rules {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        rules: Vec<RuleMatchExprGroup>,
        target: T,
    },
}

impl<T> RuleBucketSimplifiedSerde<T> {
    pub fn target(&self) -> &T {
        match self {
            RuleBucketSimplifiedSerde::JustTarget(t) => t,
            RuleBucketSimplifiedSerde::Rules { target, .. } => target,
        }
    }
    pub fn into_target(self) -> T {
        match self {
            RuleBucketSimplifiedSerde::JustTarget(t) => t,
            RuleBucketSimplifiedSerde::Rules { target, .. } => target,
        }
    }
}

impl<T: Clone> TryInto<RuleBucket<T>> for RuleBucketSimplifiedSerde<T> {
    type Error = crate::error::BuildError;

    fn try_into(self) -> Result<RuleBucket<T>, Self::Error> {
        let bucket_serde: RuleBucketSerde<T> = self.into();
        bucket_serde.try_into()
    }
}

impl<T> Into<RuleBucketSerde<T>> for RuleBucketSimplifiedSerde<T>
where
    T: Clone,
{
    fn into(self) -> RuleBucketSerde<T> {
        match self {
            RuleBucketSimplifiedSerde::JustTarget(t) => RuleBucketSerde::new(t),
            RuleBucketSimplifiedSerde::Rules { rules, target } => {
                let mut bucket = RuleBucketSerde::new(target);
                for expr in rules {
                    let mut rule_match_serde = RuleMatchSerde {
                        method: None,
                        headers: Vec::new(),
                        queries: Vec::new(),
                    };
                    for e in expr.exprs {
                        match e {
                            RuleMatchExpr::Method(m) => {
                                rule_match_serde.method = Some(m);
                            }
                            RuleMatchExpr::Header(h) => {
                                rule_match_serde.headers.push(h);
                            }
                            RuleMatchExpr::Query(q) => {
                                rule_match_serde.queries.push(q);
                            }
                        }
                    }
                    bucket.rules.push(rule_match_serde);
                }
                bucket.sort();
                bucket
            }
        }
    }
}

impl<T> From<RuleBucketSerde<T>> for RuleBucketSimplifiedSerde<T>
where
    T: Clone,
{
    fn from(bucket: RuleBucketSerde<T>) -> Self {
        if bucket.rules.is_empty() {
            return RuleBucketSimplifiedSerde::JustTarget(bucket.target);
        }
        let mut rules = Vec::with_capacity(bucket.rules.len());
        for rule_match_serde in bucket.rules {
            let mut exprs = Vec::new();
            if let Some(m) = &rule_match_serde.method {
                exprs.push(RuleMatchExpr::Method(m.clone()));
            }
            for h in &rule_match_serde.headers {
                exprs.push(RuleMatchExpr::Header(h.clone()));
            }
            for q in &rule_match_serde.queries {
                exprs.push(RuleMatchExpr::Query(q.clone()));
            }
            rules.push(RuleMatchExprGroup { exprs });
        }
        let target = bucket.target;
        RuleBucketSimplifiedSerde::Rules { rules, target }
    }
}

#[derive(Debug, Clone, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct RuleMatchExprGroup {
    pub exprs: Vec<RuleMatchExpr>,
}

impl std::fmt::Display for RuleMatchExprGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (idx, expr) in self.exprs.iter().enumerate() {
            if idx == 0 {
                write!(f, "{}", expr)?;
            } else {
                write!(f, ",{}", expr)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, bincode::Encode, bincode::Decode)]
pub enum RuleMatchExpr {
    Method(String),
    Header(HeaderMatchSerde),
    Query(QueryMatchSerde),
}

impl<'de> serde::Deserialize<'de> for RuleMatchExpr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        RuleMatchExpr::from_str(&s).map_err(serde::de::Error::custom)
    }
}
impl serde::Serialize for RuleMatchExpr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RuleMatchExprParseError {
    #[error("expected dot `.` in rule match expression")]
    ExpectDot,
    #[error("invalid rule kind in rule match expression: {0}")]
    InvalidRuleKind(String),
    #[error("expected equal sign `=` in rule match expression")]
    ExpectEqualSign,
    #[error("invalid exact or regex in rule match expression")]
    InvalidExactOrRegex(#[from] Infallible),
}

impl std::fmt::Display for RuleMatchExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleMatchExpr::Method(m) => write!(f, "method={}", m),
            RuleMatchExpr::Header(h) => write!(f, "header.{}={}", h.header_name, h.header_value),
            RuleMatchExpr::Query(q) => write!(f, "query.{}={}", q.query_name, q.query_value),
        }
    }
}
impl FromStr for RuleMatchExpr {
    type Err = RuleMatchExprParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((field, cond)) = s.split_once('=') else {
            return Err(RuleMatchExprParseError::ExpectEqualSign);
        };
        let field = field.trim();
        let cond = cond.trim();
        if field.eq_ignore_ascii_case("method") {
            Ok(RuleMatchExpr::Method(cond.to_string()))
        } else if let Some((kind, key)) = field.split_once('.') {
            let kind = kind.trim().to_lowercase();
            match kind.as_str() {
                "header" => {
                    let header_match = HeaderMatchSerde {
                        header_name: key.trim().to_string(),
                        header_value: RegexOrExactSerde::from_str(cond)?,
                    };
                    Ok(RuleMatchExpr::Header(header_match))
                }
                "query" => {
                    let query_match = QueryMatchSerde {
                        query_name: key.trim().to_string(),
                        query_value: RegexOrExactSerde::from_str(cond)?,
                    };
                    Ok(RuleMatchExpr::Query(query_match))
                }
                _ => Err(RuleMatchExprParseError::InvalidRuleKind(s.to_string())),
            }
        } else {
            Err(RuleMatchExprParseError::ExpectDot)
        }
    }
}

#[derive(
    Debug, Clone, Default, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode,
)]

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

#[serde(tag = "kind", content = "value")]
pub enum RegexOrExactSerde {
    Regex(String),
    Exact(String),
}

impl FromStr for RegexOrExactSerde {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("re:") {
            Ok(RegexOrExactSerde::Regex(s[3..].to_string()))
        } else {
            Ok(RegexOrExactSerde::Exact(s.to_string()))
        }
    }
}

impl std::fmt::Display for RegexOrExactSerde {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegexOrExactSerde::Regex(re) => write!(f, "re:{}", re),
            RegexOrExactSerde::Exact(ex) => write!(f, "{}", ex),
        }
    }
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
