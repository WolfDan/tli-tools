//! Unit, boundary, negative, and corpus integration tests for `parsers::bin`.

mod boundaries;
mod negative;
mod positive;

use crate::bin::{BinError, BinRow, WireCursor};
use crate::bounded_len_u32;

/// Wraps an uncompressed byte slice into a valid RFC 8878 Zstandard `Raw_Block` frame.
pub(crate) fn wrap_zstd_raw_frame(uncompressed: &[u8]) -> Vec<u8> {
    let len = bounded_len_u32(uncompressed.len());
    assert!(len < (1 << 17));
    let mut out = Vec::with_capacity(12 + uncompressed.len());
    // Zstd magic number: 0xFD2FB528 (LE)
    out.extend_from_slice(&[0x28, 0xB5, 0x2F, 0xFD]);
    // Frame_Header_Descriptor: FCS_Flag=2 (4-byte FCS), Single_Segment_Flag=1 -> 0xA0
    out.push(0xA0);
    // 4-byte Frame_Content_Size
    out.extend_from_slice(&len.to_le_bytes());
    // 3-byte Block_Header: Last_Block=1, Block_Type=0 (Raw_Block), Block_Size=len
    let block_hdr: u32 = 1 | (len << 3);
    out.push((block_hdr & 0xFF) as u8);
    out.push(((block_hdr >> 8) & 0xFF) as u8);
    out.push(((block_hdr >> 16) & 0xFF) as u8);
    out.extend_from_slice(uncompressed);
    out
}

/// Builds a `*_ChunkA0.bin` container from a slice of `Option<uncompressed_block>` and `hash_buckets`.
pub(crate) fn build_chunk_bin(blocks: &[Option<&[u8]>], hash_buckets: u32) -> Vec<u8> {
    let block_count = bounded_len_u32(blocks.len());
    let mut out = Vec::new();
    out.extend_from_slice(&block_count.to_le_bytes());
    out.extend_from_slice(&hash_buckets.to_le_bytes());

    let mut payload = Vec::new();
    let mut descs = Vec::with_capacity(blocks.len());
    for blk in blocks {
        let comp_off = bounded_len_u32(payload.len());
        match blk {
            Some(raw) => {
                let frame = wrap_zstd_raw_frame(raw);
                payload.extend_from_slice(&frame);
                descs.push((comp_off, bounded_len_u32(raw.len())));
            }
            None => {
                descs.push((comp_off, 0u32));
            }
        }
    }
    for (comp_off, uncomp_len) in descs {
        out.extend_from_slice(&comp_off.to_le_bytes());
        out.extend_from_slice(&uncomp_len.to_le_bytes());
    }
    out.extend_from_slice(&payload);
    out
}

pub(crate) fn push_str(buf: &mut Vec<u8>, s: &str) {
    let len = bounded_len_u32(s.len());
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(s.as_bytes());
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct TestSimpleRow {
    pub label: Box<str>,
    pub value: i32,
}

impl BinRow for TestSimpleRow {
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.value = cur.read_i32()?;
        self.label = cur.read_str()?;
        Ok(())
    }
}
