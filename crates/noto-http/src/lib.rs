// notochord 
pub mod layer;
pub mod response;
pub mod service;
pub mod utils;
use hyper::{
    body::{Body, Incoming},
    rt::{Read, Write},
    server::conn::{http1, http2},
    service::HttpService,
};
use std::error::Error as StdError;

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
