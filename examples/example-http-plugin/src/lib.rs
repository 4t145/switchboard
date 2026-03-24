use std::convert::Infallible;

use switchboard_http::{
    DynResponse, bytes_body,
    flow::node::{NodeClass, NodeLike},
    instance::class::{plugin::API_VERSION, registry::ClassRegistry},
};
use switchboard_model::services::http::ClassId;
struct HelloWorldClass;
#[derive(serde::Deserialize, serde::Serialize)]
struct HelloWorldClassConfig {
    
}
impl NodeClass for HelloWorldClass {
    type Node = HelloWorld;

    type Error = Infallible;

    type Config = HelloWorldClassConfig;

    fn id(&self) -> ClassId {
        ClassId::new("test", "hello-world")
    }

    fn construct(&self, _config: Self::Config) -> Result<Self::Node, Self::Error> {
        Ok(HelloWorld)
    }
}

pub struct HelloWorld;

impl NodeLike for HelloWorld {
    fn call<'c>(
        &self,
        _req: switchboard_http::DynRequest,
        _context: &'c mut switchboard_http::flow::FlowContext,
    ) -> impl Future<Output = switchboard_http::DynResponse> + 'c + Send {
        async move { return DynResponse::new(bytes_body("hello world")) }
    }

    fn interface(&self) -> switchboard_model::services::http::NodeInterface {
        switchboard_model::services::http::NodeInterface::service()
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn register(register: &mut ClassRegistry, api_version: &'static str) {
    if api_version == API_VERSION {
        register.register_node(HelloWorldClass);
    }
}
