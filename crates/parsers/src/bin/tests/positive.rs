//! Positive round-trip and end-to-end `.pak` corpus tests for `parsers::bin`.

use crate::bin::tables::{
    ActionData, ActionData1004, AffixOp3Code, FConfigI32I32, parse_config_c,
    verify_all_chunk_bin_by_stem, verify_all_map_bin_by_stem,
};
use crate::bin::tests::{TestSimpleRow, build_chunk_bin, push_str};
use crate::bin::{
    AffixEffectRow, AffixLookupEntry, AffixOperand, AffixTreeNode, BinLimits, BinScratch, BinTable,
    FConfigAffix, MultiLanguageRow, VecDim, WireCursor, normalize_chunk_bin_stem,
    parse_affix_effect_table, parse_chunk_bin_table, parse_map_bin_table,
    parse_multi_language_table,
};
use std::path::Path;

#[test]
fn chunk_bin_round_trip_int_key_depths_and_empty_block() {
    let mut blk_int1 = Vec::new();
    blk_int1.extend_from_slice(&1u32.to_le_bytes());
    blk_int1.extend_from_slice(&10i32.to_le_bytes());
    blk_int1.extend_from_slice(&42i32.to_le_bytes());
    push_str(&mut blk_int1, "alpha");

    let chunk_int1 = build_chunk_bin(&[None, Some(&blk_int1)], 4);
    let mut scratch = BinScratch::with_limits(BinLimits::DEFAULT);
    let t1: BinTable<i32, TestSimpleRow> =
        parse_chunk_bin_table(&chunk_int1, &mut scratch).expect("parse Int1");
    assert_eq!(t1.len(), 1);
    assert!(!t1.is_empty());
    assert_eq!(
        t1.get(&10),
        Some(&TestSimpleRow {
            label: "alpha".into(),
            value: 42,
        })
    );
    assert_eq!(t1.iter().count(), 1);

    let mut blk_int3 = Vec::new();
    blk_int3.extend_from_slice(&1u32.to_le_bytes()); // level 0 count
    blk_int3.extend_from_slice(&1i32.to_le_bytes()); // k0
    blk_int3.extend_from_slice(&4u32.to_le_bytes()); // buckets
    blk_int3.extend_from_slice(&1u32.to_le_bytes()); // level 1 count
    blk_int3.extend_from_slice(&2i32.to_le_bytes()); // k1
    blk_int3.extend_from_slice(&4u32.to_le_bytes()); // buckets
    blk_int3.extend_from_slice(&1u32.to_le_bytes()); // level 2 count
    blk_int3.extend_from_slice(&3i32.to_le_bytes()); // k2
    blk_int3.extend_from_slice(&99i32.to_le_bytes());
    push_str(&mut blk_int3, "triple");

    let chunk_int3 = build_chunk_bin(&[Some(&blk_int3)], 4);
    let t3: BinTable<(i32, i32, i32), TestSimpleRow> =
        parse_chunk_bin_table(&chunk_int3, &mut scratch).expect("parse Int3");
    assert_eq!(t3.get(&(1, 2, 3)).map(|r| r.value), Some(99));
}

#[test]
fn chunk_bin_round_trip_int2_and_str_key_schemas() {
    let mut scratch = BinScratch::with_limits(BinLimits::DEFAULT);

    let mut blk_int2 = Vec::new();
    blk_int2.extend_from_slice(&1u32.to_le_bytes());
    blk_int2.extend_from_slice(&7i32.to_le_bytes());
    blk_int2.extend_from_slice(&4u32.to_le_bytes());
    blk_int2.extend_from_slice(&1u32.to_le_bytes());
    blk_int2.extend_from_slice(&8i32.to_le_bytes());
    blk_int2.extend_from_slice(&77i32.to_le_bytes());
    push_str(&mut blk_int2, "pair");
    let t2: BinTable<(i32, i32), TestSimpleRow> =
        parse_chunk_bin_table(&build_chunk_bin(&[Some(&blk_int2)], 4), &mut scratch)
            .expect("parse Int2");
    assert_eq!(t2.get(&(7, 8)).map(|r| &*r.label), Some("pair"));

    let mut blk_str_int = Vec::new();
    blk_str_int.extend_from_slice(&1u32.to_le_bytes());
    push_str(&mut blk_str_int, "hero_a");
    blk_str_int.extend_from_slice(&4u32.to_le_bytes());
    blk_str_int.extend_from_slice(&1u32.to_le_bytes());
    blk_str_int.extend_from_slice(&5i32.to_le_bytes());
    blk_str_int.extend_from_slice(&123i32.to_le_bytes());
    push_str(&mut blk_str_int, "nested");
    let t_str: BinTable<(Box<str>, i32), TestSimpleRow> =
        parse_chunk_bin_table(&build_chunk_bin(&[Some(&blk_str_int)], 4), &mut scratch)
            .expect("parse StrInt1");
    assert_eq!(t_str.get(&("hero_a".into(), 5)).map(|r| r.value), Some(123));
    assert_eq!(t_str.rows().len(), 1);
}

#[test]
fn map_bin_multi_language_and_affix_effect_round_trip() {
    let mut map_buf = Vec::new();
    map_buf.extend_from_slice(&4u32.to_le_bytes()); // outer_buckets
    map_buf.extend_from_slice(&1u32.to_le_bytes()); // top_count
    map_buf.extend_from_slice(&55i32.to_le_bytes()); // key
    map_buf.extend_from_slice(&777i32.to_le_bytes()); // row.value
    push_str(&mut map_buf, "map_point");

    let map_table: BinTable<i32, TestSimpleRow> =
        parse_map_bin_table(&map_buf, BinLimits::DEFAULT).expect("parse map bin");
    assert_eq!(
        map_table.get(&55),
        Some(&TestSimpleRow {
            label: "map_point".into(),
            value: 777,
        })
    );

    let mut ml_buf = Vec::new();
    ml_buf.extend_from_slice(&4u32.to_le_bytes()); // buckets
    ml_buf.extend_from_slice(&1u32.to_le_bytes()); // top_count
    push_str(&mut ml_buf, "table_ui");
    push_str(&mut ml_buf, "UI_OK");
    push_str(&mut ml_buf, "dev");
    push_str(&mut ml_buf, "Confirm");
    let ml_table =
        parse_multi_language_table(&ml_buf, BinLimits::DEFAULT).expect("parse multi_language");
    assert_eq!(
        ml_table.get(&("table_ui".into(), "UI_OK".into())),
        Some(&MultiLanguageRow {
            author: "dev".into(),
            text: "Confirm".into(),
        })
    );

    let mut ae_buf = Vec::new();
    ae_buf.extend_from_slice(&4u32.to_le_bytes()); // buckets
    ae_buf.extend_from_slice(&1u32.to_le_bytes()); // top_count
    ae_buf.extend_from_slice(&100i32.to_le_bytes()); // id
    ae_buf.extend_from_slice(&200i32.to_le_bytes()); // sub_id
    ae_buf.extend_from_slice(&200i32.to_le_bytes()); // sub_id_inner
    ae_buf.extend_from_slice(&100i32.to_le_bytes()); // id_inner
    ae_buf.extend_from_slice(&1i32.to_le_bytes()); // param0
    push_str(&mut ae_buf, "{\"trig\":1}");
    push_str(&mut ae_buf, "");
    push_str(&mut ae_buf, "{\"cond2\":2}");
    push_str(&mut ae_buf, "");
    push_str(&mut ae_buf, "{\"fx\":1}");
    ae_buf.extend_from_slice(&6i32.to_le_bytes()); // param1
    let ae_table =
        parse_affix_effect_table(&ae_buf, BinLimits::DEFAULT).expect("parse affix_effect");
    assert_eq!(
        ae_table.get(&(100, 200)),
        Some(&AffixEffectRow {
            trigger_json: "{\"trig\":1}".into(),
            condition1_json: "".into(),
            condition2_json: "{\"cond2\":2}".into(),
            pre_effect_json: "".into(),
            effect_json: "{\"fx\":1}".into(),
            id: 100,
            sub_id: 200,
            sub_id_inner: 200,
            id_inner: 100,
            param0: 1,
            param1: 6,
        })
    );
}

#[test]
fn wire_cursor_vec_dim_affix_tree_action_data_and_config_c() {
    let mut raw = Vec::new();
    raw.push(0); // VecDim::Vec2([0.0, 0.0])
    raw.push(2); // VecDim::Vec2([1.5, 2.5])
    raw.extend_from_slice(&1.5f64.to_le_bytes());
    raw.extend_from_slice(&2.5f64.to_le_bytes());
    raw.extend_from_slice(&10i32.to_le_bytes()); // FConfigPairI32.a
    raw.extend_from_slice(&20i32.to_le_bytes()); // FConfigPairI32.b
    raw.extend_from_slice(&1004i32.to_le_bytes()); // ActionData::Tag1004
    raw.extend_from_slice(&3.25f64.to_le_bytes());

    // FConfigAffix with 1 root formula containing 3 nodes: Op3(Op15 -> 1 FormulaOrI32), LookupTable, Tag8
    raw.extend_from_slice(&99i32.to_le_bytes()); // id
    raw.push(1); // has_tree
    raw.extend_from_slice(&1u32.to_le_bytes()); // root_count = 1
    raw.push(1); // root FormulaOrF64 is_formula = 1
    raw.extend_from_slice(&3u32.to_le_bytes()); // node_count = 3
    raw.push(3); // node 0: tag 3 (Op3)
    raw.push(15); // Op15 (1 FormulaOrI32 operand)
    raw.push(0); // Op15 operand 0: is_formula = 0 (literal i32)
    raw.extend_from_slice(&77i32.to_le_bytes());
    raw.push(4); // node 1: tag 4 (LookupTable)
    raw.extend_from_slice(&2u32.to_le_bytes()); // buckets
    raw.extend_from_slice(&1u32.to_le_bytes()); // count = 1
    raw.extend_from_slice(&1i32.to_le_bytes());
    raw.extend_from_slice(&2i32.to_le_bytes());
    raw.extend_from_slice(&3i32.to_le_bytes());
    raw.push(8); // node 2: tag 8

    let mut cur = WireCursor::new(&raw, BinLimits::DEFAULT);
    assert_eq!(cur.read_vec_dim().unwrap(), VecDim::Vec2([0.0, 0.0]));
    assert_eq!(cur.read_vec_dim().unwrap(), VecDim::Vec2([1.5, 2.5]));
    assert_eq!(
        FConfigI32I32::read(&mut cur).unwrap(),
        FConfigI32I32 { f0: 10, f1: 20 }
    );
    assert_eq!(
        ActionData::read(&mut cur).unwrap(),
        ActionData::Tag1004(ActionData1004 { f0: 3.25 })
    );
    let affix = FConfigAffix::read(&mut cur).unwrap();
    assert_eq!(affix.id, 99);
    assert_eq!(affix.root_count, 1);
    assert_eq!(
        &*affix.operands,
        &[
            AffixOperand::Formula { node_count: 3 },
            AffixOperand::LiteralI32(77),
        ]
    );
    assert_eq!(
        &*affix.nodes,
        &[
            AffixTreeNode::Op3(AffixOp3Code::Op15),
            AffixTreeNode::LookupTable(
                vec![AffixLookupEntry {
                    key: 1,
                    val_a: 2,
                    val_b: 3,
                }]
                .into_boxed_slice()
            ),
            AffixTreeNode::Tag8,
        ]
    );
    assert!(cur.is_empty());
}

#[test]
fn generated_config_c_and_stem_normalization() {
    assert_eq!(
        normalize_chunk_bin_stem("config_ui_style_CN_ChunkA0"),
        "config_ui_style"
    );
    assert_eq!(
        normalize_chunk_bin_stem("config_ui_style_v2_EN_ChunkA0"),
        "config_ui_style_v2"
    );
    assert_eq!(
        normalize_chunk_bin_stem("config_item_base_ChunkA0"),
        "config_item_base"
    );

    let mut blk = Vec::new();
    blk.extend_from_slice(&1u32.to_le_bytes());
    blk.extend_from_slice(&1i32.to_le_bytes()); // outer key = 1
    blk.extend_from_slice(&1i32.to_le_bytes()); // id = 1
    push_str(&mut blk, "int"); // type
    push_str(&mut blk, "100"); // param
    push_str(&mut blk, "core"); // sys
    push_str(&mut blk, "desc"); // intro
    push_str(&mut blk, "desc7"); // intro7

    let chunk = build_chunk_bin(&[Some(&blk)], 4);
    let mut scratch = BinScratch::with_limits(BinLimits::DEFAULT);
    let table = parse_config_c(&chunk, &mut scratch).expect("parse config_c");
    let row = table.get(&1).expect("row 1");
    assert_eq!(row.id, 1);
    assert_eq!(&*row.r#type, "int");
    assert_eq!(&*row.param, "100");
    assert_eq!(&*row.sys, "core");
}

#[test]
#[ignore = "requires TLI_GAME_DIR and TLI_AES_KEY"]
fn tli_game_dir_all_bin_tables() {
    let game_dir = std::env::var("TLI_GAME_DIR").expect("TLI_GAME_DIR must be set");
    let key_hex = std::env::var("TLI_AES_KEY").expect("TLI_AES_KEY must be set");
    let key = pak::AesKey::try_from_hex(key_hex.trim()).expect("valid TLI_AES_KEY");
    let vfs = pak_vfs::PakVfs::open_tli_game_dir(Path::new(&game_dir), Some(key))
        .expect("open TLI_GAME_DIR PakVfs");

    let mut pak_scratch = pak::ReadScratch::with_limits(pak::ScratchLimits::DEFAULT);
    let mut bin_scratch = BinScratch::with_limits(BinLimits::DEFAULT);
    let mut buf = Vec::new();
    let mut path_buf = Vec::new();

    let mut chunk_files = 0u32;
    let mut chunk_rows = 0u32;
    let mut map_files = 0u32;
    let mut map_rows = 0u32;
    let mut ml_files = 0u32;
    let mut ml_rows = 0u32;
    let mut affix_effect_rows = 0u32;

    for fid in vfs.iter_files() {
        path_buf.clear();
        vfs.write_file_path(fid, &mut path_buf);
        let path_str = core::str::from_utf8(&path_buf).expect("valid vfs path");
        if !path_str.ends_with(".bin") || path_str.ends_with("_idx.bin") {
            continue;
        }
        let raw_len = u32::try_from(vfs.file_meta(fid).uncompressed_bytes()).expect("u32 size");
        buf.resize(crate::u32_to_usize(raw_len), 0);
        vfs.read_into(fid, &mut pak_scratch, &mut buf)
            .expect("read_into .bin");
        let fname = path_str.rsplit('/').next().expect("filename");

        if let Some(stem) = fname.strip_suffix("_ChunkA0.bin") {
            chunk_rows +=
                verify_all_chunk_bin_by_stem(stem, &buf, &mut bin_scratch).expect("chunk bin");
            chunk_files += 1;
        } else if path_str.contains("/Maps/") {
            let stem = fname.strip_suffix(".bin").expect(".bin");
            map_rows +=
                verify_all_map_bin_by_stem(stem, &buf, BinLimits::DEFAULT).expect("map bin");
            map_files += 1;
        } else if fname == "config_affix_effect.bin" {
            affix_effect_rows += parse_affix_effect_table(&buf, BinLimits::DEFAULT)
                .expect("affix_effect")
                .len();
        } else if fname.starts_with("config_multi_language") {
            ml_rows += parse_multi_language_table(&buf, BinLimits::DEFAULT)
                .expect("multi_language")
                .len();
            ml_files += 1;
        }
    }

    assert_eq!(chunk_files, 361);
    assert_eq!(chunk_rows, 343_180);
    assert_eq!(map_files, 10_408);
    assert_eq!(map_rows, 129_312);
    assert_eq!(affix_effect_rows, 15_950);
    assert_eq!(ml_files, 14);
    assert_eq!(ml_rows, 1_380_809);
}
