use std::convert::Infallible;

use futures::future::BoxFuture;
use http::StatusCode;
use http_body_util::BodyExt;
use tokio::io;

use hyper_rustls::{ConfigBuilderExt, HttpsConnector};
use rustls::ClientConfig;

use hyper_util::{
    client::legacy::{Client as HyperClient, connect::HttpConnector},
    rt::TokioExecutor,
};

use crate::{ERR_HTTP_CLIENT, instance::class::*, utils::error_response};

use crate::{DynBody, DynRequest, DynResponse, DynService, box_error};

type HyperHttpsClient = HyperClient<HttpsConnector<HttpConnector>, DynBody>;

pub struct ClientService {
    pub client: HyperHttpsClient,
}
fn build() -> io::Result<HyperHttpsClient> {
    let client = HyperClient::builder(TokioExecutor::default()).build(
        HttpsConnector::<HttpConnector>::builder()
            .with_tls_config(
                ClientConfig::builder()
                    .with_native_roots()?
                    .with_no_client_auth(),
            )
            .https_or_http()
            .enable_all_versions()
            .build(),
    );
    Ok(client)
}

impl DynService for ClientService {
    fn call(&self, req: DynRequest) -> BoxFuture<'static, Result<DynResponse, Infallible>> {
        let client = self.client.clone();
        Box::pin(async move {
            match client.request(req).await {
                Ok(response) => {
                    Ok(response.map(|incoming| incoming.map_err(box_error).boxed_unsync()))
                }
                Err(e) => Ok(error_response(StatusCode::BAD_GATEWAY, e, ERR_HTTP_CLIENT)),
            }
        })
    }
}
