use std::{
    collections::BTreeMap,
    net::Ipv4Addr,
    sync::{Arc, atomic::AtomicUsize},
};

use anyhow::Context;
use axum::{
    Json, Router, body::Body, extract::State, http::Request, response::IntoResponse, routing,
};
use tokio::net;

#[tokio::main]
pub async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    let bind_port = std::env::var("BIND_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .context("parse bind port")?;
    let listener = net::TcpListener::bind((Ipv4Addr::LOCALHOST, bind_port))
        .await
        .context("bind tcp listener")?;
    tracing::info!("API server listening on port {}", bind_port);
    let router = router();
    axum::serve(listener, router).await?;
    Ok(())
}

fn router() -> Router<()> {
    let app_context = AppContext::new();
    Router::new()
        .nest(
            "/counter",
            Router::new()
                .route("/increase", routing::post(increase_counter))
                .route("/decrease", routing::post(decrease_counter))
                .route("/get", routing::get(get_counter)),
        )
        .nest(
            "/echo",
            Router::new()
                .route("/body", routing::any(echo_body))
                .route("/parts/{*rest}", routing::any(echo_parts)),
        )
        .fallback(not_found)
        .with_state(app_context)
}

#[derive(Clone, Debug, Default)]
pub struct AppContext {
    pub counter: Arc<AtomicUsize>,
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(AtomicUsize::new(0)),
        }
    }
    pub fn increase_counter(&self) -> usize {
        self.counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
    pub fn decrease_counter(&self) -> usize {
        self.counter
            .fetch_min(1, std::sync::atomic::Ordering::SeqCst)
    }
    pub fn get_counter(&self) -> usize {
        self.counter.load(std::sync::atomic::Ordering::SeqCst)
    }
}

async fn increase_counter(context: State<AppContext>) -> Json<usize> {
    Json(context.increase_counter())
}

async fn decrease_counter(context: State<AppContext>) -> Json<usize> {
    Json(context.decrease_counter())
}

async fn get_counter(context: State<AppContext>) -> Json<usize> {
    Json(context.get_counter())
}

async fn echo_body(request: Request<Body>) -> impl IntoResponse {
    axum::http::Response::builder()
        .status(axum::http::StatusCode::OK)
        .body(request.into_body())
        .expect("should be valid response")
}

#[derive(serde::Serialize)]
pub struct EchoParts {
    headers: BTreeMap<String, String>,
    uri: String,
    method: String,
    http_version: String,
}

impl EchoParts {
    pub fn from_request_parts(parts: &axum::http::request::Parts) -> Self {
        let mut headers = BTreeMap::new();
        for (name, value) in parts.headers.iter() {
            headers.insert(
                name.as_str().to_string(),
                value.to_str().unwrap_or_default().to_string(),
            );
        }
        Self {
            headers,
            uri: parts.uri.to_string(),
            method: parts.method.as_str().to_string(),
            http_version: format!("{:?}", parts.version),
        }
    }
}

async fn echo_parts(request: Request<Body>) -> Json<EchoParts> {
    let (parts, _body) = request.into_parts();
    Json(EchoParts::from_request_parts(&parts))
}

async fn not_found() -> impl IntoResponse {
    (axum::http::StatusCode::NOT_FOUND, "Not Found")
}
