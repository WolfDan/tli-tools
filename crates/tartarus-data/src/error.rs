//! Structured error type for `tartarus-data`.

use thiserror::Error;

/// Errors returned by database extraction, font parsing, catalog rendering, and CV codegen.
#[derive(Debug, Error)]
pub enum DataError {
    #[error("PAK extraction error: {0}")]
    Extract(#[from] tartarus_extract::ExtractError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("font error: {0}")]
    Font(String),

    #[error("missing required game asset: {0}")]
    MissingAsset(String),
}

const _: () = {
    assert!(std::mem::size_of::<DataError>() > 0);
};

impl From<DataError> for String {
    fn from(err: DataError) -> Self {
        let msg = err.to_string();
        debug_assert!(!msg.is_empty());
        debug_assert!(msg.len() <= 65536);
        msg
    }
}
