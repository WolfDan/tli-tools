//! Workspace automation and TigerStyle code generator binary (`cargo xtask` / `cargo codegen`).

pub mod bincfg;
pub mod datacfg;
pub mod error;

use crate::datacfg::emit::CodegenMode;
use crate::error::XtaskError;

const _: () = assert!(usize::BITS >= 32);
const _: () = assert!(usize::BITS <= 64);

/// Converts a `u32` into `usize` losslessly on 32-bit and 64-bit targets.
#[inline(always)]
#[must_use]
pub const fn u32_to_usize(x: u32) -> usize {
    x as usize
}

/// Converts a bounded slice length `<= u32::MAX` into `u32`.
#[inline(always)]
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "Invariant: bounded slice/table length is asserted <= u32::MAX"
)]
pub const fn bounded_len_u32(len: usize) -> u32 {
    assert!(len <= u32_to_usize(u32::MAX));
    len as u32
}

/// Parses `xtask` CLI arguments into a [`CodegenMode`], rejecting unknown subcommands or flags.
pub fn parse_cli_args<I, S>(args: I) -> Result<CodegenMode, XtaskError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut iter = args.into_iter();
    let Some(first) = iter.next() else {
        return Err(XtaskError::Usage {
            message: "missing subcommand (expected `cargo codegen [--check]`)",
        });
    };
    let cmd = first.as_ref();
    if cmd != "codegen" {
        return Err(XtaskError::UnknownCommand {
            command: cmd.into(),
        });
    }
    let mut mode = CodegenMode::Write;
    for arg in iter {
        let flag = arg.as_ref();
        if flag == "--check" {
            mode = CodegenMode::Check;
        } else {
            return Err(XtaskError::UnknownFlag { flag: flag.into() });
        }
    }
    Ok(mode)
}

fn main() -> Result<(), XtaskError> {
    let mode = parse_cli_args(std::env::args().skip(1))?;
    datacfg::run_codegen(mode)?;
    bincfg::run_codegen(mode)
}
