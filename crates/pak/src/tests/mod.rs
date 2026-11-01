//! Unit, boundary, and negative test suite for `pak`.

mod boundaries;
pub(crate) mod builder;
mod negative;
mod positive;

use crate::archive::PakArchive;
use crate::scratch::ReadScratch;
use crate::types::{AesKey, EntryId};
use crate::u32_to_usize;

pub(crate) fn test_aes_key() -> AesKey {
    let mut raw = [0_u8; 32];
    for (idx, slot) in raw.iter_mut().enumerate() {
        let byte = u8::try_from(idx).expect("idx < 32");
        *slot = byte.wrapping_mul(7).wrapping_add(0x3D);
    }
    AesKey::from_bytes(raw)
}

pub(crate) fn read_entry_vec(
    archive: &PakArchive,
    id: EntryId,
    scratch: &mut ReadScratch,
) -> Vec<u8> {
    let meta = archive.entry_meta(id);
    let len_u32 = u32::try_from(meta.uncompressed_bytes()).expect("test entry fits in u32");
    let mut dst = vec![0_u8; u32_to_usize(len_u32)];
    let written = archive
        .read_entry_into(id, scratch, &mut dst)
        .expect("read_entry_into must succeed");
    assert_eq!(written, meta.uncompressed_bytes());
    dst
}
