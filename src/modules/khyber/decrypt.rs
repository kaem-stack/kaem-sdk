use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DecryptCommand {
    pub ciphertext: Vec<u8>,
    pub secret_key: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DecryptedEvent {
    pub plaintext: Vec<u8>,
}
