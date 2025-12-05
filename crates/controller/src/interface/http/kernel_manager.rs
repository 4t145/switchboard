use std::collections::{BTreeMap, HashMap};

use axum::{Json, extract::State};
use switchboard_model::kernel::KernelConnectionAndState;

use crate::{interface::http::HttpState, kernel::KernelAddr};



pub async fn get_kernel_states(State(state): State<HttpState>) -> Json<BTreeMap<KernelAddr, KernelConnectionAndState>> {
    let kernel_manager = state.controller_context.kernel_manager.read().await;
    let states = kernel_manager.get_kernel_states().await;
    
    Json(states)
}