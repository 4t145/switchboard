// switchboard 
pub mod layer;
pub mod response;
pub mod service;
pub mod utils;
pub mod node;
pub mod router;

use hyper::{
    body::{Body, Incoming},
    rt::{Read, Write},
    server::conn::{http1, http2},
    service::HttpService,
};
use rustls::ServerConfig;
use std::{error::Error as StdError, sync::Arc};

async fn serve<S>(stream: impl Read + Write + Unpin, service: S)
where
    S: HttpService<Incoming>,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
    S::ResBody: 'static,
    <S::ResBody as Body>::Error: Into<Box<dyn StdError + Send + Sync>>,
{
    let stream = http1::Builder::new()
        .serve_connection(stream, service)
        .await;
}

pub enum HttpVersion {
    Http1,
    Http2,
    Auto,
}

pub enum Tls {
    Tls { config: Arc<ServerConfig> },
    NoTls,
    Auto { config: Arc<ServerConfig> },
}

pub struct Http<S: HttpService<Incoming>> {
    version: HttpVersion,
    tls: Tls,
    service: S,
}
