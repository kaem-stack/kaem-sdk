use serde::{Deserialize, Serialize};

use crate::modules::khyber::ipc as khyber;

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Khyber(khyber::Request),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Khyber(khyber::Response),
}
