use crate::TcpConnectionInfo;

use super::outbound::{OutboundEndpoint, OutboundName};
use rand::{
    SeedableRng,
    distr::{Distribution, weighted::WeightedIndex},
};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    net::IpAddr,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

const DEFAULT_WEIGHT: u32 = 1;

pub trait BalancerStrategy: Send + Sync + 'static + std::fmt::Debug {
    fn dispatch<'a>(
        &self,
        outbounds: &'a HashMap<OutboundName, OutboundEndpoint>,
        connection_info: &TcpConnectionInfo,
    ) -> Option<&'a OutboundEndpoint>;
}

#[derive(
    Debug,
    Clone,
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
    IpHash,
}

impl BalancerStrategyConfig {
    pub fn build(&self) -> Arc<dyn BalancerStrategy> {
        match self {
            Self::RoundRobin => Arc::new(RoundRobinBalancer::default()),
            Self::Random => Arc::new(RandomBalancer),
            Self::IpHash => Arc::new(IpHashBalancer),
        }
    }
}

fn effective_weight(endpoint: &OutboundEndpoint) -> u32 {
    endpoint.weight.unwrap_or(DEFAULT_WEIGHT)
}

fn ip_hash(addr: &IpAddr) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    addr.hash(&mut hasher);
    hasher.finish()
}

#[derive(Debug, Default)]
struct RoundRobinBalancer {
    current: AtomicUsize,
}

impl BalancerStrategy for RoundRobinBalancer {
    fn dispatch<'a>(
        &self,
        outbounds: &'a HashMap<OutboundName, OutboundEndpoint>,
        _connection_info: &TcpConnectionInfo,
    ) -> Option<&'a OutboundEndpoint> {
        let weighted = outbounds
            .values()
            .map(|endpoint| (endpoint, effective_weight(endpoint)))
            .filter(|(_, weight)| *weight > 0)
            .collect::<Vec<_>>();
        if weighted.is_empty() {
            return None;
        }

        let total_weight = weighted
            .iter()
            .map(|(_, weight)| *weight as usize)
            .sum::<usize>();
        if total_weight == 0 {
            return None;
        }
        let mut slot = self.current.fetch_add(1, Ordering::Relaxed) % total_weight;
        for (endpoint, weight) in weighted {
            let weight = weight as usize;
            if slot < weight {
                return Some(endpoint);
            }
            slot -= weight;
        }
        None
    }
}

#[derive(Debug)]
struct RandomBalancer;

impl BalancerStrategy for RandomBalancer {
    fn dispatch<'a>(
        &self,
        outbounds: &'a HashMap<OutboundName, OutboundEndpoint>,
        _connection_info: &TcpConnectionInfo,
    ) -> Option<&'a OutboundEndpoint> {
        let weighted = outbounds
            .values()
            .map(|endpoint| (endpoint, effective_weight(endpoint)))
            .filter(|(_, weight)| *weight > 0)
            .collect::<Vec<_>>();
        if weighted.is_empty() {
            return None;
        }
        if weighted.len() == 1 {
            return Some(weighted[0].0);
        }

        let weights = weighted
            .iter()
            .map(|(_, weight)| *weight as usize)
            .collect::<Vec<_>>();
        let dist = WeightedIndex::new(weights).ok()?;
        thread_local! {
            static RNG: std::cell::RefCell<rand::prelude::SmallRng> = std::cell::RefCell::new(rand::prelude::SmallRng::from_os_rng());
        }
        let idx = RNG.with_borrow_mut(|rng| dist.sample(rng));
        weighted.get(idx).map(|(endpoint, _)| *endpoint)
    }
}

#[derive(Debug)]
struct IpHashBalancer;

impl BalancerStrategy for IpHashBalancer {
    fn dispatch<'a>(
        &self,
        outbounds: &'a HashMap<OutboundName, OutboundEndpoint>,
        connection_info: &TcpConnectionInfo,
    ) -> Option<&'a OutboundEndpoint> {
        let weighted = outbounds
            .values()
            .map(|endpoint| (endpoint, effective_weight(endpoint)))
            .filter(|(_, weight)| *weight > 0)
            .collect::<Vec<_>>();
        if weighted.is_empty() {
            return None;
        }

        let total_weight = weighted.iter().map(|(_, weight)| *weight).sum::<u32>();
        if total_weight == 0 {
            return None;
        }

        let mut target = (ip_hash(&connection_info.from.ip()) as u32) % total_weight;
        for (endpoint, weight) in weighted {
            if target < weight {
                return Some(endpoint);
            }
            target = target.saturating_sub(weight);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TcpConnectionInfo;
    use std::{collections::HashMap, net::SocketAddr};

    fn outbound_name(name: &str) -> OutboundName {
        OutboundName::new(name)
    }

    fn outbound(host: &str, port: u16, weight: Option<u32>) -> OutboundEndpoint {
        OutboundEndpoint {
            host: host.to_string(),
            port,
            weight,
        }
    }

    fn connection_info(ip: &str) -> TcpConnectionInfo {
        TcpConnectionInfo {
            from: format!("{ip}:443")
                .parse::<SocketAddr>()
                .expect("test address must parse"),
        }
    }

    #[test]
    fn round_robin_should_respect_weight() {
        let strategy = BalancerStrategyConfig::RoundRobin.build();
        let outbounds = HashMap::from([
            (outbound_name("a"), outbound("10.0.0.1", 80, Some(2))),
            (outbound_name("b"), outbound("10.0.0.2", 80, Some(1))),
        ]);
        let info = connection_info("192.168.1.10");

        let selected = (0..30)
            .filter_map(|_| strategy.dispatch(&outbounds, &info).map(|o| o.host.clone()))
            .collect::<Vec<_>>();
        let a_count = selected.iter().filter(|h| h.as_str() == "10.0.0.1").count();
        let b_count = selected.iter().filter(|h| h.as_str() == "10.0.0.2").count();

        assert_eq!(a_count, 20);
        assert_eq!(b_count, 10);
    }

    #[test]
    fn random_should_skip_zero_weight() {
        let strategy = BalancerStrategyConfig::Random.build();
        let outbounds = HashMap::from([
            (outbound_name("a"), outbound("10.0.0.1", 80, Some(0))),
            (outbound_name("b"), outbound("10.0.0.2", 80, Some(1))),
            (outbound_name("c"), outbound("10.0.0.3", 80, Some(3))),
        ]);
        let info = connection_info("192.168.1.11");

        for _ in 0..128 {
            let host = strategy
                .dispatch(&outbounds, &info)
                .expect("must choose non-zero weight endpoint")
                .host
                .as_str();
            assert!(host == "10.0.0.2" || host == "10.0.0.3");
        }
    }

    #[test]
    fn ip_hash_should_be_stable_for_same_ip() {
        let strategy = BalancerStrategyConfig::IpHash.build();
        let outbounds = HashMap::from([
            (outbound_name("a"), outbound("10.0.0.1", 80, None)),
            (outbound_name("b"), outbound("10.0.0.2", 80, None)),
        ]);
        let info = connection_info("10.1.2.3");

        let first = strategy
            .dispatch(&outbounds, &info)
            .expect("must choose endpoint")
            .host
            .clone();
        for _ in 0..32 {
            assert_eq!(
                strategy
                    .dispatch(&outbounds, &info)
                    .expect("must choose endpoint")
                    .host,
                first
            );
        }
    }

    #[test]
    fn should_return_none_when_all_weights_disabled() {
        let strategy = BalancerStrategyConfig::Random.build();
        let outbounds = HashMap::from([
            (outbound_name("a"), outbound("10.0.0.1", 80, Some(0))),
            (outbound_name("b"), outbound("10.0.0.2", 80, Some(0))),
        ]);
        let info = connection_info("127.0.0.1");

        assert!(strategy.dispatch(&outbounds, &info).is_none());
    }
}
