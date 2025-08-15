pub mod rewrite;
pub mod timeout;
use std::sync::Arc;

use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};

use crate::{
    DynRequest, DynResponse, IntoDynResponse,
    flow::{FlowContext, NodeTarget, node::NodeFn},
};
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FilterId(Arc<str>);

impl std::fmt::Display for FilterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct Next {
    pub target: NodeTarget,
    pub output_filters: Vec<FilterReference>,
    pub input_filters: Vec<FilterReference>,
    pub call: Arc<NodeFn>,
    pub location: NextLocation,
}

pub enum NextLocation {
    Source,
    Target,
}

impl Next {
    pub async fn call(mut self, req: DynRequest, context: &mut FlowContext) -> DynResponse {
        let is_boundary =
            matches!(self.location, NextLocation::Source) && self.input_filters.is_empty();
        if is_boundary {
            context.entry(self.target.clone());
            self.location = NextLocation::Target;
        }
        let response = if let Some(filter) = self.output_filters.pop() {
            match context.get_filter(&filter.id) {
                Ok(filter) => (filter.call.clone())(req, context, self).await,
                Err(e) => e.into_dyn_response(),
            }
        } else {
            if let Some(filter) = self.input_filters.pop() {
                match context.get_filter(&filter.id) {
                    Ok(filter) => (filter.call.clone())(req, context, self).await,
                    Err(e) => e.into_dyn_response(),
                }
            } else {
                (self.call)(req, context).await
            }
        };
        if is_boundary {
            context.leave();
        }
        return response;
    }
}

pub type FilterFn = dyn Fn(DynRequest, &'_ mut FlowContext, Next) -> BoxFuture<'_, DynResponse>
    + Send
    + Sync
    + 'static;

pub trait FilterType {
    fn call<'c>(
        self: Arc<Self>,
        req: DynRequest,
        ctx: &'c mut FlowContext,
        next: Next,
    ) -> impl futures::Future<Output = DynResponse> + 'c + Send;
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FilterReference {
    pub id: FilterId,
    // pub call: Arc<FilterFn>,
}

#[derive(Clone)]
pub struct Filter {
    pub call: Arc<FilterFn>,
}

impl Filter {
    pub fn from_trait<F>(filter: F) -> Self
    where
        F: FilterType + Send + Sync + 'static,
    {
        let filter = Arc::new(filter);
        Self {
            call: Arc::new(move |req, ctx, next| Box::pin(filter.clone().call(req, ctx, next))),
        }
    }
}
