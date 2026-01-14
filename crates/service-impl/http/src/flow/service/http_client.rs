use http::StatusCode;
use http_body_util::BodyExt;
use switchboard_model::services::http::{ClassId, consts::HTTP_CLIENT_CLASS_ID};

use crate::{
    consts::ERR_HTTP_CLIENT,
    flow::{FlowContext, node::NodeClass, service::ServiceNode},
    utils::{HyperHttpsClient, build_client, error_response},
};

use crate::{DynRequest, DynResponse, box_error};

pub struct HttpClientService {
    pub client: HyperHttpsClient,
}

impl super::Service for HttpClientService {
    fn call<'c>(
        &self,
        req: DynRequest,
        _: &'c mut FlowContext,
    ) -> impl Future<Output = DynResponse> + Send + 'c {
        let client = self.client.clone();
        async move {
            match client.request(req).await {
                Ok(response) => response.map(|incoming| incoming.map_err(box_error).boxed_unsync()),
                Err(e) => error_response(StatusCode::BAD_GATEWAY, e, ERR_HTTP_CLIENT),
            }
        }
    }
}

pub struct HttpClientClass;

impl NodeClass for HttpClientClass {
    type Config = ();
    type Error = std::io::Error;
    type Node = ServiceNode<HttpClientService>;

    fn construct(&self, _: Self::Config) -> Result<Self::Node, Self::Error> {
        Ok(ServiceNode::new(HttpClientService {
            client: build_client()?,
        }))
    }

    fn id(&self) -> ClassId {
        ClassId::std(HTTP_CLIENT_CLASS_ID)
    }
}
