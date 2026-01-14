use std::{collections::BTreeMap, convert::identity, net::IpAddr};

use switchboard_model::services::http::NodePort;

use crate::flow::balancer::BalancerStrategy;

pub type IpHashBalancerConfig = BTreeMap<NodePort, u32>;
#[derive(Debug)]
pub struct WithAccWeight {
    pub port: NodePort,
    pub acc_weight: u32,
}
#[derive(Debug)]
pub struct IpHashBalancer {
    pub total_weight: u32,
    pub ports: Vec<WithAccWeight>,
}

impl IpHashBalancer {
    pub fn new(config: BTreeMap<NodePort, u32>) -> Self {
        let mut ports = Vec::new();
        let mut acc_weight = 0;
        for (port, weight) in config {
            acc_weight += weight;
            ports.push(WithAccWeight { port, acc_weight });
        }
        ports.sort_by_key(|x| x.acc_weight);
        IpHashBalancer {
            total_weight: acc_weight,
            ports,
        }
    }
    pub fn select_by_hash(&self, hash: u32) -> Option<NodePort> {
        const BINARY_SEARCH_THRESHOLD: usize = 256;
        if self.total_weight == 0 {
            return None;
        }
        let target = hash % self.total_weight;
        let size = self.ports.len();
        if size <= 1 {
            return Some(self.ports[0].port.clone());
        } else if size >= BINARY_SEARCH_THRESHOLD {
            // use binary search for large number of backends
            let index = self
                .ports
                .binary_search_by_key(&target, |x| x.acc_weight)
                .unwrap_or_else(identity)
                .max(size - 1);
            return Some(self.ports[index].port.clone());
        } else {
            for entry in &self.ports {
                if target < entry.acc_weight {
                    return Some(entry.port.clone());
                }
            }
        }
        None
    }
}

fn ip_hash(addr: &IpAddr) -> u64 {
    use std::hash::{Hash, Hasher};
    // use default hasher
    let mut hasher = std::hash::DefaultHasher::new();
    addr.hash(&mut hasher);
    hasher.finish() as u64
}

impl BalancerStrategy for IpHashBalancer {
    fn select(
        &self,
        _request_parts: &mut http::request::Parts,
        context: &mut crate::flow::FlowContext,
    ) -> Option<switchboard_model::services::http::NodePort> {
        let ip = context.connection_info.as_ref()?.peer_addr.ip();
        let hash = ip_hash(&ip);
        self.select_by_hash(hash as u32)
    }
}
