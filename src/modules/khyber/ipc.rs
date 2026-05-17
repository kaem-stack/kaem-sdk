use serde::{Deserialize, Serialize};

use super::{
    decrypt::{DecryptCommand, DecryptedEvent},
    encrypt::{EncryptCommand, EncryptedEvent},
    generate_keys::{GenerateKeysCommand, KeysGeneratedEvent},
};

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    GenerateKeys(GenerateKeysCommand),
    Encrypt(EncryptCommand),
    Decrypt(DecryptCommand),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    KeysGenerated(KeysGeneratedEvent),
    Encrypted(EncryptedEvent),
    Decrypted(DecryptedEvent),
    Error(String),
}
