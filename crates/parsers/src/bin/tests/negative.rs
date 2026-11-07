//! Negative tests asserting specific [`BinError`] variants on malformed `.bin` inputs.

use crate::bin::tables::{ActionData, verify_all_chunk_bin_by_stem, verify_all_map_bin_by_stem};
use crate::bin::tests::{TestSimpleRow, build_chunk_bin, push_str, wrap_zstd_raw_frame};
use crate::bin::{
    BinError, BinLimits, BinScratch, FConfigAffix, WireContext, WireCursor,
    parse_affix_effect_table, parse_chunk_bin_table, parse_map_bin_table,
    parse_multi_language_table,
};

#[test]
fn chunk_header_and_block_descriptor_rejections() {
    let mut scratch = BinScratch::with_limits(BinLimits::DEFAULT);

    let err_eof =
        parse_chunk_bin_table::<i32, TestSimpleRow>(&[1, 2, 3], &mut scratch).unwrap_err();
    assert_eq!(
        err_eof,
        BinError::CorruptWireLayout {
            context: WireContext::ChunkHeader,
        }
    );

    // 1 block declared, but only 4 bytes follow (needs 8 for RawBlockDesc)
    let mut short_desc = Vec::new();
    short_desc.extend_from_slice(&1u32.to_le_bytes());
    short_desc.extend_from_slice(&4u32.to_le_bytes());
    short_desc.extend_from_slice(&0u32.to_le_bytes());
    let err_desc =
        parse_chunk_bin_table::<i32, TestSimpleRow>(&short_desc, &mut scratch).unwrap_err();
    assert_eq!(
        err_desc,
        BinError::CorruptWireLayout {
            context: WireContext::BlockDescriptorTable,
        }
    );

    // Block offset out of bounds of payload
    let mut bad_range = Vec::new();
    bad_range.extend_from_slice(&1u32.to_le_bytes());
    bad_range.extend_from_slice(&4u32.to_le_bytes());
    bad_range.extend_from_slice(&100u32.to_le_bytes()); // compressed_offset = 100
    bad_range.extend_from_slice(&10u32.to_le_bytes());
    let err_range =
        parse_chunk_bin_table::<i32, TestSimpleRow>(&bad_range, &mut scratch).unwrap_err();
    assert_eq!(
        err_range,
        BinError::InvalidBlockRange {
            block_index: 0,
            start_offset: 100,
            end_offset: 0,
            payload_bytes: 0,
        }
    );
}

#[test]
fn zstd_and_zero_sized_block_rejections() {
    let mut scratch = BinScratch::with_limits(BinLimits::DEFAULT);

    // uncomp_size == 0 but non-empty frame
    let mut nonzero_frame = Vec::new();
    nonzero_frame.extend_from_slice(&1u32.to_le_bytes());
    nonzero_frame.extend_from_slice(&4u32.to_le_bytes());
    nonzero_frame.extend_from_slice(&0u32.to_le_bytes()); // comp_off = 0
    nonzero_frame.extend_from_slice(&0u32.to_le_bytes()); // uncomp_size = 0
    nonzero_frame.extend_from_slice(&[0xAA, 0xBB]);
    let err_zero =
        parse_chunk_bin_table::<i32, TestSimpleRow>(&nonzero_frame, &mut scratch).unwrap_err();
    assert_eq!(
        err_zero,
        BinError::NonEmptyFrameForEmptyBlock {
            block_index: 0,
            compressed_bytes: 2,
        }
    );

    // Corrupt Zstd frame bytes
    let mut corrupt_zstd = Vec::new();
    corrupt_zstd.extend_from_slice(&1u32.to_le_bytes());
    corrupt_zstd.extend_from_slice(&4u32.to_le_bytes());
    corrupt_zstd.extend_from_slice(&0u32.to_le_bytes());
    corrupt_zstd.extend_from_slice(&16u32.to_le_bytes());
    corrupt_zstd.extend_from_slice(&[0xFF, 0x00, 0x11, 0x22]);
    let err_zstd =
        parse_chunk_bin_table::<i32, TestSimpleRow>(&corrupt_zstd, &mut scratch).unwrap_err();
    assert!(matches!(
        err_zstd,
        BinError::ZstdDecompressFailed { block_index: 0, .. }
    ));

    // Decompressed size mismatch (header declares 8, Zstd frame decompresses to 4)
    let frame_4b = wrap_zstd_raw_frame(&0u32.to_le_bytes());
    let mut mismatch = Vec::new();
    mismatch.extend_from_slice(&1u32.to_le_bytes());
    mismatch.extend_from_slice(&4u32.to_le_bytes());
    mismatch.extend_from_slice(&0u32.to_le_bytes());
    mismatch.extend_from_slice(&8u32.to_le_bytes());
    mismatch.extend_from_slice(&frame_4b);
    let err_mismatch =
        parse_chunk_bin_table::<i32, TestSimpleRow>(&mismatch, &mut scratch).unwrap_err();
    assert_eq!(
        err_mismatch,
        BinError::DecompressedSizeMismatch {
            block_index: 0,
            expected_bytes: 8,
            actual_bytes: 4,
        }
    );
}

#[test]
fn trailing_bytes_and_duplicate_key_rejections() {
    let mut scratch = BinScratch::with_limits(BinLimits::DEFAULT);

    // Decompressed block with 0 rows + 3 trailing bytes
    let mut blk_trailing = Vec::new();
    blk_trailing.extend_from_slice(&0u32.to_le_bytes());
    blk_trailing.extend_from_slice(&[1, 2, 3]);
    let err_blk_trail = parse_chunk_bin_table::<i32, TestSimpleRow>(
        &build_chunk_bin(&[Some(&blk_trailing)], 4),
        &mut scratch,
    )
    .unwrap_err();
    assert_eq!(
        err_blk_trail,
        BinError::TrailingBytes {
            context: WireContext::DecompressedBlock,
            remaining_bytes: 3,
        }
    );

    // Map .bin file with 0 rows + 2 trailing bytes
    let mut map_trailing = Vec::new();
    map_trailing.extend_from_slice(&4u32.to_le_bytes());
    map_trailing.extend_from_slice(&0u32.to_le_bytes());
    map_trailing.extend_from_slice(&[9, 9]);
    let err_map_trail: BinError =
        parse_map_bin_table::<i32, TestSimpleRow>(&map_trailing, BinLimits::DEFAULT).unwrap_err();
    assert_eq!(
        err_map_trail,
        BinError::TrailingBytes {
            context: WireContext::MapContainerEof,
            remaining_bytes: 2,
        }
    );

    // Duplicate key in ChunkA0 table
    let mut blk_dup = Vec::new();
    blk_dup.extend_from_slice(&2u32.to_le_bytes());
    for _ in 0..2 {
        blk_dup.extend_from_slice(&7i32.to_le_bytes());
        blk_dup.extend_from_slice(&1i32.to_le_bytes());
        push_str(&mut blk_dup, "dup");
    }
    let err_dup: BinError = parse_chunk_bin_table::<i32, TestSimpleRow>(
        &build_chunk_bin(&[Some(&blk_dup)], 4),
        &mut scratch,
    )
    .unwrap_err();
    assert_eq!(
        err_dup,
        BinError::DuplicateKey {
            context: WireContext::KeyHierarchy,
        }
    );
}

#[test]
fn wire_tags_and_utf8_rejections() {
    let mut bad_utf8 = Vec::new();
    bad_utf8.extend_from_slice(&2u32.to_le_bytes());
    bad_utf8.extend_from_slice(&[0xFF, 0xFF]);
    let mut cur_utf8 = WireCursor::new(&bad_utf8, BinLimits::DEFAULT);
    assert!(matches!(
        cur_utf8.read_str().unwrap_err(),
        BinError::InvalidUtf8 { .. }
    ));

    let mut cur_vec = WireCursor::new(&[1u8], BinLimits::DEFAULT);
    assert_eq!(
        cur_vec.read_vec_dim().unwrap_err(),
        BinError::InvalidVecDim { raw_dim: 1 }
    );

    let mut bad_node = Vec::new();
    bad_node.extend_from_slice(&1i32.to_le_bytes());
    bad_node.push(1);
    bad_node.extend_from_slice(&1u32.to_le_bytes());
    bad_node.push(1);
    bad_node.extend_from_slice(&1u32.to_le_bytes());
    bad_node.push(9);
    let mut cur_node = WireCursor::new(&bad_node, BinLimits::DEFAULT);
    assert_eq!(
        FConfigAffix::read(&mut cur_node).unwrap_err(),
        BinError::InvalidTreeNodeTag { raw_tag: 9 }
    );

    let mut bad_op3 = Vec::new();
    bad_op3.extend_from_slice(&1i32.to_le_bytes());
    bad_op3.push(1);
    bad_op3.extend_from_slice(&1u32.to_le_bytes());
    bad_op3.push(1);
    bad_op3.extend_from_slice(&1u32.to_le_bytes());
    bad_op3.push(3);
    bad_op3.push(250);
    let mut cur_op3 = WireCursor::new(&bad_op3, BinLimits::DEFAULT);
    assert_eq!(
        FConfigAffix::read(&mut cur_op3).unwrap_err(),
        BinError::InvalidAffixOp3 { raw_op: 250 }
    );
}

#[test]
fn action_data_and_unknown_stem_rejections() {
    let raw_act = 999i32.to_le_bytes();
    let mut cur_act = WireCursor::new(&raw_act, BinLimits::DEFAULT);
    assert_eq!(
        ActionData::read(&mut cur_act).unwrap_err(),
        BinError::InvalidActionDataTag { raw_tag: 999 }
    );

    let mut scratch = BinScratch::with_limits(BinLimits::DEFAULT);
    assert_eq!(
        verify_all_chunk_bin_by_stem("config_does_not_exist", &[], &mut scratch).unwrap_err(),
        BinError::UnknownTableStem {
            stem: "config_does_not_exist".into()
        }
    );
    assert_eq!(
        verify_all_map_bin_by_stem("config_does_not_exist", &[], BinLimits::DEFAULT).unwrap_err(),
        BinError::UnknownTableStem {
            stem: "config_does_not_exist".into()
        }
    );

    assert_eq!(
        parse_multi_language_table(&[], BinLimits::DEFAULT).unwrap_err(),
        BinError::CorruptWireLayout {
            context: WireContext::MultiLanguageRecord,
        }
    );
    assert_eq!(
        parse_affix_effect_table(&[], BinLimits::DEFAULT).unwrap_err(),
        BinError::CorruptWireLayout {
            context: WireContext::AffixEffectRecord,
        }
    );
    assert_eq!(
        parse_map_bin_table::<i32, TestSimpleRow>(&[], BinLimits::DEFAULT).unwrap_err(),
        BinError::CorruptWireLayout {
            context: WireContext::MapContainerHeader,
        }
    );
}
