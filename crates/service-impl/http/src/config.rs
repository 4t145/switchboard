use std::{
    collections::{BTreeMap, HashMap},
    time::Duration,
};

use hyper::server::conn::{http1, http2};
use hyper_util::server::conn::auto::Http1Builder;
use serde::{Deserialize, Serialize};

use crate::{
    HttpVersion,
    object::{
        ObjectId,
        orchestration::{Orchestration, OrchestrationContext, OrchestrationError},
        registry::{ObjectClassRegistry, ObjectRegistry},
    },
    service::dynamic::SharedService,
};
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub object: ObjectRegistry,
    pub server: HashMap<ObjectId, ServerConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub version: HttpVersion,
}

impl Config {
    pub fn build_services(
        &self,
        class_reg: &ObjectClassRegistry,
    ) -> Result<HashMap<ObjectId, SharedService>, OrchestrationError> {
        let mut orchestration = Orchestration::default();
        let mut context = OrchestrationContext::new(class_reg, &self.object);
        orchestration.rebuild_all_target(&mut context)?;
        let services = orchestration.build_entries(self.server.keys(), &mut context)?;
        Ok(services)
    }
}
