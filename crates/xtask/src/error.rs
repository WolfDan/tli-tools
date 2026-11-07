//! Concrete error hierarchy for `xtask` workspace tooling and code generation.

use thiserror::Error;

/// Errors returned by `xtask` CLI commands and the `DataCfg` schema code generator.
#[derive(Debug, Error)]
pub enum XtaskError {
    /// Missing or invalid CLI subcommand.
    #[error("invalid CLI arguments: {message}")]
    Usage { message: &'static str },

    /// Unrecognized top-level subcommand.
    #[error("unknown xtask command `{command}` (expected `codegen`)")]
    UnknownCommand { command: Box<str> },

    /// Unrecognized CLI flag.
    #[error("unknown CLI flag `{flag}` (supported flags: `--check`)")]
    UnknownFlag { flag: Box<str> },

    /// Required environment variable is missing or non-Unicode.
    #[error("failed to read environment variable `{name}`")]
    EnvVar {
        name: &'static str,
        #[source]
        source: std::env::VarError,
    },

    /// Failed to open or read a `.pak` archive.
    #[error("pak archive error")]
    Pak(#[from] pak::PakError),

    /// Failed to mount or query `PakVfs`.
    #[error("pak virtual filesystem error")]
    Vfs(#[from] pak_vfs::VfsError),

    /// Filesystem I/O failure while reading or writing generated Rust modules.
    #[error("filesystem I/O error")]
    Io(#[from] std::io::Error),

    /// Invalid UTF-8 encountered in a virtual path or bytecode string constant.
    #[error("invalid UTF-8 in path or bytecode constant")]
    Utf8(#[from] core::str::Utf8Error),

    /// Malformed or truncated `\x01XDI` bytecode chunk during schema extraction.
    #[error("malformed bytecode in `{stem}`: {context}")]
    MalformedBytecode {
        stem: Box<str>,
        context: &'static str,
    },

    /// Root instruction evaluation exceeded `GEN_INSTRUCTIONS_MAX` without returning.
    #[error("instruction step limit ({max}) exceeded in `{stem}` without Return")]
    StepLimitExceeded { stem: Box<str>, max: u32 },

    /// Two rows or table entries had incompatible column types during schema inference.
    #[error("incompatible inferred types: {left} vs {right}")]
    IncompatibleTypes { left: Box<str>, right: Box<str> },

    /// `--check` mode detected that a committed generated file is missing or out of date.
    #[error("generated file `{path}` is out of date; run `cargo codegen` to update")]
    GeneratedFileDrift { path: Box<str> },

    /// Failed to parse PE64 headers or extract `.bin` schema metadata from the client executable.
    #[error("PE64 schema extraction failed in `{context}`: {detail}")]
    PeExtract {
        context: &'static str,
        detail: Box<str>,
    },

    /// `.bin` corpus verification failed against an extracted schema.
    #[error(".bin corpus verification failed in `{path}`: {detail}")]
    BinCorpusVerify { path: Box<str>, detail: Box<str> },

    /// Generated token stream failed to parse as a valid Rust syntax tree.
    #[error("generated Rust AST parse error")]
    Syn(#[from] syn::Error),
}
