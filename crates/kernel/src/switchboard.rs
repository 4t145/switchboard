use std::{collections::HashMap, sync::Arc};
use switchboard_service::tcp::{RunningTcpService, SharedTcpService, TcpListener};
use tokio::sync::watch::{Receiver, Sender, channel};
pub type ResourceKey = Arc<str>;

pub mod tcp;
