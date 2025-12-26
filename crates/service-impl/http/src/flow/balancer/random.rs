use std::collections::BTreeMap;

use rand::{
    SeedableRng,
    distr::{Distribution, weighted::WeightedIndex},
};
use switchboard_model::services::http::NodePort;

use crate::flow::{FlowContext, balancer::BalancerStrategy};
#[derive(Debug)]
pub struct RandomBalancer {
    weights: WeightedIndex<usize>,
    ports: Vec<NodePort>,
}

pub type RandomBalancerConfig = BTreeMap<NodePort, u32>;
impl RandomBalancer {
    pub fn new(
        config: RandomBalancerConfig,
    ) -> Result<RandomBalancer, rand::distr::weighted::Error> {
        let mut port_list = Vec::new();
        let mut weight_list = Vec::new();
        for (port, weight) in config.into_iter() {
            port_list.push(port.clone());
            weight_list.push(weight as usize);
        }
        let dist = WeightedIndex::new(&weight_list)?;
        Ok(Self {
            weights: dist,
            ports: port_list,
        })
    }
}

impl BalancerStrategy for RandomBalancer {
    fn select(
        &self,
        _request_parts: &mut http::request::Parts,
        _context: &mut FlowContext,
    ) -> Option<NodePort> {
        if self.ports.is_empty() {
            return None;
        }
        thread_local! {
            static RNG: std::cell::RefCell<rand::prelude::SmallRng> = std::cell::RefCell::new(rand::prelude::SmallRng::from_os_rng());
        };
        let dist = &self.weights;
        let choice = RNG.with_borrow_mut(|rng| dist.sample(rng));
        Some(self.ports[choice].clone())
    }
}
