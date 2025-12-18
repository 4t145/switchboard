use std::ops::{Deref, DerefMut};

use switchboard_model::{Listener, ConfigService, TcpServiceConfig, Tls};

pub type BindId = String;
fn next_bind_id() -> String {
    static ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let id = ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    format!("bind-{}", id)
}


#[derive(Clone, Debug, Default)]
pub struct MemConfig {
    pub config: switchboard_model::Config,
}

impl Deref for MemConfig {
    type Target = switchboard_model::Config;
    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl DerefMut for MemConfig {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.config
    }
}

impl MemConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_named_service(&mut self, service: TcpServiceConfig) {
        self.tcp_services.insert(service.name.clone(), service);
    }

    pub fn add_tls(&mut self, name: String, tls: Tls) {
        self.tls.insert(name, tls);
    }

    pub fn into_inner(self) -> switchboard_model::Config {
        self.config
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no such index {0}")]
    NotSuchIndex(String),
}

impl ConfigService for MemConfig {
    type Error = Error;
    async fn fetch_config(&self) -> Result<switchboard_model::Config, Self::Error> {
        Ok(self.config.clone())
    }
}
