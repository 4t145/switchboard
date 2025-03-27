use std::fmt::Result;

pub use tokio::net::TcpListener;
use tokio::net::TcpStream;

pub trait TcpService {
    type Fut: Future<Output = std::io::Result<()>>;
    fn serve(self, addr: TcpStream) -> Self::Fut;
}
