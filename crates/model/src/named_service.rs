use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Hash, bincode::Encode, bincode::Decode, PartialEq, Eq)]
#[derive(bon::Builder)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct NamedService {
    pub provider: String,
    pub name: String,
    pub config: Option<String>,
    pub description: Option<String>,
    pub tls: Option<String>,
}
