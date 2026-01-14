use std::{collections::BTreeMap, sync::atomic::AtomicUsize};

use switchboard_model::services::http::NodePort;

use crate::flow::{FlowContext, balancer::BalancerStrategy};

pub type RoundRobinBalancerConfig = BTreeMap<NodePort, u32>;

#[derive(Debug)]
pub struct RoundRobinBalancer {
    weights: Vec<(usize, NodePort)>,
    current_index: AtomicUsize,
    current_position: AtomicUsize,
}

impl RoundRobinBalancer {
    pub fn new(config: RoundRobinBalancerConfig) -> Self {
        Self {
            weights: config
                .into_iter()
                .map(|(port, weight)| (weight as usize, port))
                .collect(),
            current_index: AtomicUsize::new(0),
            current_position: AtomicUsize::new(0),
        }
    }
}

impl BalancerStrategy for RoundRobinBalancer {
    fn select(
        &self,
        _request_parts: &mut http::request::Parts,
        _context: &mut FlowContext,
    ) -> Option<NodePort> {
        let backend_count = self.weights.len();
        if backend_count == 0 {
            return None;
        } else if backend_count == 1 {
            return Some(self.weights[0].1.clone());
        } else {
            let current_index = self
                .current_index
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let current_position = self
                .current_position
                .load(std::sync::atomic::Ordering::Relaxed);
            let (current_weight, port) = self.weights.get(current_position)?;
            if *current_weight <= current_index {
                let next_position = (current_position + 1) % backend_count;
                self.current_position
                    .store(next_position, std::sync::atomic::Ordering::Relaxed);
                self.current_index
                    .store(0, std::sync::atomic::Ordering::Relaxed);
            }
            Some(port.clone())
        }
    }
}
