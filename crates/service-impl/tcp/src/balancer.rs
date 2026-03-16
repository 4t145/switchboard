use crate::TcpConnectionInfo;

use super::outbound::{Outbound, OutboundName};
use std::{collections::HashMap, sync::Arc};

pub trait BalancerStrategy: Send + Sync + 'static + std::fmt::Debug {
    fn dispatch(
        &self,
        outbounds: &HashMap<OutboundName, Outbound>,
        connection_info: &TcpConnectionInfo,
    ) -> &Outbound;
}

#[derive(
    Debug,
    Clone,
    Hash,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
pub enum BalancerStrategyConfig {
    RoundRobin,
    Random,
}

impl BalancerStrategyConfig {
    pub fn build(&self) -> Arc<dyn BalancerStrategy> {
        todo!()
    }
}
