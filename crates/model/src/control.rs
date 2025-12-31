
use sha2::Digest;

use crate::ServiceConfig;


impl ServiceConfig {
    pub fn digest_sha256_base64(&self) -> String {
        use base64::prelude::*;
        let digest = self.digest_sha256();
        BASE64_STANDARD.encode(digest)
    }
    pub fn digest_sha256(&self) -> Vec<u8> {
        let config_as_bytes = bincode::encode_to_vec(self, bincode::config::standard())
            .expect("Config should be always serializable");
        let mut hasher = sha2::Sha256::default();
        hasher.update(&config_as_bytes);
        hasher.finalize().to_vec()
    }
}
