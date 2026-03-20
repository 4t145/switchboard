use gateway_api::constants::{
    GatewayClassConditionType as GatewayClassConditionTypeEnum,
    GatewayConditionReason as GatewayConditionReasonEnum,
    GatewayConditionType as GatewayConditionTypeEnum,
};
use gateway_api::gatewayclasses::GatewayClass;
use gateway_api::gateways::{Gateway, GatewayStatus};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{Condition, Time};
use k8s_openapi::chrono::Utc;
use kube::api::{Patch, PatchParams};
use kube::{Api, ResourceExt};

use crate::ControllerContext;
use crate::run::k8s::ChangeKind;
use crate::utils::k8s::{CONTROLLER_NAME, kube_client_if_in_cluster};

use super::{ObjectKey, trace_reconcile_start};

const CONDITION_STATUS_TRUE: &str = "True";
const CONDITION_STATUS_FALSE: &str = "False";
const MESSAGE_ACCEPTED: &str = "Gateway is accepted by switchboard controller";
const MESSAGE_PROGRAMMED: &str = "Gateway is programmed by switchboard controller";
const MESSAGE_GATEWAY_CLASS_NOT_FOUND: &str =
    "Referenced GatewayClass does not exist for this Gateway";
const MESSAGE_GATEWAY_CLASS_NOT_ACCEPTED: &str =
    "Referenced GatewayClass is not accepted by switchboard controller";
const MESSAGE_APPLY_FAILED: &str = "Gateway configuration apply failed on one or more kernels";

#[derive(Debug, thiserror::Error)]
pub enum GatewayReconcileError {
    #[error("kubernetes runtime environment error: {0}")]
    RuntimeEnv(#[from] crate::utils::k8s::K8sRuntimeEnvError),
    #[error("kubernetes api error: {0}")]
    Kube(#[from] kube::Error),
    #[error("gateway key must contain namespace: {0:?}")]
    MissingNamespace(ObjectKey),
}

pub async fn reconcile(context: &ControllerContext, change: ChangeKind, key: &ObjectKey) {
    trace_reconcile_start("gateway", change, key);

    if let Err(err) = reconcile_inner(context, change, key).await {
        tracing::warn!(
            error = %err,
            namespace = key.namespace.as_deref().unwrap_or("<none>"),
            gateway = %key.name,
            "failed to reconcile gateway"
        );
    }
}

async fn reconcile_inner(
    context: &ControllerContext,
    change: ChangeKind,
    key: &ObjectKey,
) -> Result<(), GatewayReconcileError> {
    let namespace = key
        .namespace
        .clone()
        .ok_or_else(|| GatewayReconcileError::MissingNamespace(key.clone()))?;

    if matches!(change, ChangeKind::Deleted) {
        return Ok(());
    }

    let Some(client) = kube_client_if_in_cluster().await? else {
        return Ok(());
    };

    let gateway_api: Api<Gateway> = Api::namespaced(client.clone(), &namespace);
    let gateway = match gateway_api.get(&key.name).await {
        Ok(gateway) => gateway,
        Err(kube::Error::Api(response)) if response.code == 404 => return Ok(()),
        Err(err) => return Err(err.into()),
    };

    let gateway_class_api: Api<GatewayClass> = Api::all(client);
    let gateway_class = match gateway_class_api.get(&gateway.spec.gateway_class_name).await {
        Ok(gateway_class) => Some(gateway_class),
        Err(kube::Error::Api(response)) if response.code == 404 => None,
        Err(err) => return Err(err.into()),
    };

    let generation = gateway.metadata.generation.unwrap_or(0);
    let apply_status = context.k8s_apply_status.read().await.clone();
    let mut desired_status = GatewayStatus::default();
    desired_status.conditions = Some(build_gateway_conditions(
        generation,
        gateway_class.as_ref(),
        gateway.spec.gateway_class_name.as_str(),
        apply_status.as_ref(),
    ));

    let status_unchanged = gateway
        .status
        .as_ref()
        .is_some_and(|status| status == &desired_status);
    if status_unchanged {
        return Ok(());
    }

    let patch = Patch::Merge(serde_json::json!({
        "status": desired_status,
    }));
    gateway_api
        .patch_status(&key.name, &PatchParams::default(), &patch)
        .await?;

    Ok(())
}

fn build_gateway_conditions(
    generation: i64,
    gateway_class: Option<&GatewayClass>,
    gateway_class_name: &str,
    apply_status: Option<&crate::run::k8s::K8sApplyStatus>,
) -> Vec<Condition> {
    let Some(gateway_class) = gateway_class else {
        return vec![new_gateway_condition(
            GatewayConditionTypeEnum::Accepted,
            CONDITION_STATUS_FALSE,
            GatewayConditionReasonEnum::Invalid,
            generation,
            &format!(
                "{}: {}",
                MESSAGE_GATEWAY_CLASS_NOT_FOUND, gateway_class_name
            ),
        )];
    };

    if gateway_class.spec.controller_name != CONTROLLER_NAME {
        return vec![new_gateway_condition(
            GatewayConditionTypeEnum::Accepted,
            CONDITION_STATUS_FALSE,
            GatewayConditionReasonEnum::InvalidParameters,
            generation,
            MESSAGE_GATEWAY_CLASS_NOT_ACCEPTED,
        )];
    }

    let Some(accepted_condition) = gateway_class
        .status
        .as_ref()
        .and_then(|status| status.conditions.as_ref())
        .and_then(|conditions| {
            conditions
                .iter()
                .find(|condition| condition.type_ == GatewayClassConditionTypeEnum::Accepted.to_string())
        })
    else {
        return vec![new_gateway_condition(
            GatewayConditionTypeEnum::Accepted,
            CONDITION_STATUS_FALSE,
            GatewayConditionReasonEnum::Pending,
            generation,
            MESSAGE_GATEWAY_CLASS_NOT_ACCEPTED,
        )];
    };

    if accepted_condition.status != CONDITION_STATUS_TRUE {
        return vec![new_gateway_condition(
            GatewayConditionTypeEnum::Accepted,
            CONDITION_STATUS_FALSE,
            GatewayConditionReasonEnum::InvalidParameters,
            generation,
            MESSAGE_GATEWAY_CLASS_NOT_ACCEPTED,
        )];
    }

    let programmed = if apply_status.is_some_and(|status| !status.last_apply_succeeded) {
        new_gateway_condition(
            GatewayConditionTypeEnum::Programmed,
            CONDITION_STATUS_FALSE,
            GatewayConditionReasonEnum::Invalid,
            generation,
            MESSAGE_APPLY_FAILED,
        )
    } else {
        new_gateway_condition(
            GatewayConditionTypeEnum::Programmed,
            CONDITION_STATUS_TRUE,
            GatewayConditionReasonEnum::Programmed,
            generation,
            MESSAGE_PROGRAMMED,
        )
    };

    vec![
        new_gateway_condition(
            GatewayConditionTypeEnum::Accepted,
            CONDITION_STATUS_TRUE,
            GatewayConditionReasonEnum::Accepted,
            generation,
            MESSAGE_ACCEPTED,
        ),
        programmed,
    ]
}

pub async fn reconcile_programmed_from_apply_status(context: &ControllerContext) {
    if let Err(err) = reconcile_programmed_from_apply_status_inner(context).await {
        tracing::warn!(error = %err, "failed to refresh gateway programmed status from apply result");
    }
}

async fn reconcile_programmed_from_apply_status_inner(
    context: &ControllerContext,
) -> Result<(), GatewayReconcileError> {
    let Some(client) = kube_client_if_in_cluster().await? else {
        return Ok(());
    };
    let apply_status = context.k8s_apply_status.read().await.clone();

    let gateway_api: Api<Gateway> = Api::all(client.clone());
    let gateways = gateway_api.list(&Default::default()).await?;
    for gateway in gateways.items {
        let Some(namespace) = gateway.namespace() else {
            continue;
        };
        let key = ObjectKey {
            namespace: Some(namespace),
            name: gateway.name_any(),
            uid: gateway.metadata.uid.clone(),
            generation: gateway.metadata.generation,
        };
        let _ = reconcile_gateway_with_apply_status(&client, &key, apply_status.as_ref()).await;
    }
    Ok(())
}

async fn reconcile_gateway_with_apply_status(
    client: &kube::Client,
    key: &ObjectKey,
    apply_status: Option<&crate::run::k8s::K8sApplyStatus>,
) -> Result<(), GatewayReconcileError> {
    let namespace = key
        .namespace
        .clone()
        .ok_or_else(|| GatewayReconcileError::MissingNamespace(key.clone()))?;

    let gateway_api: Api<Gateway> = Api::namespaced(client.clone(), &namespace);
    let gateway = match gateway_api.get(&key.name).await {
        Ok(gateway) => gateway,
        Err(kube::Error::Api(response)) if response.code == 404 => return Ok(()),
        Err(err) => return Err(err.into()),
    };

    let gateway_class_api: Api<GatewayClass> = Api::all(client.clone());
    let gateway_class = match gateway_class_api.get(&gateway.spec.gateway_class_name).await {
        Ok(gateway_class) => Some(gateway_class),
        Err(kube::Error::Api(response)) if response.code == 404 => None,
        Err(err) => return Err(err.into()),
    };

    let generation = gateway.metadata.generation.unwrap_or(0);
    let mut desired_status = GatewayStatus::default();
    desired_status.conditions = Some(build_gateway_conditions(
        generation,
        gateway_class.as_ref(),
        gateway.spec.gateway_class_name.as_str(),
        apply_status,
    ));

    let status_unchanged = gateway
        .status
        .as_ref()
        .is_some_and(|status| status == &desired_status);
    if status_unchanged {
        return Ok(());
    }

    let patch = Patch::Merge(serde_json::json!({
        "status": desired_status,
    }));
    gateway_api
        .patch_status(&key.name, &PatchParams::default(), &patch)
        .await?;

    Ok(())
}

fn new_gateway_condition(
    condition_type: GatewayConditionTypeEnum,
    status: &'static str,
    reason: GatewayConditionReasonEnum,
    observed_generation: i64,
    message: &str,
) -> Condition {
    Condition {
        type_: condition_type.to_string(),
        status: status.to_string(),
        reason: reason.to_string(),
        observed_generation: Some(observed_generation),
        message: message.to_string(),
        last_transition_time: Time(Utc::now()),
    }
}
