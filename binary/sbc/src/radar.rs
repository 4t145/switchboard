use std::collections::{HashMap, HashSet};

use switchboard_model::Config;

use crate::sbk_discovery::SbkInstance;

pub struct Radar {
    pub config: RadarConfig,
}

pub struct RadarConfig {
    pub event_channel_capacity: usize,
}

impl RadarConfig {
    pub async fn spawn(self)  {
        let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<RadarEvent>(self.event_channel_capacity);
        let event_handle_task = tokio::spawn(async move {
            let mut discovered_sbks: HashMap<String, SbkInstance> = HashMap::new();
            while let Some(event) = event_rx.recv().await {
                match event {
                    RadarEvent::SbkListUpdate(update) => {
                        tracing::info!("received sbk list update: discovered: {:?}, lost: {:?}", update.discovered.keys(), update.lost);
                    }
                    RadarEvent::SbkConfigUpdate(update) => {
                        tracing::info!("received sbk config update: {:?}", update.config);
                    }
                }
            }
        });

    }
}

pub struct SbkConnection {
    pub instance: SbkInstance,
    
}

pub struct SbkListUpdate {
    pub discovered: HashMap<String, SbkInstance>,
    pub lost: HashSet<String>,
}

pub struct SbkConfigUpdate {
    pub config: switchboard_model::Config,
}

pub enum RadarEvent {
    SbkListUpdate(SbkListUpdate),
    SbkConfigUpdate(SbkConfigUpdate),
}
