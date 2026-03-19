use crate::ControllerContext;

use super::{ChangeKind, ObjectKey, trace_reconcile_start};

pub async fn reconcile(_context: &ControllerContext, change: ChangeKind, key: &ObjectKey) {
    trace_reconcile_start("tlsroute", change, key);
}
