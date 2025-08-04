use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    HttpVersion,
    instance::{
        InstanceId,
        orchestration::{Orchestration, OrchestrationContext, OrchestrationError},
        registry::{ClassRegistry, InstanceRegistry},
    },
    service::dynamic::SharedService,
};
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub object: InstanceRegistry,
    pub server: HashMap<InstanceId, ServerConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub version: HttpVersion,
}

impl Config {
    pub fn build_services(
        &self,
        class_reg: &ClassRegistry,
    ) -> Result<HashMap<InstanceId, SharedService>, OrchestrationError> {
        let mut orchestration = Orchestration::default();
        let mut context = OrchestrationContext::new(class_reg, &self.object);
        orchestration.rebuild_all_target(&mut context)?;
        let services = orchestration.build_entries(self.server.keys(), &mut context)?;
        Ok(services)
    }
}
