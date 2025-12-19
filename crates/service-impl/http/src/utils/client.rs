use tokio::io;

use hyper_rustls::{ConfigBuilderExt, HttpsConnector};
use rustls::ClientConfig;

use hyper_util::{
    client::legacy::{Client as HyperClient, connect::HttpConnector},
    rt::TokioExecutor,
};

use crate::DynBody;
pub type HyperHttpsClient = HyperClient<HttpsConnector<HttpConnector>, DynBody>;
pub fn build_client() -> io::Result<HyperHttpsClient> {
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

pub struct ClientOptions {
    pub pool_idle_timeout: Option<crate::utils::duration_expr::TimeoutDuration>,
    pub https_only: bool,
}

pub fn build_client_with_options(options: ClientOptions) -> io::Result<HyperHttpsClient> {
    let mut client_builder = HyperClient::builder(TokioExecutor::default());
    if let Some(timeout) = options.pool_idle_timeout {
        client_builder.pool_idle_timeout(timeout.as_duration());
    }
    let connector_builder = HttpsConnector::<HttpConnector>::builder().with_tls_config(
        ClientConfig::builder()
            .with_native_roots()?
            .with_no_client_auth(),
    );
    let connector_builder = if options.https_only {
        connector_builder.https_only()
    } else {
        connector_builder.https_or_http()
    };
    let client = client_builder.build(connector_builder.enable_all_versions().build());
    Ok(client)
}

#[cfg(test)]
mod test {
    // use crate::empty_body;

    #[tokio::test]
    async fn test_request_with_host_header() {
        // hyper::client::conn::http1::Builder::new().
        // let client = super::build_client().unwrap();
        // let request = http::Request::builder()
        //     .uri("https://httpbin.org/get")
        //     .body(empty_body())
        //     .unwrap();
        // let response = client.request(request).await.unwrap();
        // assert_eq!(response.status(), http::StatusCode::OK);
    }
}

