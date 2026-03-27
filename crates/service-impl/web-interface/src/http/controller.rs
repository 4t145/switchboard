use super::HttpState;

pub fn router() -> axum::Router<HttpState> {
    axum::Router::new()
}
