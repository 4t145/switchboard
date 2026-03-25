use std::{collections::HashMap, sync::OnceLock};

use k8s_openapi::api::core::v1::Service;
use kube::{Api, ResourceExt, api::ListParams};
use switchboard_model::discovery::{DiscoveryConnectionInfo, DiscoveryInfo};

use crate::kernel::{DiscoveredKernel, KernelAddr, KernelDiscoveryError, KernelGrpcConnection};

const SBK_LABEL_APP_NAME_KEY: &str = "app.kubernetes.io/name";
const SBK_LABEL_APP_NAME_VALUE: &str = "sbk";
const SBK_LABEL_DISCOVERY_KEY: &str = "switchboard.rs/discovery";
const SBK_LABEL_DISCOVERY_VALUE: &str = "enabled";
fn sbk_label_selector() -> &'static str {
    pub static SELECTOR: OnceLock<String> = OnceLock::new();
    SELECTOR.get_or_init(||format!(
        "{SBK_LABEL_APP_NAME_KEY}={SBK_LABEL_APP_NAME_VALUE},{SBK_LABEL_DISCOVERY_KEY}={SBK_LABEL_DISCOVERY_VALUE}"
    ))
}
const SBK_SERVICE_GRPC_PORT_NAME: &str = "grpc";
const K8S_SERVICE_DNS_SUFFIX: &str = "svc";
const SBK_FALLBACK_GRPC_PORT: u16 = switchboard_model::kernel::HTTP_DEFAULT_PORT;

fn selected_service_port(service: &Service) -> u16 {
    let Some(spec) = service.spec.as_ref() else {
        return SBK_FALLBACK_GRPC_PORT;
    };
    let Some(ports) = spec.ports.as_ref() else {
        return SBK_FALLBACK_GRPC_PORT;
    };

    if let Some(port) = ports
        .iter()
        .find(|port| port.name.as_deref() == Some(SBK_SERVICE_GRPC_PORT_NAME))
        .or_else(|| ports.first())
        .and_then(|port| u16::try_from(port.port).ok())
    {
        return port;
    }

    SBK_FALLBACK_GRPC_PORT
}

fn build_service_endpoint(service_name: &str, namespace: &str, port: u16) -> String {
    format!("grpc://{service_name}.{namespace}.{K8S_SERVICE_DNS_SUFFIX}:{port}")
}

pub async fn scan_k8s_kernels() -> Result<HashMap<String, DiscoveredKernel>, KernelDiscoveryError> {
    let mut kernels = HashMap::new();
    let Some(client) = crate::utils::k8s::kube_client_if_in_cluster().await? else {
        return Ok(kernels);
    };
    let sbk_label_selector = sbk_label_selector();
    tracing::debug!(
        selector = sbk_label_selector,
        "Scanning kubernetes services for sbk kernels"
    );

    let service_api: Api<Service> = Api::all(client.clone());
    let service_list = service_api
        .list(&ListParams::default().labels(sbk_label_selector))
        .await?;

    for service in service_list.items {
        let Some(namespace) = service.namespace() else {
            continue;
        };
        let service_name = service.name_any();
        let port = selected_service_port(&service);
        let endpoint = build_service_endpoint(&service_name, &namespace, port);
        let addr = KernelAddr::Grpc(endpoint.clone().into());

        let Ok(connection) = KernelGrpcConnection::connect(addr.clone())
            .await
            .inspect_err(|error| {
                tracing::warn!(
                    service = service_name,
                    namespace,
                    endpoint,
                    "Skip non-kernel or unreachable sbk candidate: {error}"
                )
            })
        else {
            continue;
        };

        let kernel_info = connection.get_info_from_cache();
        let kernel_id = kernel_info.id.clone();
        let discovered = DiscoveredKernel {
            addr,
            info: DiscoveryInfo {
                connection: DiscoveryConnectionInfo {
                    grpc: endpoint.clone(),
                },
                kernel: kernel_info,
            },
        };

        if kernels.insert(kernel_id.clone(), discovered).is_some() {
            tracing::warn!(
                kernel_id,
                endpoint,
                "Found duplicated kernel id while scanning kubernetes"
            );
        }
    }

    Ok(kernels)
}
