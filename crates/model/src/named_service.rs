use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[derive(bon::Builder)]
#[builder(on(String, into))]
pub struct NamedService {
    pub provider: String,
    pub name: String,
    pub config: Option<String>,
    pub description: Option<String>,
}
