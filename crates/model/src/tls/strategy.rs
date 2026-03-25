#[derive(
    Debug,
    Clone,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
    PartialEq,
    Default,
)]
#[non_exhaustive]
pub enum TlsStrategy {
    Passthrough,
    #[default]
    Terminate,
    // ReEncrypt,
}
