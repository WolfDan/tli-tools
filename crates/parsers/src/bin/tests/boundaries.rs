//! Boundary transition tests (`0`, `MAX - 1`, `MAX`, `MAX + 1`) for `BinLimits` and runtime wire decoders.

use crate::bin::tests::{TestSimpleRow, build_chunk_bin, push_str};
use crate::bin::{
    AFFIX_STACK_DEPTH_MAX, BLOCK_UNCOMPRESSED_BYTES_MAX, BinError, BinLimitKind, BinLimits,
    BinScratch, BinTable, CHUNK_BLOCKS_COUNT_MAX, COLLECTION_COUNT_MAX, FConfigAffix,
    STRING_BYTES_MAX, TABLE_ROWS_COUNT_MAX, TREE_NODES_COUNT_MAX, WireCursor,
    parse_chunk_bin_table,
};

#[test]
fn bin_limits_try_new_zero_and_hard_max_transitions() {
    assert!(
        BinLimits::try_new(
            CHUNK_BLOCKS_COUNT_MAX - 1,
            BLOCK_UNCOMPRESSED_BYTES_MAX - 1,
            TABLE_ROWS_COUNT_MAX - 1,
            STRING_BYTES_MAX - 1,
            COLLECTION_COUNT_MAX - 1,
            TREE_NODES_COUNT_MAX - 1,
        )
        .is_ok()
    );
    let at_max = BinLimits::try_new(
        CHUNK_BLOCKS_COUNT_MAX,
        BLOCK_UNCOMPRESSED_BYTES_MAX,
        TABLE_ROWS_COUNT_MAX,
        STRING_BYTES_MAX,
        COLLECTION_COUNT_MAX,
        TREE_NODES_COUNT_MAX,
    )
    .expect("exact _MAX succeeds");
    assert_eq!(at_max.block_count_max(), CHUNK_BLOCKS_COUNT_MAX);
    assert_eq!(
        at_max.block_uncompressed_bytes_max(),
        BLOCK_UNCOMPRESSED_BYTES_MAX
    );
    assert_eq!(
        BinLimits::DEFAULT.block_uncompressed_bytes_max(),
        2 * 1024 * 1024
    );

    assert_eq!(
        BinLimits::try_new(0, 1, 1, 1, 1, 1).unwrap_err(),
        BinError::InvalidLimits {
            kind: BinLimitKind::ChunkBlocksCount,
            value: 0,
            max: u64::from(CHUNK_BLOCKS_COUNT_MAX),
        }
    );
    assert_eq!(
        BinLimits::try_new(CHUNK_BLOCKS_COUNT_MAX + 1, 1, 1, 1, 1, 1).unwrap_err(),
        BinError::InvalidLimits {
            kind: BinLimitKind::ChunkBlocksCount,
            value: u64::from(CHUNK_BLOCKS_COUNT_MAX) + 1,
            max: u64::from(CHUNK_BLOCKS_COUNT_MAX),
        }
    );
    assert_eq!(
        BinLimits::try_new(1, BLOCK_UNCOMPRESSED_BYTES_MAX + 1, 1, 1, 1, 1).unwrap_err(),
        BinError::InvalidLimits {
            kind: BinLimitKind::BlockUncompressedBytes,
            value: u64::from(BLOCK_UNCOMPRESSED_BYTES_MAX) + 1,
            max: u64::from(BLOCK_UNCOMPRESSED_BYTES_MAX),
        }
    );
}

#[test]
fn bin_limits_try_new_rows_collections_strings_and_tree_nodes() {
    assert_eq!(
        BinLimits::try_new(1, 1, TABLE_ROWS_COUNT_MAX + 1, 1, 1, 1).unwrap_err(),
        BinError::InvalidLimits {
            kind: BinLimitKind::RowsCount,
            value: u64::from(TABLE_ROWS_COUNT_MAX) + 1,
            max: u64::from(TABLE_ROWS_COUNT_MAX),
        }
    );
    assert_eq!(
        BinLimits::try_new(1, 1, 1, STRING_BYTES_MAX + 1, 1, 1).unwrap_err(),
        BinError::InvalidLimits {
            kind: BinLimitKind::StringBytes,
            value: u64::from(STRING_BYTES_MAX) + 1,
            max: u64::from(STRING_BYTES_MAX),
        }
    );
    assert_eq!(
        BinLimits::try_new(1, 1, 1, 1, COLLECTION_COUNT_MAX + 1, 1).unwrap_err(),
        BinError::InvalidLimits {
            kind: BinLimitKind::CollectionCount,
            value: u64::from(COLLECTION_COUNT_MAX) + 1,
            max: u64::from(COLLECTION_COUNT_MAX),
        }
    );
    assert_eq!(
        BinLimits::try_new(1, 1, 1, 1, 1, TREE_NODES_COUNT_MAX + 1).unwrap_err(),
        BinError::InvalidLimits {
            kind: BinLimitKind::TreeNodesCount,
            value: u64::from(TREE_NODES_COUNT_MAX) + 1,
            max: u64::from(TREE_NODES_COUNT_MAX),
        }
    );
}

#[test]
fn runtime_block_and_row_count_limit_transitions() {
    let lim = BinLimits::try_new(2, 1024, 2, 4, 8, 4).expect("custom limits");
    let mut scratch = BinScratch::with_limits(lim);

    let ok_blocks = build_chunk_bin(&[None, None], 4);
    let t_ok: BinTable<i32, TestSimpleRow> =
        parse_chunk_bin_table(&ok_blocks, &mut scratch).expect("2 blocks <= max 2");
    assert!(t_ok.is_empty());

    let over_blocks = build_chunk_bin(&[None, None, None], 4);
    assert_eq!(
        parse_chunk_bin_table::<i32, TestSimpleRow>(&over_blocks, &mut scratch).unwrap_err(),
        BinError::LimitExceeded {
            kind: BinLimitKind::ChunkBlocksCount,
            actual: 3,
            max: 2,
        }
    );

    let mut blk_2 = Vec::new();
    blk_2.extend_from_slice(&2u32.to_le_bytes());
    for k in 1..=2i32 {
        blk_2.extend_from_slice(&k.to_le_bytes());
        blk_2.extend_from_slice(&10i32.to_le_bytes());
        push_str(&mut blk_2, "ab");
    }
    let t_2: BinTable<i32, TestSimpleRow> =
        parse_chunk_bin_table(&build_chunk_bin(&[Some(&blk_2)], 4), &mut scratch)
            .expect("2 rows <= max 2");
    assert_eq!(t_2.len(), 2);

    let mut blk_3 = Vec::new();
    blk_3.extend_from_slice(&3u32.to_le_bytes());
    assert_eq!(
        parse_chunk_bin_table::<i32, TestSimpleRow>(
            &build_chunk_bin(&[Some(&blk_3)], 4),
            &mut scratch
        )
        .unwrap_err(),
        BinError::LimitExceeded {
            kind: BinLimitKind::RowsCount,
            actual: 3,
            max: 2,
        }
    );
}

#[test]
fn runtime_string_and_collection_limit_transitions() {
    let lim = BinLimits::try_new(2, 1024, 4, 3, 2, 2).expect("custom limits");

    let mut s_ok = Vec::new();
    push_str(&mut s_ok, "abc");
    assert_eq!(&*WireCursor::new(&s_ok, lim).read_str().unwrap(), "abc");

    let mut s_over = Vec::new();
    push_str(&mut s_over, "abcd");
    assert_eq!(
        WireCursor::new(&s_over, lim).read_str().unwrap_err(),
        BinError::LimitExceeded {
            kind: BinLimitKind::StringBytes,
            actual: 4,
            max: 3,
        }
    );

    let mut arr_ok = Vec::new();
    arr_ok.extend_from_slice(&2u32.to_le_bytes());
    arr_ok.extend_from_slice(&1i32.to_le_bytes());
    arr_ok.extend_from_slice(&2i32.to_le_bytes());
    assert_eq!(
        &*WireCursor::new(&arr_ok, lim).read_array_i32().unwrap(),
        &[1, 2]
    );

    let arr_over = 3u32.to_le_bytes();
    assert_eq!(
        WireCursor::new(&arr_over, lim)
            .read_array_i32()
            .unwrap_err(),
        BinError::LimitExceeded {
            kind: BinLimitKind::CollectionCount,
            actual: 3,
            max: 2,
        }
    );
}

#[test]
fn runtime_tree_nodes_and_stack_depth_limit_transitions() {
    let lim = BinLimits::try_new(2, 1024, 4, 3, 2, 2).expect("custom limits");

    let mut tree_3 = Vec::new();
    tree_3.extend_from_slice(&1i32.to_le_bytes());
    tree_3.push(1);
    tree_3.extend_from_slice(&1u32.to_le_bytes());
    tree_3.push(1);
    tree_3.extend_from_slice(&2u32.to_le_bytes());
    tree_3.push(3);
    tree_3.push(15);
    tree_3.push(1);
    tree_3.extend_from_slice(&2u32.to_le_bytes());
    tree_3.push(8);
    tree_3.push(8);
    assert_eq!(
        FConfigAffix::read(&mut WireCursor::new(&tree_3, lim)).unwrap_err(),
        BinError::LimitExceeded {
            kind: BinLimitKind::TreeNodesCount,
            actual: 3,
            max: 2,
        }
    );

    let mut deep = Vec::new();
    deep.extend_from_slice(&1i32.to_le_bytes());
    deep.push(1);
    deep.extend_from_slice(&1u32.to_le_bytes());
    for _ in 0..AFFIX_STACK_DEPTH_MAX {
        deep.push(1);
        deep.extend_from_slice(&1u32.to_le_bytes());
        deep.push(3);
        deep.push(15);
    }
    assert_eq!(
        FConfigAffix::read(&mut WireCursor::new(&deep, BinLimits::DEFAULT)).unwrap_err(),
        BinError::LimitExceeded {
            kind: BinLimitKind::AffixStackDepth,
            actual: u64::from(AFFIX_STACK_DEPTH_MAX) + 1,
            max: u64::from(AFFIX_STACK_DEPTH_MAX),
        }
    );
}
