//! Structured error type for `tartarus-extract`.

use thiserror::Error;

/// Errors returned by PAK archive reading, AES decryption, texture decoding, and Lua parsing.
#[derive(Debug, Error)]
pub enum ExtractError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid AES-256 hex key: {0}")]
    InvalidKey(String),

    #[error("PAK format error: {0}")]
    PakFormat(String),

    #[error("virtual path not found in Pak VFS: {0}")]
    NotFound(String),

    #[error("decompression error: {0}")]
    Decompress(String),

    #[error("UAsset format error: {0}")]
    UassetFormat(String),

    #[error("texture decode error: {0}")]
    TextureDecode(String),

    #[error("PNG codec error: {0}")]
    PngCodec(String),

    #[error("Lua bytecode format error: {0}")]
    LuacFormat(String),
}

const _: () = {
    assert!(std::mem::size_of::<ExtractError>() > 0);
};

impl From<ExtractError> for String {
    fn from(err: ExtractError) -> Self {
        let msg = err.to_string();
        debug_assert!(!msg.is_empty());
        debug_assert!(msg.len() <= 65536);
        msg
    }
}
