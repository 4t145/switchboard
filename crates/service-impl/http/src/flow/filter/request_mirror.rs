use std::str::FromStr;

use http::StatusCode;

use serde::{Deserialize, Serialize};
use switchboard_model::services::http::{ClassId, NodeTarget};

use crate::{
    BoxedError, DynRequest, DynResponse,
    consts::ERR_FILTER_REQUEST_MIRROR,
    flow::filter::{FilterClass, FilterLike},
    utils::error_response,
};

#[derive(Clone, Deserialize, Serialize, bincode::Encode, bincode::Decode)]
pub struct RequestMirrorFilterConfig {
    pub target: NodeTarget,
    #[serde(default)]
    /// need to be a fraction like "1/10" or percentage string like "10%"
    pub possibility: Option<FractionOrPercentage>,
    #[serde(default)]
    pub record_response: bool,
}

#[derive(Debug, Clone, bincode::Encode, bincode::Decode)]
pub enum FractionOrPercentage {
    Fraction(u16, u16),
    Percentage(u16),
}

#[derive(Debug, thiserror::Error)]
pub enum FractionOrPercentageParseError {
    #[error("Invalid u16 value: {0}")]
    InvalidU16(#[from] std::num::ParseIntError),
    #[error("Invalid format")]
    InvalidFormat,
}

impl std::fmt::Display for FractionOrPercentage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FractionOrPercentage::Fraction(num, den) => write!(f, "{}/{}", num, den),
            FractionOrPercentage::Percentage(percent) => write!(f, "{}%", percent),
        }
    }
}

impl serde::Serialize for FractionOrPercentage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for FractionOrPercentage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FractionOrPercentage::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl FromStr for FractionOrPercentage {
    type Err = FractionOrPercentageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((numerator, denominator)) = s.split_once('/') {
            let num: u16 = numerator.trim().parse()?;
            let den: u16 = denominator.trim().parse()?;
            Ok(FractionOrPercentage::Fraction(num, den))
        } else if let Some(percent_str) = s.strip_suffix('%') {
            let percent: u16 = percent_str.trim().parse()?;
            Ok(FractionOrPercentage::Percentage(percent))
        } else {
            Err(FractionOrPercentageParseError::InvalidFormat)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RequestMirrorFilterConfigError {
    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
    #[error("Invalid header name: {0}")]
    InvalidHeaderName(#[from] http::header::InvalidHeaderName),
}

impl TryInto<RequestMirrorFilter> for RequestMirrorFilterConfig {
    type Error = http::header::InvalidHeaderValue;

    fn try_into(self) -> Result<RequestMirrorFilter, Self::Error> {
        let possibility_to_mirror = if let Some(possibility) = self.possibility {
            match possibility {
                FractionOrPercentage::Fraction(num, den) => Some((num, den)),
                FractionOrPercentage::Percentage(percent) => {
                    if percent > 100 {
                        Some((100, 100))
                    } else {
                        Some((percent, 100))
                    }
                }
            }
        } else {
            None
        };
        Ok(RequestMirrorFilter {
            target: self.target,
            possibility_to_mirror,
            record_response: self.record_response,
        })
    }
}

pub struct RequestMirrorFilter {
    pub target: NodeTarget,
    // this is a fraction: {.1}/{.2}
    pub possibility_to_mirror: Option<(u16, u16)>,

    pub record_response: bool,
}

impl RequestMirrorFilter {
    pub async fn call_inner(
        self: std::sync::Arc<Self>,
        req: DynRequest,
        ctx: &mut crate::flow::FlowContext,
        next: super::Next,
    ) -> Result<DynResponse, RequestMirrorFilterError> {
        let (parts, mut body) = req.into_parts();
        // throw dice here
        if let Some((numerator, denominator)) = self.possibility_to_mirror {
            let roll: u16 = rand::random_range(0..denominator);
            if roll >= numerator {
                let (forked_context, forked_request) = ctx.fork(&mut body, &parts).await?;
                let mut forked_next = next.clone();
                let record_response = self.record_response;
                let target = self.target.clone();
                forked_next.target = target.clone();
                tokio::spawn(async move {
                    let mirror_response = forked_next
                        .call(forked_request, &mut forked_context.clone())
                        .await;
                    if record_response {
                        tracing::info!(target:"request-mirror-response",
                            target=%target,
                            status=%mirror_response.status(),
                            "mirrored response"
                        );
                    }
                });
            }
        }
        let req = DynRequest::from_parts(parts, body);
        Ok(next.call(req, ctx).await)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RequestMirrorFilterError {
    #[error("Failed to fork request: {0}")]
    ForkError(#[from] BoxedError),
}

impl FilterLike for RequestMirrorFilter {
    async fn call(
        self: std::sync::Arc<Self>,
        req: DynRequest,
        ctx: &mut crate::flow::FlowContext,
        next: super::Next,
    ) -> DynResponse {
        self.call_inner(req, ctx, next).await.unwrap_or_else(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                e,
                ERR_FILTER_REQUEST_MIRROR,
            )
        })
    }
}

pub struct RequestMirrorFilterClass;

impl FilterClass for RequestMirrorFilterClass {
    type Filter = RequestMirrorFilter;
    type Error = RequestMirrorFilterConfigError;
    type Config = RequestMirrorFilterConfig;

    fn id(&self) -> ClassId {
        ClassId::std("request-mirror")
    }

    fn construct(&self, config: Self::Config) -> Result<Self::Filter, Self::Error> {
        let filter: RequestMirrorFilter = config.try_into()?;
        Ok(filter)
    }
}
