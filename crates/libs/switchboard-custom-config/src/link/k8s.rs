use std::str::FromStr;

/// k8s link format: k8s://namespace/name or k8s://name
#[derive(Debug, Clone, Hash, bincode::Encode, bincode::Decode, PartialEq, Eq)]
pub struct K8sResource {
    pub namespace: Option<String>,
    pub name: String,
}

impl K8sResource {
    pub fn new(namespace: Option<String>, name: String) -> Self {
        Self { namespace, name }
    }
}

impl serde::Serialize for K8sResource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for K8sResource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        K8sResource::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum K8sResourceParseError {
    #[error("Invalid k8s resource name")]
    InvalidK8sName,
}

fn is_valid_k8s_name(name: &str) -> bool {
    // Kubernetes resource names must consist of lower case alphanumeric characters, '-' or '.',
    // and must start and end with an alphanumeric character (e.g. 'my-name',  or '123-abc', regex used for validation is '[a-z0-9]([-a-z0-9]*[a-z0-9])?(\.[a-z0-9]([-a-z0-9]*[a-z0-9])?)*')
    let bytes = name.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    if !bytes[0].is_ascii_lowercase() && !bytes[0].is_ascii_digit() {
        return false;
    }
    if !bytes[bytes.len() - 1].is_ascii_lowercase() && !bytes[bytes.len() - 1].is_ascii_digit() {
        return false;
    }
    for &b in bytes {
        if !(b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'.') {
            return false;
        }
    }
    true
}

impl FromStr for K8sResource {
    type Err = K8sResourceParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((namespace, name)) = s.split_once('/') {
            if !is_valid_k8s_name(namespace) || !is_valid_k8s_name(name) {
                return Err(K8sResourceParseError::InvalidK8sName);
            }
            Ok(K8sResource {
                namespace: Some(namespace.to_string()),
                name: name.to_string(),
            })
        } else {
            if !is_valid_k8s_name(s) {
                return Err(K8sResourceParseError::InvalidK8sName);
            }
            Ok(K8sResource {
                namespace: None,
                name: s.to_string(),
            })
        }
    }
}

impl std::fmt::Display for K8sResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(namespace) = &self.namespace {
            write!(f, "{}/{}", namespace, self.name)
        } else {
            write!(f, "{}", self.name)
        }
    }
}