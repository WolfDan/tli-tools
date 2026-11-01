//! Hardware-accelerated AES-256-ECB decryption and SHA-1 integrity verification.

use aes::Aes256;
#[cfg(test)]
use aes::cipher::BlockCipherEncrypt;
use aes::cipher::consts::U16;
use aes::cipher::{Array, BlockCipherDecrypt, KeyInit};
use sha1::{Digest, Sha1};

use crate::error::{EncryptedSectionKind, PakError};
use crate::types::{AesKey, Sha1Hash};

/// Pre-expanded AES-256 round keys for stateless parallel ECB block decryption.
pub(crate) struct PakCipher {
    cipher: Aes256,
}

impl core::fmt::Debug for PakCipher {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("PakCipher([REDACTED])")
    }
}

impl PakCipher {
    #[inline]
    #[must_use]
    pub(crate) fn new(key: &AesKey) -> Self {
        let key_array = Array::from(*key.as_bytes());
        Self {
            cipher: Aes256::new(&key_array),
        }
    }

    /// Decrypts a 16-byte-aligned buffer in-place using batched hardware AES-256-ECB.
    ///
    /// Total over empty slices (`&mut []`).
    pub(crate) fn decrypt_in_place(
        &self,
        section: EncryptedSectionKind,
        buf: &mut [u8],
    ) -> Result<(), PakError> {
        if !buf.len().is_multiple_of(16) {
            let byte_len =
                u64::try_from(buf.len()).map_err(|_| PakError::UnalignedEncryptedSize {
                    context: section,
                    byte_len: u64::MAX,
                })?;
            return Err(PakError::UnalignedEncryptedSize {
                context: section,
                byte_len,
            });
        }
        if buf.is_empty() {
            return Ok(());
        }
        let (blocks, tail) = Array::<u8, U16>::slice_as_chunks_mut(buf);
        assert!(tail.is_empty());
        self.cipher.decrypt_blocks(blocks);
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn encrypt_in_place_for_test(&self, buf: &mut [u8]) {
        assert_eq!(buf.len() % 16, 0);
        if buf.is_empty() {
            return;
        }
        let (blocks, tail) = Array::<u8, U16>::slice_as_chunks_mut(buf);
        assert!(tail.is_empty());
        self.cipher.encrypt_blocks(blocks);
    }
}

/// Computes the 20-byte SHA-1 digest of `bytes`.
#[inline]
#[must_use]
pub(crate) fn compute_sha1(bytes: &[u8]) -> Sha1Hash {
    let digest = Sha1::digest(bytes);
    Sha1Hash::from_bytes(digest.into())
}
