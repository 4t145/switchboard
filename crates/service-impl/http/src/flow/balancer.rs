use std::{collections::BTreeMap, sync::Arc};

use http::StatusCode;
use switchboard_model::services::http::{
    NodeInterface, NodeOutput, NodePort, WithOutputs, consts::BALANCER_CLASS_ID,
};

use crate::{
    DynRequest, DynResponse,
    consts::ERROR_BALANCER,
    flow::node::{NodeClass, NodeLike},
    utils::error_response,
};

mod ip_hash;
mod random;
mod round_robin;
#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum WeightedPortsConfig {
    List(Vec<NodePort>),
    Map(BTreeMap<NodePort, u32>),
}

impl WeightedPortsConfig {
    pub fn to_map(self) -> BTreeMap<NodePort, u32> {
        match self {
            WeightedPortsConfig::List(ports) => ports.into_iter().map(|p| (p, 1)).collect(),
            WeightedPortsConfig::Map(map) => map,
        }
    }
}

pub struct Balancer {
    pub strategy: Arc<dyn BalancerStrategy>,
    pub outputs: BTreeMap<NodePort, NodeOutput>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum BalancerConfig {
    RoundRobin(WeightedPortsConfig),
    Random(WeightedPortsConfig),
    IpHash(WeightedPortsConfig),
}

#[derive(Debug, thiserror::Error)]
pub enum BalancerBuildError {
    #[error("Failed to build random balancer: {0}")]
    Random(#[from] rand::distr::weighted::Error),
}
pub trait BalancerStrategy: Send + Sync + 'static {
    fn select(
        &self,
        request_parts: &mut http::request::Parts,
        context: &mut super::FlowContext,
    ) -> Option<NodePort>;
    #[allow(unused_variables)]
    fn resolve(
        &self,
        port: NodePort,
        response_parts: &mut http::response::Parts,
        context: &mut super::FlowContext,
    ) {
        // Default implementation does nothing
    }
}

impl NodeLike for Balancer {
    fn call<'c>(
        &self,
        req: DynRequest,
        context: &'c mut super::FlowContext,
    ) -> impl Future<Output = crate::DynResponse> + 'c + Send {
        let (mut parts, body) = req.into_parts();
        let port = self.strategy.select(&mut parts, context);
        let strategy = self.strategy.clone();
        if let Some(port) = port {
            let req = DynRequest::from_parts(parts, body);
            return futures::future::Either::Left(async move {
                let response = context.call(req, port.clone()).await;
                let (mut response_parts, body) = response.into_parts();
                strategy.resolve(port, &mut response_parts, context);
                DynResponse::from_parts(response_parts, body)
            });
        } else {
            return futures::future::Either::Right(futures::future::ready(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "No available backend service",
                ERROR_BALANCER,
            )));
        }
    }
    fn interface(&self) -> NodeInterface {
        NodeInterface::with_default_input(self.outputs.clone())
    }
}

pub struct BalancerClass;

impl NodeClass for BalancerClass {
    type Config = WithOutputs<BalancerConfig>;
    type Error = BalancerBuildError;
    type Node = Balancer;
    fn construct(&self, config: Self::Config) -> Result<Self::Node, Self::Error> {
        let outputs = config.output;
        match config.config {
            BalancerConfig::RoundRobin(ports_config) => {
                let rr = round_robin::RoundRobinBalancer::new(ports_config.to_map());
                Ok(Balancer {
                    strategy: Arc::new(rr),
                    outputs,
                })
            }
            BalancerConfig::Random(ports_config) => {
                let random = random::RandomBalancer::new(ports_config.to_map())?;
                Ok(Balancer {
                    strategy: Arc::new(random),
                    outputs,
                })
            }
            BalancerConfig::IpHash(ports_config) => {
                let ip_hash = ip_hash::IpHashBalancer::new(ports_config.to_map());
                Ok(Balancer {
                    strategy: Arc::new(ip_hash),
                    outputs,
                })
            }
        }
    }

    fn id(&self) -> switchboard_model::services::http::ClassId {
        switchboard_model::services::http::ClassId::std(BALANCER_CLASS_ID)
    }
}
