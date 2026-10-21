//! Fast AES-256-ECB decryption (hardware AES-NI / ARMv8 Crypto via the `aes` crate).

use aes::Aes256Dec;
use aes::cipher::{Array, BlockCipherDecrypt, KeyInit};

use crate::error::ExtractError;

/// Default AES-256 key for Torchlight: Infinite UE 4.27 Pak v12 archives.
pub const DEFAULT_AES_KEY_HEX: &str =
    "0x4F17B93C3F6BC5566B505A9445EE2F620C6FBB4EF99AFD7AD6F50F361C1B34E0";

pub const DEFAULT_AES_KEY: [u8; 32] = [
    0x4F, 0x17, 0xB9, 0x3C, 0x3F, 0x6B, 0xC5, 0x56, 0x6B, 0x50, 0x5A, 0x94, 0x45, 0xEE, 0x2F, 0x62,
    0x0C, 0x6F, 0xBB, 0x4E, 0xF9, 0x9A, 0xFD, 0x7A, 0xD6, 0xF5, 0x0F, 0x36, 0x1C, 0x1B, 0x34, 0xE0,
];

const _: () = {
    assert!(DEFAULT_AES_KEY.len() == 32);
    assert!(DEFAULT_AES_KEY_HEX.len() == 66);
};

pub fn parse_hex_key(s: &str) -> Result<[u8; 32], ExtractError> {
    debug_assert!(s.len() <= 4096);
    let clean = s
        .trim()
        .strip_prefix("0x")
        .or_else(|| s.trim().strip_prefix("0X"))
        .unwrap_or(s.trim());
    let mut out = [0u8; 32];
    hex::decode_to_slice(clean, &mut out)
        .map_err(|e| ExtractError::InvalidKey(format!("({} chars): {e}", clean.len())))?;
    assert_eq!(out.len(), 32);
    Ok(out)
}

pub fn parse_aes_key_hex(s: &str) -> Result<[u8; 32], ExtractError> {
    debug_assert!(s.len() <= 4096);
    let key = parse_hex_key(s)?;
    debug_assert_eq!(key.len(), 32);
    Ok(key)
}

/// Decrypts `buf` in-place using AES-256-ECB.
/// `buf.len()` is rounded down to a multiple of 16 bytes.
#[inline]
pub fn decrypt_aes256_ecb_in_place(buf: &mut [u8], key: &[u8; 32]) {
    let len_initial = buf.len();
    assert_eq!(key.len(), 32);
    let cipher = Aes256Dec::new(&Array::from(*key));
    for chunk in buf.as_chunks_mut::<16>().0 {
        let mut block = Array::from(*chunk);
        cipher.decrypt_block(&mut block);
        chunk.copy_from_slice(block.as_slice());
    }
    assert_eq!(buf.len(), len_initial);
}
