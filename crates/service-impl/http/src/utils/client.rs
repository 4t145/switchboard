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
