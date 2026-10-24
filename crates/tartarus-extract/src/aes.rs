//! Hardware-accelerated AES-256-ECB decryption using 8-block pipelined AES-NI / ARMv8 Crypto.
//!
//! Why: AES-256 decryption has a 14-round dependency chain per 16-byte block (~56 cycles latency
//! on x86_64 AES-NI), whereas the CPU can issue 1 `aesdec` instruction per cycle across independent
//! blocks. Decrypting 8 blocks (128 bytes) in parallel via `BlockCipherDecrypt::decrypt_blocks`
//! saturates hardware vector pipelines and avoids re-expanding round keys on the data plane.

use aes::Aes256Dec;
use aes::cipher::{Array, BlockCipherDecrypt, KeyInit};

use crate::error::ExtractError;

/// Default AES-256 key in hexadecimal for *Torchlight: Infinite* UE 4.27 Pak v12 archives.
pub const DEFAULT_AES_KEY_HEX: &str =
    "0x4F17B93C3F6BC5566B505A9445EE2F620C6FBB4EF99AFD7AD6F50F361C1B34E0";

/// Default 32-byte AES-256 key for *Torchlight: Infinite* UE 4.27 Pak v12 archives.
pub const DEFAULT_AES_KEY: [u8; 32] = [
    0x4F, 0x17, 0xB9, 0x3C, 0x3F, 0x6B, 0xC5, 0x56, 0x6B, 0x50, 0x5A, 0x94, 0x45, 0xEE, 0x2F, 0x62,
    0x0C, 0x6F, 0xBB, 0x4E, 0xF9, 0x9A, 0xFD, 0x7A, 0xD6, 0xF5, 0x0F, 0x36, 0x1C, 0x1B, 0x34, 0xE0,
];

/// Maximum number of 128-byte (8-block) AES batches allowed in a single buffer (4 GiB).
pub const AES_BATCH_COUNT_MAX: usize = 33_554_432;

const AES_BLOCK_SIZE: usize = 16;
const AES_PAR_BLOCKS: usize = 8;
const AES_BATCH_BYTES: usize = AES_BLOCK_SIZE * AES_PAR_BLOCKS;

const _: () = {
    assert!(DEFAULT_AES_KEY.len() == 32);
    assert!(DEFAULT_AES_KEY_HEX.len() == 66);
    assert!(AES_BLOCK_SIZE == 16);
    assert!(AES_BATCH_BYTES == 128);
};

/// Pre-expanded AES-256 decryption round keys for zero-overhead data-plane decryption.
#[derive(Clone, Debug)]
pub struct Aes256KeySchedule {
    cipher: Aes256Dec,
    key_raw: [u8; 32],
}

impl Aes256KeySchedule {
    /// Expands a 256-bit key into 15 decryption round keys once during control-plane setup.
    #[inline]
    pub fn new(key_raw: [u8; 32]) -> Self {
        assert_eq!(key_raw.len(), 32);
        let cipher = Aes256Dec::new(&Array::from(key_raw));
        let schedule = Self { cipher, key_raw };
        assert_eq!(schedule.key_raw.len(), 32);
        schedule
    }

    /// Returns the raw 32-byte key used to construct this schedule.
    #[inline]
    pub const fn key_bytes(&self) -> &[u8; 32] {
        assert!(self.key_raw.len() == 32);
        &self.key_raw
    }

    /// Decrypts `target` in-place using 8-block (128-byte) parallel hardware AES-256-ECB batches.
    ///
    /// Trailing bytes that do not form a complete 16-byte AES block are left untouched.
    #[inline]
    pub fn decrypt_in_place(&self, target: &mut [u8]) {
        let length_initial = target.len();
        assert!(length_initial <= AES_BATCH_COUNT_MAX * AES_BATCH_BYTES);

        let (batches_128, remainder_128) = target.as_chunks_mut::<AES_BATCH_BYTES>();
        let mut iterations = 0usize;
        for batch in batches_128 {
            iterations += 1;
            assert!(iterations <= AES_BATCH_COUNT_MAX);

            let mut blocks = [Array::default(); AES_PAR_BLOCKS];
            let (chunks_16, _) = batch.as_chunks::<AES_BLOCK_SIZE>();
            for (block_dst, block_src) in blocks.iter_mut().zip(chunks_16.iter()) {
                *block_dst = Array::from(*block_src);
            }
            self.cipher.decrypt_blocks(&mut blocks);
            let (out_16, _) = batch.as_chunks_mut::<AES_BLOCK_SIZE>();
            for (block_dst, block_src) in out_16.iter_mut().zip(blocks.iter()) {
                block_dst.copy_from_slice(block_src.as_slice());
            }
        }

        let (tail_blocks, _) = remainder_128.as_chunks_mut::<AES_BLOCK_SIZE>();
        assert!(tail_blocks.len() < AES_PAR_BLOCKS);
        for chunk in tail_blocks {
            let mut block = Array::from(*chunk);
            self.cipher.decrypt_block(&mut block);
            chunk.copy_from_slice(block.as_slice());
        }

        assert_eq!(target.len(), length_initial);
    }
}

/// Parses a 64-character hex string (with optional `0x`/`0X` prefix) into a 32-byte AES-256 key.
pub fn parse_hex_key(source: &str) -> Result<[u8; 32], ExtractError> {
    assert!(source.len() <= 4096);
    let trimmed = source.trim();
    let clean = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    let mut target = [0u8; 32];
    hex::decode_to_slice(clean, &mut target)
        .map_err(|e| ExtractError::InvalidKey(format!("({} chars): {e}", clean.len())))?;
    assert_eq!(target.len(), 32);
    Ok(target)
}

/// Alias for [`parse_hex_key`] preserving CLI/library API compatibility.
pub fn parse_aes_key_hex(source: &str) -> Result<[u8; 32], ExtractError> {
    assert!(source.len() <= 4096);
    let key = parse_hex_key(source)?;
    assert_eq!(key.len(), 32);
    Ok(key)
}

/// Decrypts `target` in-place using AES-256-ECB with 8-block hardware pipelining.
///
/// `target.len()` is rounded down to a multiple of 16 bytes.
#[inline]
pub fn decrypt_aes256_ecb_in_place(target: &mut [u8], key: &[u8; 32]) {
    let length_initial = target.len();
    assert_eq!(key.len(), 32);
    let schedule = Aes256KeySchedule::new(*key);
    schedule.decrypt_in_place(target);
    assert_eq!(target.len(), length_initial);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes256_schedule_batch_matches_single_block() {
        let schedule = Aes256KeySchedule::new(DEFAULT_AES_KEY);
        let cipher = Aes256Dec::new(&Array::from(DEFAULT_AES_KEY));

        // 19 blocks (304 bytes) exercises two 8-block batches (256 bytes) + 3 tail blocks (48 bytes).
        let mut buffer_batch = vec![0u8; 19 * 16 + 7];
        for (idx, byte) in buffer_batch.iter_mut().enumerate() {
            *byte = (idx as u8).wrapping_mul(37).wrapping_add(11);
        }
        let mut buffer_single = buffer_batch.clone();

        schedule.decrypt_in_place(&mut buffer_batch);
        for chunk in buffer_single.as_chunks_mut::<16>().0 {
            let mut block = Array::from(*chunk);
            cipher.decrypt_block(&mut block);
            chunk.copy_from_slice(block.as_slice());
        }

        assert_eq!(buffer_batch, buffer_single);
        assert_eq!(&buffer_batch[19 * 16..], &buffer_single[19 * 16..]);
    }

    #[test]
    fn test_parse_aes_key_hex_roundtrip() {
        let parsed = parse_aes_key_hex(DEFAULT_AES_KEY_HEX).expect("valid default hex key");
        assert_eq!(parsed, DEFAULT_AES_KEY);
        assert!(parse_aes_key_hex("0x1234").is_err());
    }
}
