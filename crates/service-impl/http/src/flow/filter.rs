use std::sync::Arc;

use crate::{DynRequest, DynResponse, flow::FlowContext};

pub struct Next<'c> {
    pub context: &'c mut FlowContext,
}

pub struct SharedFilter {
    // pub filter: Arc<dyn Filter + Send + Sync>,
}

pub trait Filter {
    fn call<'c>(
        self: Arc<Self>,
        req: DynRequest,
        context: Next<'c>,
    ) -> impl futures::Future<Output = DynResponse> + 'c + Send;
}
