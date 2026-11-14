use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
#[error("{reason}")]
pub struct Error {
    pub attributes: HashMap<String, String>,
    pub reason: String,
}
