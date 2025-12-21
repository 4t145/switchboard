use std::{hash::Hash, sync::atomic::AtomicU32};

use hmac::{Mac, digest::MacError};
use serde::{Deserialize, Serialize};

use crate::{Config, controller::ControllerInfo, kernel::KernelState};
#[derive(Debug, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "kind", content = "data")]
pub enum ControllerMessage {
    HeartBeat,
    TakeOver(TakeOver),
    AuthResponse(KernelAuthResponse),
    ControlCommand(ControlCommand),
    Disconnect,
    // todo: controller can notice kernel when itself is going to shutdown
}

#[derive(Debug, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "kind", content = "data")]
pub enum ControlCommandData {
    Quit,
    UpdateConfig(UpdateConfig),
}

#[derive(Debug, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "kind", content = "data")]
pub enum KernelMessage {
    HeartBeat(KernelState),
    Auth(KernelAuth),
    ControlCommandAccepted(ControlCommandAccepted),
    BeenTookOver(BeenTookOver),
    Disconnect,
}

impl KernelMessage {
    pub fn is_been_took_over(&self) -> bool {
        matches!(self, KernelMessage::BeenTookOver(_))
    }
}

#[derive(Debug, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[serde(rename_all = "camelCase")]
pub struct TakeOver {
    pub controller_info: ControllerInfo,
}

#[derive(Debug, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[serde(rename_all = "camelCase")]
pub struct BeenTookOver {
    pub new_controller_info: ControllerInfo,
}

#[derive(Debug, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KernelAuth {
    pub random_bytes: Vec<u8>,
    pub kernel_info: crate::kernel::KernelInfo,
}

#[derive(Debug, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct KernelAuthResponse {
    pub signature: Vec<u8>,
}

impl KernelAuthResponse {
    pub fn verify(&self, auth: &KernelAuth, sign_key: &[u8]) -> Result<(), MacError> {
        let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(sign_key)
            .expect("HMAC can take key of any size");
        mac.update(&auth.random_bytes);
        mac.verify_slice(&self.signature)
    }

    pub fn sign(auth: &KernelAuth, sign_key: &[u8]) -> Self {
        let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(sign_key)
            .expect("HMAC can take key of any size");
        mac.update(&auth.random_bytes);
        let signature = mac.finalize().into_bytes().to_vec();
        KernelAuthResponse { signature }
    }
}

#[derive(Debug, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConfig {
    pub config: Config,
}

pub struct UpdateConfigFinished {}

#[derive(Debug, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConfigBuilder {
    pub config: Config,
}

#[derive(Debug, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[serde(rename_all = "camelCase")]
pub struct ControlCommand {
    pub seq: u32,
    pub ts: i64,
    pub signer_name: String,
    pub data: ControlCommandData,
    pub signature: Vec<u8>,
}
#[derive(Debug, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[serde(rename_all = "camelCase")]
pub struct ControlCommandAccepted {
    pub seq: u32,
}

pub struct ControlSigner {
    pub sign_key: Vec<u8>,
    pub next_seq: AtomicU32,
    pub signer_name: String,
}
pub struct ControlVerifier {
    pub sign_key: Vec<u8>,
}

impl ControlSigner {
    pub fn new(sign_key: Vec<u8>, signer_name: String) -> Self {
        Self {
            sign_key,
            next_seq: AtomicU32::new(1),
            signer_name,
        }
    }
    pub fn sign_command(&self, data: ControlCommandData) -> ControlCommand {
        let seq = self
            .next_seq
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let ts = chrono::Utc::now().timestamp_micros();
        let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(&self.sign_key)
            .expect("HMAC can take key of any size");
        mac.update(&seq.to_be_bytes());
        mac.update(&ts.to_be_bytes());
        mac.update(self.signer_name.as_bytes());
        bincode::encode_into_std_write(&data, &mut mac, bincode::config::standard())
            .expect("control data should be always serializable");
        let signature = mac.finalize().into_bytes().to_vec();
        ControlCommand {
            data,
            seq,
            ts,
            signer_name: self.signer_name.clone(),
            signature,
        }
    }
}

impl ControlVerifier {
    pub fn verify_command(&self, command: &ControlCommand) -> Result<(), MacError> {
        let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(&self.sign_key)
            .expect("HMAC can take key of any size");
        mac.update(&command.seq.to_be_bytes());
        mac.update(&command.ts.to_be_bytes());
        mac.update(command.signer_name.as_bytes());
        bincode::encode_into_std_write(&command.data, &mut mac, bincode::config::standard())
            .expect("control data should be always serializable");
        mac.verify_slice(&command.signature)
    }
}

impl Config {
    pub fn sign(&self, key: &[u8]) -> Vec<u8> {
        let config_as_bytes = bincode::encode_to_vec(self, bincode::config::standard())
            .expect("Config should be always serializable");
        let mut mac =
            hmac::Hmac::<sha2::Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
        mac.update(&config_as_bytes);
        mac.finalize().into_bytes().to_vec()
    }
    pub fn verify_signature(&self, signature: &[u8], key: &[u8]) -> Result<(), MacError> {
        let config_as_bytes = bincode::encode_to_vec(self, bincode::config::standard())
            .expect("Config should be always serializable");
        let mut mac =
            hmac::Hmac::<sha2::Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
        mac.update(&config_as_bytes);
        mac.verify_slice(signature)
    }
}
#[cfg(test)]
mod test {
    #[test]
    fn sign_with_empty_psk() {
        use super::*;
        let config = Config::default();
        let key = b"";
        let signature = config.sign(key);
        println!("signature: {:?}", signature);
        assert!(config.verify_signature(&signature, key).is_ok());
    }
}