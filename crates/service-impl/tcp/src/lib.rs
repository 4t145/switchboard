pub mod balancer;
pub mod inbound;
pub mod outbound;
use std::{collections::HashMap, net::SocketAddr, pin::Pin, sync::Arc};
use switchboard_http_router::hostname::HostnameTree;

use crate::outbound::Outbound;
use switchboard_service::{
    SerdeValue, SerdeValueError, TcpServiceProvider,
    tcp::{AsyncStream, TcpService},
};
use tokio::{
    io::{self, AsyncWriteExt},
    net::{TcpStream, ToSocketAddrs},
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
#[non_exhaustive]
#[serde(tag = "strategy", content = "outbound")]
pub enum TlsStrategyConfig {
    Passthrough(HashMap<String, Outbound>),
    Terminate(Outbound),
    // ReEncrypt,
}
#[derive(Debug, Clone)]
pub enum TlsStrategy {
    Passthrough(HostnameTree<Outbound>),
    Terminate(Outbound),
    // ReEncrypt,
}

impl From<TlsStrategyConfig> for TlsStrategy {
    fn from(config: TlsStrategyConfig) -> Self {
        match config {
            TlsStrategyConfig::Passthrough(map) => {
                let tree = HostnameTree::from_iter(map.into_iter());
                TlsStrategy::Passthrough(tree)
            }
            TlsStrategyConfig::Terminate(outbound) => TlsStrategy::Terminate(outbound),
        }
    }
}
#[derive(Debug, Clone)]
pub struct Tcp {
    pub strategy_config: TlsStrategy,
    pub balancer_strategy: Arc<dyn balancer::BalancerStrategy>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
pub struct TcpConfig {
    #[serde(flatten)]
    pub strategy_config: TlsStrategyConfig,
    pub balancer_strategy: balancer::BalancerStrategyConfig,
}

// pub enum StrategyData {
//     Passthrough {
//         client_hello: Option<OwnedClientHello>,
//     },
//     Terminate {
//         acceptor: Option<TlsAcceptor>,
//     },
// }

pub struct TcpConnectionInfo {
    // pub strategy_data: StrategyData,
    pub from: SocketAddr,
}

impl Tcp {
    async fn serve_inner<S>(
        self: Arc<Self>,
        accepted: switchboard_service::tcp::TcpAccepted<S>,
    ) -> io::Result<()>
    where
        S: AsyncStream,
    {
        match &self.strategy_config {
            TlsStrategy::Passthrough(sni_router) => {
                let accepted = accepted.maybe_tls_passthrough().await?;
                let switchboard_service::tcp::TcpAccepted {
                    mut stream,
                    context,
                } = accepted;
                let from = context.peer_addr;
                let Some(sni_outbound) = context
                    .tls_client_hello
                    .as_ref()
                    .and_then(|h| h.server_name.as_deref())
                    .and_then(|sni| sni_router.get(sni))
                else {
                    // terminate connection if SNI is not found or client hello is not present
                    tracing::debug!(%from, "no matching SNI found, connection closed");
                    stream.shutdown().await?;
                    return Ok(());
                };
                let info = TcpConnectionInfo { from };
                let ct = context.ct.clone();
                let outbound = match &sni_outbound {
                    Outbound::NamedMap(map) => self.balancer_strategy.dispatch(map, &info),
                    Outbound::Single(outbound) => Some(outbound),
                };
                let Some(outbound) = outbound else {
                    tracing::debug!(%from, "no matching outbound selected, connection closed");
                    stream.shutdown().await?;
                    return Ok(());
                };
                tokio::select! {
                    _ = ct.cancelled() => {
                        tracing::debug!(%from, "connection cancelled before forwarding");
                        return Ok(());
                    }
                    res = forward_tcp(stream, from, outbound.socket_addr()) => {
                        if let Err(e) = res {
                            tracing::error!(%from, %e, "error forwarding connection");
                            return Err(e);
                        } else {
                            tracing::debug!(%from, "connection forwarded finished");
                            return Ok(());
                        }
                    }
                }
            }
            TlsStrategy::Terminate(outbounds) => {
                let accepted = accepted.maybe_tls_terminate().await?;
                let switchboard_service::tcp::TcpAccepted { stream, context } = accepted;
                let from = context.peer_addr;
                let info = TcpConnectionInfo { from };
                let ct = context.ct.clone();
                let outbound = match &outbounds {
                    Outbound::NamedMap(map) => self.balancer_strategy.dispatch(map, &info),
                    Outbound::Single(outbound) => Some(outbound),
                };
                let Some(outbound) = outbound else {
                    tracing::debug!(%from, "no matching outbound selected, connection closed");
                    return Ok(());
                };
                tokio::select! {
                    _ = ct.cancelled() => {
                        tracing::debug!(%from, "connection cancelled before forwarding");
                        return Ok(());
                    }
                    res = forward_tcp(stream, from, outbound.socket_addr()) => {
                        if let Err(e) = res {
                            tracing::error!(%from, %e, "error forwarding connection");
                            return Err(e);
                        } else {
                            tracing::debug!(%from, "connection forwarded finished");
                            return Ok(());
                        }
                    }
                }
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("unsupported TLS strategy: {:?}", self.strategy_config),
                ));
            }
        }
    }
}

impl TcpService for Tcp {
    fn name(&self) -> &str {
        "tcp"
    }
    fn serve(
        self: Arc<Self>,
        accepted: switchboard_service::tcp::TcpAccepted,
    ) -> Pin<Box<dyn Future<Output = io::Result<()>> + 'static + Send>> {
        Box::pin(self.serve_inner(accepted))
    }
}

async fn forward_tcp<T: AsyncStream, A: ToSocketAddrs + std::fmt::Debug>(
    mut inbound: T,
    from: SocketAddr,
    to: A,
) -> io::Result<()> {
    tracing::debug!(%from, ?to, "forward tcp connection");
    let mut out = TcpStream::connect(to).await?;
    tokio::io::copy_bidirectional(&mut inbound, &mut out).await?;
    Ok(())
}

pub struct PortForwardProvider;
impl TcpServiceProvider for PortForwardProvider {
    const NAME: &'static str = "tcp";
    type Service = Tcp;
    type Error = SerdeValueError;

    async fn construct(&self, config: Option<SerdeValue>) -> Result<Self::Service, Self::Error> {
        let config: TcpConfig = config.unwrap_or_default().deserialize_into()?;
        Ok(Tcp {
            strategy_config: config.strategy_config.into(),
            balancer_strategy: config.balancer_strategy.build(),
        })
    }
}
