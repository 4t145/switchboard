use std::{net::SocketAddr, pin::Pin, time::{Duration, Instant}};
use std::net::UdpSocket;
pub trait Rt {
    fn spawn<F: Future + Send>(&self, future: F);
    fn spawn_local<F: Future>(&self, future: F);
}

