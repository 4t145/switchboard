use std::collections::HashMap;

use http::request::Parts;
use serde::{Deserialize, Serialize};

use crate::object::class::SbhClass;
use tera::Tera;

use super::{Route, Router, SharedRouter};

pub struct PathRouter {
    pub router: matchit::Router<PathRouterEndpoint>,
    pub tera: Tera,
}

pub struct PathRouterEndpoint {
    pub route: Route,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct MatchItem {
    pub priority: i32,
    pub template: String,
    pub route: Route,
    pub path: String,
}

impl Default for MatchItem {
    fn default() -> Self {
        Self {
            priority: 0,
            template: "/{*path}".into(),
            route: Route::Fallback,
            path: "/{*path}".into(),
        }
    }
}

impl Router for PathRouter {
    fn route(&self, req: &mut Parts) -> Route {
        let path = req.uri.path();
        match self.router.at(path) {
            Ok(mat) => {
                let route = mat.value.route.clone();
                let render_context = req.extensions.get_or_insert_default::<tera::Context>();
                for (key, val) in mat.params.iter() {
                    render_context.insert(key, val);
                }
                let Ok(mut rendered_path) = self
                    .tera
                    .render(&mat.value.route.to_string(), render_context)
                else {
                    return Route::Fallback;
                };
                let new_uri = req.uri.clone();
                if let Some(raw_query) = new_uri.query() {
                    rendered_path.push('?');
                    rendered_path.push_str(raw_query);
                }

                let mut uri_parts = new_uri.into_parts();
                let Ok(p_and_q) = http::uri::PathAndQuery::from_maybe_shared(rendered_path)
                    .inspect_err(|e| {
                        tracing::error!("Failed to parse path and query: {}", e);
                    })
                else {
                    return Route::Fallback;
                };

                uri_parts.path_and_query = Some(p_and_q);
                let Ok(new_uri) = http::Uri::from_parts(uri_parts).inspect_err(|e| {
                    tracing::error!("Failed to construct URI from parts: {}", e);
                }) else {
                    return Route::Fallback;
                };
                tracing::debug!(
                    "rewrite path and query from {:?} to {:?}",
                    req.uri.path_and_query(),
                    new_uri.path_and_query(),
                );
                req.uri = new_uri;
                route
            }
            Err(e) => {
                tracing::debug!("fail to match path: {}", e);
                Route::Fallback
            }
        }
    }
}

pub struct Path;

#[derive(Debug, thiserror::Error)]
pub enum PathRouterConstructError {
    #[error("invalid route configuration: {0}")]
    InvalidRouteConfig(#[from] serde_json::Error),
    #[error("tera template error: {0}")]
    TeraTemplateError(#[from] tera::Error),
    #[error("matchit router insert error: {0}")]
    MatchitRouterInsertError(#[from] matchit::InsertError),
}

pub type PathRouterConfig = Vec<MatchItem>;

impl SbhClass for Path {
    type Error = PathRouterConstructError;
    type Type = SharedRouter;
    fn name(&self) -> crate::object::class::ObjectClassName {
        crate::object::class::ObjectClassName::std("path-match")
    }
    fn construct(&self, config: &str) -> Result<Self::Type, Self::Error> {
        let mut config: PathRouterConfig = serde_json::from_str(config)?;
        config.sort_by(|a, b| a.priority.cmp(&b.priority));
        let mut tera = Tera::default();
        let mut router = matchit::Router::new();
        for item in config {
            tera.add_raw_template(item.route.as_str(), &item.template)?;
            router.insert(item.path, PathRouterEndpoint { route: item.route })?;
        }
        Ok(SharedRouter::new(PathRouter { router, tera }))
    }
}
