use rustls::{
    crypto::CryptoProvider,
    pki_types::{CertificateDer, PrivateKeyDer},
    server::ResolvesServerCert,
    sign::CertifiedKey,
};
use std::sync::{Arc, Once};
use switchboard_model::{Tls, TlsCertParams, TlsResolver};
#[derive(Debug, thiserror::Error)]
pub enum TlsBuildError {
    #[error("Invalid certificate: {error}, when {context}")]
    InvalidKey {
        context: String,
        error: &'static str,
    },
    #[error("Rustls error: {0}")]
    RustlsError(#[from] rustls::Error),
    #[error("No default crypto provider")]
    NoDefaultCryptoProvider,
}
fn ensure_crypto_provider_installed() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}
pub fn build_tls_config(tls_config: Tls) -> Result<Arc<rustls::ServerConfig>, TlsBuildError> {
    ensure_crypto_provider_installed();
    let resolver = build_resolver(tls_config.resolver)?;
    let mut config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_cert_resolver(resolver);
    config.alpn_protocols = tls_config
        .options
        .alpn_protocols
        .into_iter()
        .map(|s| s.into_bytes())
        .collect();
    config.ignore_client_order = tls_config.options.ignore_client_order;
    config.max_early_data_size = tls_config.options.max_early_data_size;
    config.enable_secret_extraction = tls_config.options.enable_secret_extraction;
    config.max_fragment_size = tls_config.options.max_fragment_size.map(|x| x as usize);
    config.send_half_rtt_data = tls_config.options.send_half_rtt_data;
    config.send_tls13_tickets = tls_config.options.send_tls13_tickets as usize;
    config.require_ems = tls_config.options.require_ems;
    Ok(Arc::new(config))
}

struct TlsCkParams {
    pub certs: Vec<CertificateDer<'static>>,
    pub key: PrivateKeyDer<'static>,
    pub ocsp: Option<Vec<u8>>,
}

fn build_resolver(resolver: TlsResolver) -> Result<Arc<dyn ResolvesServerCert>, TlsBuildError> {
    let provider = rustls::crypto::CryptoProvider::get_default().expect("should installed");
    match resolver {
        TlsResolver::Sni(items) => {
            let items = items
                .into_iter()
                .map(|(hostname, params)| {
                    let params =
                        convert_tls_param(params).map_err(|error| TlsBuildError::InvalidKey {
                            context: format!("building key for {hostname}"),
                            error,
                        })?;
                    Ok((hostname, params))
                })
                .collect::<Result<Vec<_>, TlsBuildError>>()?;
            Ok(sni_resolver(provider, items.into_iter())?)
        }
        TlsResolver::Single(params) => {
            let params = convert_tls_param(params).map_err(|error| TlsBuildError::InvalidKey {
                context: format!("building key"),
                error,
            })?;
            Ok(single_resolver(provider, params)?)
        }
    }
}
fn convert_tls_param(params: TlsCertParams) -> Result<TlsCkParams, &'static str> {
    let key = PrivateKeyDer::try_from(params.key)?;
    let certs = params.certs.into_iter().map(CertificateDer::from).collect();
    let ocsp = params.ocsp;
    Ok(TlsCkParams { certs, key, ocsp })
}
fn sni_resolver<'s>(
    provider: &Arc<CryptoProvider>,
    items: impl Iterator<Item = (String, TlsCkParams)>,
) -> Result<Arc<dyn ResolvesServerCert>, rustls::Error> {
    let mut resolver = rustls::server::ResolvesServerCertUsingSni::new();
    for (hostname, ck_params) in items {
        resolver.add(hostname.as_str(), ck(provider, ck_params)?)?;
    }
    Ok(Arc::new(resolver))
}

fn single_resolver(
    provider: &Arc<CryptoProvider>,
    tuple: TlsCkParams,
) -> Result<Arc<dyn ResolvesServerCert>, rustls::Error> {
    let resolver = rustls::sign::SingleCertAndKey::from(ck(provider, tuple)?);
    Ok(Arc::new(resolver))
}

fn ck(
    provider: &Arc<CryptoProvider>,
    TlsCkParams {
        certs,
        key: pk,
        ocsp,
    }: TlsCkParams,
) -> Result<CertifiedKey, rustls::Error> {
    let signed_key = provider.key_provider.load_private_key(pk)?;
    let mut ck = rustls::sign::CertifiedKey::new(certs, signed_key);
    ck.ocsp = ocsp;
    Ok(ck)
}
