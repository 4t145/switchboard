pub mod timeout;
pub mod request_header_modify;
pub mod response_header_modify;
pub mod request_mirror;
pub mod request_redirect;
pub mod url_rewrite;

use std::sync::Arc;

use futures::future::BoxFuture;
use serde::de::DeserializeOwned;
use switchboard_model::{custom_config::formats::TransferObject, services::http::*};

use crate::{
    DynRequest, DynResponse, IntoDynResponse,
    flow::{FlowContext, node::NodeFn},
    instance::{InstanceValue, class::Class},
};

#[derive( Clone)]
pub struct Next {
    pub target: NodeTarget,
    pub output_filters: Vec<FilterReference>,
    pub input_filters: Vec<FilterReference>,
    pub call: Arc<NodeFn>,
    pub location: NextLocation,
}

#[derive( Clone)]
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
        } else if let Some(filter) = self.input_filters.pop() {
            match context.get_filter(&filter.id) {
                Ok(filter) => (filter.call.clone())(req, context, self).await,
                Err(e) => e.into_dyn_response(),
            }
        } else {
            (self.call)(req, context).await
        };
        if is_boundary {
            context.leave();
        }
        response
    }
}

pub type FilterFn = dyn Fn(DynRequest, &'_ mut FlowContext, Next) -> BoxFuture<'_, DynResponse>
    + Send
    + Sync
    + 'static;

pub trait FilterLike: Send + Sync + 'static {
    fn call<'c>(
        self: Arc<Self>,
        req: DynRequest,
        ctx: &'c mut FlowContext,
        next: Next,
    ) -> impl futures::Future<Output = DynResponse> + 'c + Send;
}

#[derive(Clone)]
pub struct Filter {
    pub call: Arc<FilterFn>,
}

impl std::fmt::Debug for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Filter").finish()
    }
}

impl Filter {
    pub fn from_filter_like<F>(filter: F) -> Self
    where
        F: FilterLike,
    {
        let filter = Arc::new(filter);
        Self {
            call: Arc::new(move |req, ctx, next| Box::pin(filter.clone().call(req, ctx, next))),
        }
    }
}

pub trait FilterClass: Send + Sync + 'static {
    type Filter: FilterLike;
    type Error: std::error::Error + Send + Sync + 'static;
    type Config: DeserializeOwned;
    fn id(&self) -> ClassId;
    fn meta(&self) -> ClassMeta {
        ClassMeta::from_env()
    }
    fn construct(&self, config: Self::Config) -> Result<Self::Filter, Self::Error>;
}

pub struct AsFilterClass<F>(pub F);

impl<F> Class for AsFilterClass<F>
where
    F: FilterClass,
{
    type Config = <F as FilterClass>::Config;
    type Error = <F as FilterClass>::Error;
    fn id(&self) -> ClassId {
        self.0.id()
    }

    fn meta(&self) -> ClassMeta {
        ClassMeta::default()
    }
    fn instance_type(&self) -> InstanceType {
        InstanceType::Filter
    }
    fn construct(
        &self,
        config: Self::Config,
    ) -> Result<crate::instance::InstanceValue, Self::Error> {
        let filter = self.0.construct(config)?;
        let filter = Filter::from_filter_like(filter);
        Ok(InstanceValue::Filter(filter))
    }
}
