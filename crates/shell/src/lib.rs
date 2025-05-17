use std::{
    collections::{BTreeMap, BTreeSet},
    convert::Infallible,
    fmt::Display,
    net::SocketAddr,
    str::FromStr,
};



// pub struct SwitchboardItem {
//     pub bind: SocketAddr,
//     pub service: ServiceDescriptor,
//     pub description: Option<String>,
//     pub tags: BTreeSet<String>,
// }

// pub struct SwitchboardConfig {
//     pub items: Vec<SwitchboardItem>,
// }