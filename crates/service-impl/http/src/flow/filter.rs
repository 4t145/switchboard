pub mod timeout;
pub mod rewrite;
use std::sync::Arc;

use futures::future::BoxFuture;

use crate::{
    DynRequest, DynResponse,
    flow::{FlowContext, NodeTarget, node::NodeFn},
};

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
            (filter.call)(req, context, self).await
        } else {
            if let Some(filter) = self.input_filters.pop() {
                (filter.call)(req, context, self).await
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

pub trait Filter {
    fn call<'c>(
        self: Arc<Self>,
        req: DynRequest,
        ctx: &'c mut FlowContext,
        next: Next,
    ) -> impl futures::Future<Output = DynResponse> + 'c + Send;
}

#[derive(Clone)]
pub struct FilterReference {
    pub call: Arc<FilterFn>,
}

#[derive(Clone)]
pub struct DynamicFilter {
    pub call: Arc<FilterFn>,
}

impl DynamicFilter {
    pub fn from_trait<F>(filter: F) -> Self
    where
        F: Filter + Send + Sync + 'static,
    {
        let filter = Arc::new(filter);
        Self {
            call: Arc::new(move |req, ctx, next| Box::pin(filter.clone().call(req, ctx, next))),
        }
    }
}
