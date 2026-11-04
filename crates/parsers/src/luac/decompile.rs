//! General Lua 5.3 script decompilation via `unluac` (`v1.4.4`) in permissive mode.

use crate::luac::crypto::decrypt_luac_in_place;
use crate::luac::error::LuacError;
use crate::luac::transpile::transpile_to_lua53_into;
use crate::luac::types::LuacLimits;
use crate::{FixedVec, u32_to_usize};
use unluac::decompile::{DecompileDialect, DecompileOptions, decompile};
use unluac::generate::{GenerateMode, GenerateOptions, GeneratedChunkKind};
use unluac::parser::{ParseMode, ParseOptions, StringDecodeMode, StringEncoding};

/// Indicates whether `unluac` produced pure Lua source or diagnostic fallback pseudocode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecompiledChunkKind {
    /// Complete target-dialect Lua 5.3 source code.
    Source,
    /// Permissive recovery output containing diagnostic fallback constructs.
    DiagnosticPseudocode,
}

/// Decompiled Lua 5.3 script output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecompiledLua {
    source: Box<str>,
    kind: DecompiledChunkKind,
}

impl DecompiledLua {
    /// Returns the decompiled Lua source text.
    #[inline]
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Consumes `self` and returns the boxed Lua source string.
    #[inline]
    #[must_use]
    pub fn into_source(self) -> Box<str> {
        self.source
    }

    /// Returns whether the generated chunk is pure source or diagnostic pseudocode.
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> DecompiledChunkKind {
        self.kind
    }
}

/// Configuration for `decompile_luac`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecompileConfig {
    limits: LuacLimits,
    strict_mode: bool,
    include_comments: bool,
    indent_width: u8,
    max_line_length: u16,
}

impl DecompileConfig {
    /// Default configuration (`DecompileLimits::DEFAULT` with permissive fallback comments enabled).
    pub const DEFAULT: Self = Self {
        limits: LuacLimits::DEFAULT,
        strict_mode: false,
        include_comments: false,
        indent_width: 4,
        max_line_length: 100,
    };

    #[must_use]
    pub const fn with_limits(mut self, limits: LuacLimits) -> Self {
        self.limits = limits;
        self
    }

    #[must_use]
    pub const fn with_strict_mode(mut self, strict_mode: bool) -> Self {
        self.strict_mode = strict_mode;
        self
    }

    #[must_use]
    pub const fn with_include_comments(mut self, include_comments: bool) -> Self {
        self.include_comments = include_comments;
        self
    }

    #[must_use]
    pub const fn with_indent_width(mut self, indent_width: u8) -> Self {
        self.indent_width = indent_width;
        self
    }

    #[must_use]
    pub const fn with_max_line_length(mut self, max_line_length: u16) -> Self {
        self.max_line_length = max_line_length;
        self
    }

    #[must_use]
    pub const fn limits(&self) -> LuacLimits {
        self.limits
    }

    #[must_use]
    pub const fn strict_mode(&self) -> bool {
        self.strict_mode
    }

    #[must_use]
    pub const fn include_comments(&self) -> bool {
        self.include_comments
    }

    #[must_use]
    pub const fn indent_width(&self) -> u8 {
        self.indent_width
    }

    #[must_use]
    pub const fn max_line_length(&self) -> u16 {
        self.max_line_length
    }
}

/// Reusable pre-allocated scratch buffer for `.out` -> Lua 5.3 transpilation and decompilation.
#[derive(Debug)]
pub struct DecompileScratch {
    lua53_buf: FixedVec<u8>,
}

impl DecompileScratch {
    /// Allocates a reusable `DecompileScratch` bounded by `limits.transpile_bytes_max()`.
    #[must_use]
    pub fn with_limits(limits: LuacLimits) -> Self {
        Self {
            lua53_buf: FixedVec::with_capacity(limits.transpile_bytes_max()),
        }
    }
}

/// Decrypts, transpiles, and decompiles a `.out` / `\x01XDI` or canonical `\x1bLua` buffer into Lua 5.3 source.
pub fn decompile_luac(
    buf: &mut [u8],
    scratch: &mut DecompileScratch,
    config: &DecompileConfig,
) -> Result<DecompiledLua, LuacError> {
    let _cipher = decrypt_luac_in_place(buf);
    transpile_to_lua53_into(buf, &mut scratch.lua53_buf, config.limits())?;

    let (parse_mode, gen_mode) = if config.strict_mode {
        (ParseMode::Strict, GenerateMode::Strict)
    } else {
        (ParseMode::Permissive, GenerateMode::Permissive)
    };

    let options = DecompileOptions {
        dialect: DecompileDialect::Lua53,
        parse: ParseOptions {
            mode: parse_mode,
            string_encoding: StringEncoding::Utf8,
            string_decode_mode: StringDecodeMode::Lossy,
            ignore_debug: false,
        },
        generate: GenerateOptions {
            mode: gen_mode,
            indent_width: u32_to_usize(u32::from(config.indent_width)),
            max_line_length: u32_to_usize(u32::from(config.max_line_length)),
            comment: config.include_comments,
            ..GenerateOptions::default()
        },
        ..DecompileOptions::default()
    };

    let res =
        decompile(scratch.lua53_buf.as_slice(), options).map_err(|err| LuacError::Decompile {
            source: Box::new(err),
        })?;
    let generated = res
        .state
        .generated
        .ok_or(LuacError::MissingGeneratedChunk)?;
    let kind = match generated.kind {
        GeneratedChunkKind::Source => DecompiledChunkKind::Source,
        GeneratedChunkKind::DiagnosticPseudocode => DecompiledChunkKind::DiagnosticPseudocode,
    };
    Ok(DecompiledLua {
        source: generated.source.into_boxed_str(),
        kind,
    })
}
