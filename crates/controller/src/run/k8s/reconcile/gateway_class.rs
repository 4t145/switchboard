use gateway_api::constants::{GatewayClassConditionReason, GatewayClassConditionType};
use gateway_api::gatewayclasses::{GatewayClass, GatewayClassStatus};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{Condition, Time};
use k8s_openapi::chrono::Utc;
use kube::api::ListParams;
use kube::api::{Patch, PatchParams};
use kube::{Api, ResourceExt};

use crate::ControllerContext;
use crate::run::k8s::ChangeKind;
use crate::utils::k8s::{CONTROLLER_NAME, kube_client_if_in_cluster};

use super::{ObjectKey, trace_reconcile_start};

const CONDITION_STATUS_TRUE: &str = "True";
const CONDITION_STATUS_FALSE: &str = "False";
const MESSAGE_ACCEPTED: &str = "GatewayClass is accepted by switchboard controller";
const MESSAGE_UNSUPPORTED_PARAMETERS: &str =
    "GatewayClass parametersRef is not supported by switchboard controller";
const MESSAGE_NOT_TARGET_CONTROLLER: &str =
    "GatewayClass is not managed by this switchboard controller";
const ALL_OBJECTS_KEY: &str = "*";

#[derive(Debug, thiserror::Error)]
pub enum GatewayClassReconcileError {
    #[error("kubernetes runtime environment error: {0}")]
    RuntimeEnv(#[from] crate::utils::k8s::K8sRuntimeEnvError),
    #[error("kubernetes api error: {0}")]
    Kube(#[from] kube::Error),
    #[error("gatewayclass key must not contain namespace: {0:?}")]
    UnexpectedNamespace(ObjectKey),
}

pub async fn reconcile(context: &ControllerContext, change: ChangeKind, key: &ObjectKey) {
    trace_reconcile_start("gatewayclass", change, key);

    if let Err(err) = reconcile_inner(context, change, key).await {
        tracing::warn!(
            error = %err,
            gatewayclass = %key.name,
            "failed to reconcile gatewayclass"
        );
    }
}

async fn reconcile_inner(
    _context: &ControllerContext,
    change: ChangeKind,
    key: &ObjectKey,
) -> Result<(), GatewayClassReconcileError> {
    if key.namespace.is_some() {
        return Err(GatewayClassReconcileError::UnexpectedNamespace(key.clone()));
    }
    if matches!(change, ChangeKind::Deleted) {
        return Ok(());
    }

    let Some(client) = kube_client_if_in_cluster().await? else {
        return Ok(());
    };

    let gateway_class_api: Api<GatewayClass> = Api::all(client);
    if matches!(change, ChangeKind::Restarted) && key.name == ALL_OBJECTS_KEY {
        let gateway_classes = gateway_class_api.list(&ListParams::default()).await?;
        for gateway_class in gateway_classes.items {
            reconcile_gateway_class_status(&gateway_class_api, gateway_class, None).await?;
        }
        return Ok(());
    }

    let gateway_class = match gateway_class_api.get(&key.name).await {
        Ok(gateway_class) => gateway_class,
        Err(kube::Error::Api(response)) if response.code == 404 => return Ok(()),
        Err(err) => return Err(err.into()),
    };

    reconcile_gateway_class_status(&gateway_class_api, gateway_class, key.generation).await?;

    Ok(())
}

async fn reconcile_gateway_class_status(
    gateway_class_api: &Api<GatewayClass>,
    gateway_class: GatewayClass,
    event_generation: Option<i64>,
) -> Result<(), GatewayClassReconcileError> {
    let generation = gateway_class
        .metadata
        .generation
        .or(event_generation)
        .unwrap_or(1)
        .max(1);

    let mut desired_status = GatewayClassStatus::default();
    desired_status.conditions = Some(vec![build_accepted_condition(&gateway_class, generation)]);

    let status_unchanged = gateway_class
        .status
        .as_ref()
        .is_some_and(|status| status == &desired_status);
    if status_unchanged {
        return Ok(());
    }

    let patch = Patch::Merge(serde_json::json!({
        "status": desired_status,
    }));
    gateway_class_api
        .patch_status(&gateway_class.name_any(), &PatchParams::default(), &patch)
        .await?;

    Ok(())
}

fn build_accepted_condition(gateway_class: &GatewayClass, generation: i64) -> Condition {
    if gateway_class.spec.controller_name != CONTROLLER_NAME {
        return new_condition(
            GatewayClassConditionType::Accepted,
            CONDITION_STATUS_FALSE,
            GatewayClassConditionReason::Unsupported,
            generation,
            MESSAGE_NOT_TARGET_CONTROLLER,
        );
    }

    if gateway_class.spec.parameters_ref.is_some() {
        return new_condition(
            GatewayClassConditionType::Accepted,
            CONDITION_STATUS_FALSE,
            GatewayClassConditionReason::InvalidParameters,
            generation,
            MESSAGE_UNSUPPORTED_PARAMETERS,
        );
    }

    new_condition(
        GatewayClassConditionType::Accepted,
        CONDITION_STATUS_TRUE,
        GatewayClassConditionReason::Accepted,
        generation,
        MESSAGE_ACCEPTED,
    )
}

fn new_condition(
    condition_type: GatewayClassConditionType,
    status: &'static str,
    reason: GatewayClassConditionReason,
    observed_generation: i64,
    message: &'static str,
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
