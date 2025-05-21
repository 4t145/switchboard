use std::{fmt::Display, num::ParseIntError, str::FromStr, string::FromUtf8Error, sync::Arc};

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone)]
pub enum Route {
    // utf-8 string, between 1 and 255 bytes, can not start with '['
    Named(Arc<str>),
    Anon(u32),
    Fallback,
}

impl Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Route::Named(name) => write!(f, "{}", name),
            Route::Anon(num) => write!(f, "[{}]", num),
            Route::Fallback => write!(f, "[fallback]"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InvalidRoute {
    #[error("invalid utf-8 string {0}")]
    FromUtf8Error(#[from] FromUtf8Error),
    #[error("invalid int {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("name too long")]
    NameTooLong,
    #[error("empty name")]
    EmptyName,
    #[error("name can not start with '['")]
    NameStartWithBracket,
}

impl FromStr for Route {
    type Err = InvalidRoute;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('[') {
            if s == "[fallback]" {
                Ok(Route::Fallback)
            } else {
                let num = s[1..s.len() - 1].parse::<u32>()?;
                Ok(Route::Anon(num))
            }
        } else if s.is_empty() {
            return Err(InvalidRoute::EmptyName);
        } else if s.len() > 255 {
            return Err(InvalidRoute::NameTooLong);
        } else {
            Ok(Route::Named(Arc::from(s)))
        }
    }
}

impl Serialize for Route {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Route::Named(name) => serializer.serialize_str(name),
            Route::Anon(num) => serializer.serialize_str(&format!("[{}]", num)),
            Route::Fallback => serializer.serialize_str("[fallback]"),
        }
    }
}

impl<'de> Deserialize<'de> for Route {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Route::from_str(&s).map_err(serde::de::Error::custom)
    }
}

pub trait Router: Send + Sync + 'static {
    fn route(&self, req: &http::request::Parts) -> Route;
}

#[derive(Clone)]
pub struct SharedRouter {
    router: Arc<dyn Router>,
}

impl std::fmt::Debug for SharedRouter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedRouter").finish()
    }
}

impl Router for SharedRouter {
    fn route(&self, req: &http::request::Parts) -> Route {
        self.router.route(req)
    }
}

impl SharedRouter {
    pub fn new<R>(router: R) -> Self
    where
        R: Router + Send + Sync + 'static,
    {
        Self {
            router: Arc::new(router),
        }
    }
}
