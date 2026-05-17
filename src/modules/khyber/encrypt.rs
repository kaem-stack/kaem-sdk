use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptCommand {
    pub message: Vec<u8>,
    pub public_key: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedEvent {
    pub ciphertext: Vec<u8>,
}
