pub mod balancer;
pub mod inbound;
pub mod outbound;

use std::{collections::HashMap, net::SocketAddr, pin::Pin, sync::Arc};

use switchboard_model::strategy::TlsStrategy;
use switchboard_service::{
    SerdeValue, SerdeValueError, TcpServiceProvider,
    tcp::{AsyncStream, TcpService, TlsAcceptor, tls::OwnedClientHello},
};
use tokio::{
    io,
    net::{TcpStream, ToSocketAddrs},
};

use crate::outbound::{Outbound, OutboundName};

#[derive(Debug, Clone)]
pub struct Tcp {
    pub tls_strategy: TlsStrategy,
    pub outbounds: HashMap<OutboundName, Outbound>,
    pub balancer_strategy: Arc<dyn balancer::BalancerStrategy>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
pub struct TcpConfig {
    pub tls_strategy: TlsStrategy,
    pub outbounds: HashMap<OutboundName, Outbound>,
    pub balancer_strategy: balancer::BalancerStrategyConfig,
}

pub enum StrategyData {
    Passthrough {
        client_hello: Option<OwnedClientHello>,
    },
    Terminate {
        acceptor: Option<TlsAcceptor>,
    },
}

pub struct TcpConnectionInfo {
    pub strategy_data: StrategyData,
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
        match self.tls_strategy.clone() {
            TlsStrategy::Passthrough => {
                let accepted = accepted.mayby_tls_passthrough().await?;
                let switchboard_service::tcp::TcpAccepted { stream, context } = accepted;
                let from = context.peer_addr;
                let strategy_data = StrategyData::Passthrough {
                    client_hello: context.tls_client_hello,
                };
                let info = TcpConnectionInfo {
                    strategy_data,
                    from,
                };
                let ct = context.ct.clone();
                let outbound = self.balancer_strategy.dispatch(&self.outbounds, &info);
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
            TlsStrategy::Terminate => {
                let accepted = accepted.maybe_tls_terminate().await?;
                let switchboard_service::tcp::TcpAccepted { stream, context } = accepted;
                let from = context.peer_addr;
                let strategy_data = StrategyData::Terminate {
                    acceptor: context.tls_acceptor,
                };
                let info = TcpConnectionInfo {
                    strategy_data,
                    from,
                };
                let ct = context.ct.clone();
                let outbound = self.balancer_strategy.dispatch(&self.outbounds, &info);
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
                    format!("unsupported TLS strategy: {:?}", self.tls_strategy),
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
    const NAME: &'static str = "pf";
    type Service = Tcp;
    type Error = SerdeValueError;

    async fn construct(&self, config: Option<SerdeValue>) -> Result<Self::Service, Self::Error> {
        let config: TcpConfig = config.unwrap_or_default().deserialize_into()?;
        Ok(Tcp {
            tls_strategy: config.tls_strategy,
            outbounds: config.outbounds,
            balancer_strategy: config.balancer_strategy.build(),
        })
    }
}
