#[derive(
    Debug,
    Clone,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
    PartialEq,
    Default,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub struct SniName(String);
