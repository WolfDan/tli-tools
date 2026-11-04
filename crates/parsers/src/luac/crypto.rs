//! Stream-cipher deobfuscation (`0x20210507`) for `.out` / `\x01XDI` Lua bytecode.

use crate::luac::types::CipherState;

const CIPHER_MULT: u64 = 0x2021_0507;

#[inline(always)]
const fn mask_mod0(i: u64, v: u64) -> u8 {
    ((i.wrapping_add(v & 1).wrapping_add(v >> 28)) & 0xFF) as u8
}

#[inline(always)]
const fn mask_mod1(i: u64, v: u64) -> u8 {
    (((v >> 16) & 0xFF).wrapping_sub(i) & 0xFF) as u8
}

#[inline(always)]
const fn mask_mod2(i: u64, v: u64) -> u8 {
    ((i | (v >> 21)) & 0xFF) as u8
}

#[inline(always)]
fn xor_triplet(triplet: &mut [u8; 3], i2: u64, v2: u64) {
    let i0 = i2.wrapping_add(1);
    let v0 = v2.wrapping_add(CIPHER_MULT);
    let i1 = i2.wrapping_add(2);
    let v1 = v0.wrapping_add(CIPHER_MULT);

    triplet[0] ^= mask_mod2(i2, v2);
    triplet[1] ^= mask_mod0(i0, v0);
    triplet[2] ^= mask_mod1(i1, v1);
}

/// Decrypts a `.out` / `.luac` buffer in place if it does not already start with `\x01XDI` or `\x1bLua`.
///
/// Total over all slice lengths (`0`, `1`, `2`, and arbitrary `N`).
pub(crate) fn decrypt_luac_in_place(buf: &mut [u8]) -> CipherState {
    if buf.starts_with(b"\x01XDI") {
        return CipherState::PlainXdi;
    }
    if buf.starts_with(b"\x1bLua") {
        return CipherState::PlainLua53;
    }
    let Some(tail) = buf.get_mut(2..) else {
        return CipherState::DecryptedFromCipher;
    };

    let (blocks, remainder) = tail.as_chunks_mut::<24>();
    let mut i = 2_u64;
    let mut v = CIPHER_MULT.wrapping_mul(2);
    let step_triplet_v = CIPHER_MULT.wrapping_mul(3);

    for block in blocks {
        let (triplets, empty_rem) = block.as_chunks_mut::<3>();
        assert!(empty_rem.is_empty());
        for triplet in triplets {
            xor_triplet(triplet, i, v);
            i = i.wrapping_add(3);
            v = v.wrapping_add(step_triplet_v);
        }
    }

    for byte in remainder {
        let mask = match i % 3 {
            0 => mask_mod0(i, v),
            1 => mask_mod1(i, v),
            _ => mask_mod2(i, v),
        };
        *byte ^= mask;
        i = i.wrapping_add(1);
        v = v.wrapping_add(CIPHER_MULT);
    }

    CipherState::DecryptedFromCipher
}
