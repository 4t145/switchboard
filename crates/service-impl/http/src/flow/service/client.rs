
use http::StatusCode;
use http_body_util::BodyExt;
use switchboard_model::services::http::ClassId;

use crate::{
    ERR_HTTP_CLIENT,
    flow::{FlowContext, node::NodeClass, service::ServiceNode},
    utils::{HyperHttpsClient, build_client, error_response},
};

use crate::{DynRequest, DynResponse, box_error};

pub struct ClientService {
    pub client: HyperHttpsClient,
}

impl super::Service for ClientService {
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

pub struct Client;

impl NodeClass for Client {
    type Config = ();
    type Error = std::io::Error;
    type Node = ServiceNode<ClientService>;

    fn construct(&self, _: Self::Config) -> Result<Self::Node, Self::Error> {
        Ok(ServiceNode::new(ClientService {
            client: build_client()?,
        }))
    }

    fn id(&self) -> ClassId {
        ClassId::std("client")
    }
}
