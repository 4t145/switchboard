use std::collections::BTreeMap;

use serde::{Deserialize, Serialize, Serializer};
use switchboard_model::*;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WithName<T> {
    pub name: String,
    #[serde(flatten)]
    pub data: T,
}

impl<T> WithName<T> {
    pub fn into_inner(self) -> (String, T) {
        (self.name, self.data)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileConfig {
    #[serde(default)]
    pub named_services: Vec<NamedService>,
    #[serde(default)]
    pub binds: Vec<WithName<Bind>>,
    #[serde(default)]
    pub enabled: Vec<String>,
    #[serde(default)]
    pub tls: Vec<WithName<Tls>>,
}

impl FileConfig {
    pub fn into_model_config(self) -> Config {
        Config {
            named_services: self
                .named_services
                .into_iter()
                .map(|ns| (ns.name.clone(), ns))
                .collect(),
            binds: self
                .binds
                .into_iter()
                .map(|b| b.into_inner())
                .collect::<BTreeMap<_, _>>(),
            enabled: self.enabled.into_iter().collect(),
            tls: self
                .tls
                .into_iter()
                .map(|t| t.into_inner())
                .collect::<BTreeMap<_, _>>(),
        }
    }
    pub fn from_model_config(config: Config) -> Self {
        FileConfig {
            named_services: config
                .named_services
                .into_iter()
                .map(|(_, ns)| ns)
                .collect(),
            binds: config
                .binds
                .into_iter()
                .map(|(name, bind)| WithName { name, data: bind })
                .collect(),
            enabled: config.enabled.into_iter().collect(),
            tls: config
                .tls
                .into_iter()
                .map(|(name, tls)| WithName { name, data: tls })
                .collect(),
        }
    }
}

pub fn serialize<S: Serializer>(
    data: &Config,
    serializer: S,
) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> {
    let file_config = FileConfig::from_model_config(data.clone());
    file_config.serialize(serializer)
}

pub fn deserialize<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Config, D::Error> {
    let file_config = FileConfig::deserialize(deserializer)?;
    Ok(file_config.into_model_config())
}
