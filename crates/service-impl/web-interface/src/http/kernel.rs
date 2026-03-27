use super::{config, state, HttpState};

pub fn router() -> axum::Router<HttpState> {
    axum::Router::new()
        .nest("/state", state::router())
        .nest("/config", config::router())
}
