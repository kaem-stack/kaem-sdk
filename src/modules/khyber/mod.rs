pub mod decrypt;
pub mod encrypt;
pub mod generate_keys;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum Algorithm {
    #[default]
    MlKem768,
}

impl Algorithm {
    pub fn name(&self) -> &'static str {
        match self {
            Algorithm::MlKem768 => "ML-KEM-768",
        }
    }
}
