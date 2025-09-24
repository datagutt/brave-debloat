use thiserror::Error;

#[derive(Error, Debug)]
pub enum DebloaterError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Config file not found: {0}")]
    ConfigNotFound(String),
}