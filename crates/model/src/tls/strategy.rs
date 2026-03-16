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
    #[default]
    Passthrough,
    Terminate,
    // ReEncrypt,
}
