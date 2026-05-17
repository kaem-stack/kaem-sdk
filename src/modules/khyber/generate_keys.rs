use serde::{Deserialize, Serialize};

use super::Algorithm;

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateKeysCommand {
    pub algorithm: Algorithm,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeysGeneratedEvent {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
}
