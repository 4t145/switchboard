// pub mod file;
// pub mod k8s;

// use std::collections::BTreeMap;

// use serde::{Deserialize, Serialize};
// use switchboard_model::TlsCertParams;

// use crate::resource::tls::{k8s::K8sTlsCertResource};
// #[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default, Hash, PartialEq, Eq)]
// pub struct TlsResourceConfig {
//     pub k8s: Option<k8s::K8sTlsResourceConfig>,
//     pub file: Option<file::FileTlsResourceConfig>,
// }

// pub enum TlsCertError {}

// pub struct TlsCertProviderInfo {
//     pub name: String,
//     pub description: String,
// }

// impl crate::ControllerContext {
//     pub async fn discovery_tls_certs(
//         &self,
//     ) -> Result<BTreeMap<String, TlsCertResource>, TlsCertsDiscoveryError> {
//         let mut results = BTreeMap::new();

//         // Discover from file
//         {
//             results.extend(
//                 self.discovery_tls_resolver_from_file()
//                     .await?
//                     .into_iter()
//                     .map(|(k, v)| (k, TlsCertResource::File(v))),
//             );
//         }

//         // Discover from k8s
//         {
//             results.extend(
//                 self.discovery_tls_cert_from_k8s()
//                     .await?
//                     .into_iter()
//                     .map(|(k, v)| (k, TlsCertResource::K8s(v))),
//             );
//         }

//         Ok(results)
//     }

//     pub async fn fetch_tls_cert_params(
//         &self,
//         resource: &TlsCertResource,
//     ) -> Result<TlsCertParams, TlsCertsDiscoveryError> {
//         match resource {
//             TlsCertResource::File(file_resource) => {
//                 let params = file_resource.fetch().await?;
//                 Ok(params)
//             }
//             TlsCertResource::K8s(k8s_resource) => {
//                 let params = k8s_resource.fetch(self).await?;
//                 Ok(params)
//             }
//         }
//     }
// }

// #[derive(Debug, thiserror::Error)]
// pub enum TlsCertsDiscoveryError {
//     #[error("File error: {0}")]
//     FileCertError(#[from] file::FileTlsCertResourceError),
//     #[error("K8s error: {0}")]
//     K8sCertError(#[from] k8s::K8sTlsCertResourceError),
// }

// #[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
// pub enum TlsCertResource {
//     File(FileTlsCertResource),
//     K8s(K8sTlsCertResource),
// }
