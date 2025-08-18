use http::request::Parts;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::flow::{
    node::NodeClass,
    router::{RouterNode, WithRoutes},
};

use super::{NodePort, Router};
use tera::Tera;

pub struct PathRouter {
    pub router: matchit::Router<PathRouterEndpoint>,
    pub tera: Tera,
}

pub struct PathRouterEndpoint {
    pub route: NodePort,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[typeshare]
#[serde(default)]
pub struct MatchItem {
    pub priority: i32,
    pub template: String,
    pub route: NodePort,
    pub path: String,
}

impl Default for MatchItem {
    fn default() -> Self {
        Self {
            priority: 0,
            template: "/{*path}".into(),
            route: NodePort::Default,
            path: "/{*path}".into(),
        }
    }
}

impl Router for PathRouter {
    fn route(&self, req: &mut Parts) -> NodePort {
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
                    return NodePort::Default;
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
                    return NodePort::Default;
                };

                uri_parts.path_and_query = Some(p_and_q);
                let Ok(new_uri) = http::Uri::from_parts(uri_parts).inspect_err(|e| {
                    tracing::error!("Failed to construct URI from parts: {}", e);
                }) else {
                    return NodePort::Default;
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
                NodePort::Default
            }
        }
    }
}

pub struct PathMatch;

#[derive(Debug, thiserror::Error)]
pub enum PathRouterConstructError {
    #[error("invalid route configuration: {0}")]
    InvalidRouteConfig(#[from] serde_json::Error),
    #[error("tera template error: {0}")]
    TeraTemplateError(#[from] tera::Error),
    #[error("matchit router insert error: {0}")]
    MatchitRouterInsertError(#[from] matchit::InsertError),
}
#[typeshare]
pub type PathMatchRouterConfig = Vec<MatchItem>;

#[derive(Debug, thiserror::Error)]
pub enum PathMatchConstructError {
    #[error("tera template error: {0}")]
    TeraTemplateError(#[from] tera::Error),
    #[error("matchit router insert error: {0}")]
    MatchitRouterInsertError(#[from] matchit::InsertError),
}

impl NodeClass for PathMatch {
    type Config = WithRoutes<PathMatchRouterConfig>;
    type Error = PathMatchConstructError;
    type Node = RouterNode<PathRouter>;
    fn construct(&self, config: Self::Config) -> Result<Self::Node, Self::Error> {
        let mut router_config = config.router_config;
        router_config.sort_by(|a, b| a.priority.cmp(&b.priority));
        let mut tera = Tera::default();
        let mut router = matchit::Router::new();
        for item in router_config {
            tera.add_raw_template(item.route.as_str(), &item.template)?;
            router.insert(item.path, PathRouterEndpoint { route: item.route })?;
        }
        Ok(RouterNode::new(config.routes, PathRouter { router, tera }))
    }

    fn id(&self) -> crate::instance::class::ClassId {
        crate::instance::class::ClassId::std("path-match")
    }
}
