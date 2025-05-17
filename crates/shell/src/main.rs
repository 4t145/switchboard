use switchboard_tcp::TcpServiceExt;
use tokio::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    let socks5 = switchboard_socks5::Socks5::no_auth();
    let service = socks5.bind("127.0.0.1:7777".parse().unwrap()).await?;
    tokio::signal::ctrl_c().await?;
    service.cancel().await?;
    Ok(())
}
