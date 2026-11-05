use crate::luac::opcode::LuaOpcode;
use crate::luac::tests::{
    TestConst, TestProto, build_xdi_chunk, encode_xdi_abc, encode_xdi_abx, encode_xdi_ax,
};
use crate::luac::transpile::{ChunkArenas, load_chunk_into};
use crate::luac::types::{LUA53_HEADER, XDI_MAGIC};
use crate::luac::{
    DataCfgError, DataCfgLimits, DataCfgRow, DataCfgScratch, DataTable, DecompileConfig,
    DecompileError, DecompileScratch, DecompiledChunkKind, EvalContext, EvalVal, LuacError,
    LuacLimits, decompile_luac, parse_luatable_datacfg,
};

#[derive(Debug, Clone, PartialEq, Default)]
struct NarrowRow {
    id: u16,
    name: Box<str>,
    level: u8,
}

impl DataCfgRow for NarrowRow {
    const TABLE_NAME: &'static str = "config_narrow";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Level" => self.level = ctx.decode_field(val, Self::TABLE_NAME, "Level")?,
            other => return Err(ctx.unknown_field(Self::TABLE_NAME, other)),
        }
        Ok(())
    }
}

#[test]
fn reject_invalid_magic_and_lua53_header() {
    let mut bad = vec![0xDE, 0xAD, 0xBE, 0xEF, 0, 0, 0, 0, 0, 0];
    let mut scratch = DecompileScratch::with_limits(DecompileConfig::DEFAULT.limits());
    let err = decompile_luac(&mut bad, &mut scratch, &DecompileConfig::DEFAULT).unwrap_err();
    assert!(matches!(err, LuacError::InvalidMagic));

    let mut bad_lua53 = Vec::from(LUA53_HEADER);
    bad_lua53[4] = 0x54; // Lua 5.4 instead of 5.3
    bad_lua53.push(1);
    let err_hdr =
        decompile_luac(&mut bad_lua53, &mut scratch, &DecompileConfig::DEFAULT).unwrap_err();
    assert!(matches!(err_hdr, LuacError::InvalidLua53Header));
}

#[test]
fn reject_truncated_buffers() {
    let mut proto = TestProto::new();
    proto.code = vec![encode_xdi_abc(LuaOpcode::Return, 0, 1, 0)];
    let full = build_xdi_chunk(&proto);
    let mut arenas = ChunkArenas::with_limits(LuacLimits::DEFAULT);

    let full_len = crate::bounded_len_u32(full.len());
    for cut in [0_u32, 4, 9, 10, 15, full_len - 1] {
        let err = load_chunk_into(
            &full[..crate::u32_to_usize(cut)],
            &mut arenas,
            LuacLimits::DEFAULT,
        )
        .unwrap_err();
        if cut < 4 {
            assert!(matches!(
                err,
                LuacError::InvalidMagic | LuacError::UnexpectedEof { .. }
            ));
        } else {
            assert!(matches!(err, LuacError::UnexpectedEof { .. }));
        }
    }
}

#[test]
fn reject_invalid_xdi_and_lua53_string_tags_and_lengths() {
    let mut arenas = ChunkArenas::with_limits(LuacLimits::DEFAULT);
    for bad_tag in 1..=9_u8 {
        let mut buf = Vec::from(XDI_MAGIC);
        buf.push(1); // main_upvalues
        buf.push(bad_tag);
        let err = load_chunk_into(&buf, &mut arenas, LuacLimits::DEFAULT).unwrap_err();
        assert!(matches!(
            err,
            LuacError::InvalidXdiStringTag { tag, .. } if tag == bad_tag
        ));
    }

    for bad_raw_len in 0..=9_u32 {
        let mut buf = Vec::from(XDI_MAGIC);
        buf.push(1);
        buf.push(0xFF);
        buf.extend_from_slice(&bad_raw_len.to_le_bytes());
        let err = load_chunk_into(&buf, &mut arenas, LuacLimits::DEFAULT).unwrap_err();
        assert!(matches!(
            err,
            LuacError::InvalidXdiLongStringLength { raw_len, .. } if raw_len == bad_raw_len
        ));
    }

    // Standard Lua 5.3 long string with raw_len == 0
    let mut lua53_buf = Vec::from(LUA53_HEADER);
    lua53_buf.push(1);
    lua53_buf.push(0xFF);
    lua53_buf.extend_from_slice(&0_u64.to_le_bytes());
    let err = load_chunk_into(&lua53_buf, &mut arenas, LuacLimits::DEFAULT).unwrap_err();
    assert!(matches!(
        err,
        LuacError::InvalidLua53StringLength { raw_len: 0, .. }
    ));
}

#[test]
fn reject_null_constant_string_invalid_const_tag_and_nan_float() {
    let mut arenas = ChunkArenas::with_limits(LuacLimits::DEFAULT);
    for str_tag in [0x04_u8, 0x14_u8] {
        let mut buf = Vec::from(XDI_MAGIC);
        buf.push(1);
        buf.push(0x00); // source = None
        buf.extend_from_slice(&[0; 11]); // metadata
        buf.extend_from_slice(&0_u32.to_le_bytes()); // size_code = 0
        buf.extend_from_slice(&1_u32.to_le_bytes()); // size_k = 1
        buf.push(str_tag);
        buf.push(0x00); // null string inside constant pool
        let err = load_chunk_into(&buf, &mut arenas, LuacLimits::DEFAULT).unwrap_err();
        assert!(matches!(
            err,
            LuacError::NullConstantString { tag, .. } if tag == str_tag
        ));
    }

    // Invalid constant tag 0x99
    let mut buf = Vec::from(XDI_MAGIC);
    buf.push(1);
    buf.push(0x00);
    buf.extend_from_slice(&[0; 11]);
    buf.extend_from_slice(&0_u32.to_le_bytes());
    buf.extend_from_slice(&1_u32.to_le_bytes());
    buf.push(0x99);
    let err = load_chunk_into(&buf, &mut arenas, LuacLimits::DEFAULT).unwrap_err();
    assert!(matches!(
        err,
        LuacError::InvalidConstantTag { tag: 0x99, .. }
    ));

    // NaN / Inf float constants
    for bad_float in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut proto = TestProto::new();
        proto.consts = vec![TestConst::Float(bad_float)];
        proto.code = vec![encode_xdi_abc(LuaOpcode::Return, 0, 1, 0)];
        let chunk = build_xdi_chunk(&proto);
        let err = load_chunk_into(&chunk, &mut arenas, LuacLimits::DEFAULT).unwrap_err();
        assert!(matches!(err, LuacError::NonFiniteFloatConstant { .. }));
    }
}

#[test]
fn reject_invalid_opcodes_and_trailing_bytes() {
    let mut arenas = ChunkArenas::with_limits(LuacLimits::DEFAULT);
    for raw_op in 47..=63_u8 {
        let mut proto = TestProto::new();
        proto.code = vec![u32::from(raw_op) << 26];
        let chunk = build_xdi_chunk(&proto);
        let err = load_chunk_into(&chunk, &mut arenas, LuacLimits::DEFAULT).unwrap_err();
        assert!(matches!(
            err,
            LuacError::InvalidXdiOpcode { raw_op: op, .. } if op == raw_op
        ));
    }

    let mut proto = TestProto::new();
    proto.code = vec![encode_xdi_abc(LuaOpcode::Return, 0, 1, 0)];
    let mut chunk = build_xdi_chunk(&proto);
    chunk.push(0xAA);
    let err = load_chunk_into(&chunk, &mut arenas, LuacLimits::DEFAULT).unwrap_err();
    assert!(matches!(err, LuacError::TrailingBytes { .. }));
}

#[test]
fn reject_decompile_strict_mode_out_of_bounds_const_and_upvalue() {
    let strict_cfg = DecompileConfig::DEFAULT.with_strict_mode(true);
    let perm_cfg = DecompileConfig::DEFAULT.with_strict_mode(false);
    let mut scratch = DecompileScratch::with_limits(strict_cfg.limits());
    let mut run_inst = |inst: u32, cfg: &DecompileConfig| {
        let mut p = TestProto::new();
        p.code = vec![inst, encode_xdi_abc(LuaOpcode::Return, 0, 2, 0)];
        let mut chunk = build_xdi_chunk(&p);
        decompile_luac(&mut chunk, &mut scratch, cfg)
    };

    for bad_const_inst in [
        encode_xdi_abx(LuaOpcode::LoadK, 0, 5),
        encode_xdi_abc(LuaOpcode::Add, 0, 0, 0x100 | 5),
    ] {
        let err_const = run_inst(bad_const_inst, &strict_cfg).unwrap_err();
        assert!(matches!(
            err_const,
            LuacError::Decompile {
                source: DecompileError::ConstantIndexOutOfBounds {
                    proto_index: 0,
                    pc: 0,
                    index: 5,
                    count: 0,
                }
            }
        ));
        let perm_const = run_inst(bad_const_inst, &perm_cfg).unwrap();
        assert_eq!(perm_const.kind(), DecompiledChunkKind::DiagnosticPseudocode);
    }

    let up_inst = encode_xdi_abc(LuaOpcode::GetUpval, 0, 9, 0);
    let err_up = run_inst(up_inst, &strict_cfg).unwrap_err();
    assert!(matches!(
        err_up,
        LuacError::Decompile {
            source: DecompileError::UpvalueIndexOutOfBounds {
                proto_index: 0,
                pc: 0,
                index: 9,
                count: 1,
            }
        }
    ));
    assert_eq!(
        run_inst(up_inst, &perm_cfg).unwrap().kind(),
        DecompiledChunkKind::DiagnosticPseudocode
    );
}

#[test]
fn reject_decompile_strict_mode_out_of_bounds_child_and_missing_extraarg() {
    let strict_cfg = DecompileConfig::DEFAULT.with_strict_mode(true);
    let perm_cfg = DecompileConfig::DEFAULT.with_strict_mode(false);
    let mut scratch = DecompileScratch::with_limits(strict_cfg.limits());

    let mut p_child = TestProto::new();
    p_child.code = vec![
        encode_xdi_abx(LuaOpcode::Closure, 0, 3),
        encode_xdi_abc(LuaOpcode::Return, 0, 2, 0),
    ];
    let mut chunk_child = build_xdi_chunk(&p_child);
    let err_child = decompile_luac(&mut chunk_child, &mut scratch, &strict_cfg).unwrap_err();
    assert!(matches!(
        err_child,
        LuacError::Decompile {
            source: DecompileError::ChildProtoIndexOutOfBounds {
                proto_index: 0,
                pc: 0,
                index: 3,
                count: 0,
            }
        }
    ));
    assert_eq!(
        decompile_luac(&mut chunk_child, &mut scratch, &perm_cfg)
            .unwrap()
            .kind(),
        DecompiledChunkKind::DiagnosticPseudocode
    );

    for code in [
        vec![encode_xdi_abx(LuaOpcode::LoadKx, 0, 0)],
        vec![
            encode_xdi_abx(LuaOpcode::LoadKx, 0, 0),
            encode_xdi_abc(LuaOpcode::Return, 0, 2, 0),
        ],
    ] {
        let mut p_kx = TestProto::new();
        p_kx.code = code;
        let mut chunk_kx = build_xdi_chunk(&p_kx);
        let err_kx = decompile_luac(&mut chunk_kx, &mut scratch, &strict_cfg).unwrap_err();
        assert!(matches!(
            err_kx,
            LuacError::Decompile {
                source: DecompileError::MissingExtraArg {
                    proto_index: 0,
                    pc: 0,
                }
            }
        ));
        assert_eq!(
            decompile_luac(&mut chunk_kx, &mut scratch, &perm_cfg)
                .unwrap()
                .kind(),
            DecompiledChunkKind::DiagnosticPseudocode
        );
    }
}

#[test]
fn reject_decompile_unmatched_loop_end_and_invalid_utf8() {
    let strict_cfg = DecompileConfig::DEFAULT.with_strict_mode(true);
    let perm_cfg = DecompileConfig::DEFAULT.with_strict_mode(false);
    let mut scratch = DecompileScratch::with_limits(strict_cfg.limits());

    for loop_op in [LuaOpcode::ForLoop, LuaOpcode::TForLoop] {
        let mut p = TestProto::new();
        p.code = vec![
            encode_xdi_abx(loop_op, 0, 131_071),
            encode_xdi_abc(LuaOpcode::Return, 0, 1, 0),
        ];
        let mut chunk = build_xdi_chunk(&p);
        let err = decompile_luac(&mut chunk, &mut scratch, &strict_cfg).unwrap_err();
        assert!(matches!(
            err,
            LuacError::Decompile {
                source: DecompileError::UnmatchedBlockEnd {
                    proto_index: 0,
                    pc: 0,
                }
            }
        ));
        let perm = decompile_luac(&mut chunk, &mut scratch, &perm_cfg).unwrap();
        assert_eq!(perm.kind(), DecompiledChunkKind::DiagnosticPseudocode);
    }

    // Non-UTF-8 string constant and local variable name
    let mut p_utf8 = TestProto::new();
    p_utf8.consts = vec![TestConst::Str(vec![0xFF, 0xFE])];
    p_utf8.loc_vars = vec![(vec![0xFF, 0xFE], 0, 2)];
    p_utf8.code = vec![
        encode_xdi_abx(LuaOpcode::LoadK, 0, 0),
        encode_xdi_abc(LuaOpcode::Return, 0, 2, 0),
    ];
    let mut chunk_utf8 = build_xdi_chunk(&p_utf8);
    let err_utf8 = decompile_luac(&mut chunk_utf8, &mut scratch, &strict_cfg).unwrap_err();
    assert!(matches!(err_utf8, LuacError::InvalidUtf8 { .. }));

    let perm_utf8 = decompile_luac(&mut chunk_utf8, &mut scratch, &perm_cfg).unwrap();
    assert_eq!(perm_utf8.kind(), DecompiledChunkKind::DiagnosticPseudocode);
    assert!(perm_utf8.source().contains("\\xFF\\xFE"));
}

fn build_single_row_chunk(field_name: &[u8], val: TestConst) -> Vec<u8> {
    let mut proto = TestProto::new();
    proto.consts = vec![
        TestConst::Str(b"Id".to_vec()),
        TestConst::Str(field_name.to_vec()),
        TestConst::Int(1),
        val,
    ];
    proto.code = vec![
        encode_xdi_abc(LuaOpcode::NewTable, 0, 0, 0),
        encode_xdi_abc(LuaOpcode::NewTable, 1, 0, 0),
        encode_xdi_abc(LuaOpcode::SetTable, 1, 0x100, 0x100 | 2),
        encode_xdi_abc(LuaOpcode::SetTable, 1, 0x100 | 1, 0x100 | 3),
        encode_xdi_abc(LuaOpcode::SetTable, 0, 0x100 | 2, 1),
        encode_xdi_abc(LuaOpcode::NewTable, 1, 0, 0),
        encode_xdi_abc(LuaOpcode::NewTable, 2, 0, 0),
        encode_xdi_abc(LuaOpcode::NewTable, 3, 0, 0),
        encode_xdi_abc(LuaOpcode::NewTable, 4, 0, 0),
        encode_xdi_abx(LuaOpcode::Closure, 5, 0),
    ];
    let mut child = TestProto::new();
    child.code = vec![encode_xdi_abc(LuaOpcode::Return, 0, 1, 0)];
    proto.children.push(child);
    build_xdi_chunk(&proto)
}

#[test]
fn reject_datacfg_unknown_field_type_mismatch_and_integer_overflow() {
    let mut scratch = DataCfgScratch::with_limits(DataCfgLimits::DEFAULT);

    // 1. Unknown column name
    let mut unknown_chunk = build_single_row_chunk(b"BogusColumn", TestConst::Int(5));
    let err_unknown =
        parse_luatable_datacfg::<u16, NarrowRow>(&mut unknown_chunk, &mut scratch).unwrap_err();
    assert!(matches!(
        err_unknown,
        LuacError::DataCfg {
            source: DataCfgError::UnknownField { table: "config_narrow", ref field }
        } if &**field == "BogusColumn"
    ));

    // 2. Type mismatch (int passed to Box<str> field "Name")
    let mut mismatch_chunk = build_single_row_chunk(b"Name", TestConst::Int(99));
    let err_mismatch =
        parse_luatable_datacfg::<u16, NarrowRow>(&mut mismatch_chunk, &mut scratch).unwrap_err();
    assert!(matches!(
        err_mismatch,
        LuacError::DataCfg {
            source: DataCfgError::TypeMismatch {
                table: "config_narrow",
                field: "Name",
                expected: "Box<str>",
            }
        }
    ));

    // 3. Narrowed integer overflow (300 > u8::MAX for "Level")
    let mut overflow_chunk = build_single_row_chunk(b"Level", TestConst::Int(300));
    let err_overflow =
        parse_luatable_datacfg::<u16, NarrowRow>(&mut overflow_chunk, &mut scratch).unwrap_err();
    assert!(matches!(
        err_overflow,
        LuacError::DataCfg {
            source: DataCfgError::IntegerOverflow {
                table: "config_narrow",
                field: "Level",
                value: 300,
                target_type: "u8",
            }
        }
    ));

    // 4. Invalid UTF-8 string in field value
    let mut bad_utf8_chunk = build_single_row_chunk(b"Name", TestConst::Str(vec![0xFF, 0xFE]));
    let err_utf8 =
        parse_luatable_datacfg::<u16, NarrowRow>(&mut bad_utf8_chunk, &mut scratch).unwrap_err();
    assert!(matches!(err_utf8, LuacError::InvalidUtf8 { .. }));
}

#[test]
fn reject_datacfg_invalid_vm_instructions() {
    let mut scratch = DataCfgScratch::with_limits(DataCfgLimits::DEFAULT);

    // 1. Unsupported instruction (ADD) before mode_readonly epilogue
    let mut p_add = TestProto::new();
    p_add.code = vec![
        encode_xdi_abc(LuaOpcode::Add, 0, 1, 2),
        encode_xdi_abc(LuaOpcode::NewTable, 4, 0, 0),
        encode_xdi_abx(LuaOpcode::Closure, 5, 0),
    ];
    let mut chunk_add = build_xdi_chunk(&p_add);
    let err_add =
        parse_luatable_datacfg::<u16, NarrowRow>(&mut chunk_add, &mut scratch).unwrap_err();
    assert!(matches!(
        err_add,
        LuacError::DataCfg {
            source: DataCfgError::UnsupportedInstruction { pc: 0, .. }
        }
    ));

    // 2. LOADBOOL with C != 0
    let mut p_lb = TestProto::new();
    p_lb.code = vec![
        encode_xdi_abc(LuaOpcode::LoadBool, 0, 1, 1),
        encode_xdi_abc(LuaOpcode::NewTable, 4, 0, 0),
        encode_xdi_abx(LuaOpcode::Closure, 5, 0),
    ];
    let mut chunk_lb = build_xdi_chunk(&p_lb);
    let err_lb = parse_luatable_datacfg::<u16, NarrowRow>(&mut chunk_lb, &mut scratch).unwrap_err();
    assert!(matches!(
        err_lb,
        LuacError::DataCfg {
            source: DataCfgError::LoadBoolSkipNotAllowed { pc: 0, c: 1 }
        }
    ));

    // 3. LOADKX without EXTRAARG
    let mut p_kx = TestProto::new();
    p_kx.code = vec![
        encode_xdi_abx(LuaOpcode::LoadKx, 0, 0),
        encode_xdi_abc(LuaOpcode::NewTable, 4, 0, 0),
        encode_xdi_abx(LuaOpcode::Closure, 5, 0),
    ];
    let mut chunk_kx = build_xdi_chunk(&p_kx);
    let err_kx = parse_luatable_datacfg::<u16, NarrowRow>(&mut chunk_kx, &mut scratch).unwrap_err();
    assert!(matches!(
        err_kx,
        LuacError::DataCfg {
            source: DataCfgError::MissingExtraArg { pc: 0 }
        }
    ));
}

#[test]
fn reject_datacfg_invalid_setlist_register_span_and_batch_zero() {
    let mut scratch = DataCfgScratch::with_limits(DataCfgLimits::DEFAULT);

    // 1. SETLIST with out-of-bounds register span (A=250, B=10 -> 260 > 255)
    let mut p_sl_oob = TestProto::new();
    p_sl_oob.code = vec![
        encode_xdi_abc(LuaOpcode::NewTable, 250, 0, 0),
        encode_xdi_abc(LuaOpcode::SetList, 250, 10, 1),
        encode_xdi_abc(LuaOpcode::NewTable, 4, 0, 0),
        encode_xdi_abx(LuaOpcode::Closure, 5, 0),
    ];
    let mut chunk_sl_oob = build_xdi_chunk(&p_sl_oob);
    let err_sl_oob =
        parse_luatable_datacfg::<u16, NarrowRow>(&mut chunk_sl_oob, &mut scratch).unwrap_err();
    assert!(matches!(
        err_sl_oob,
        LuacError::DataCfg {
            source: DataCfgError::InvalidSetListCount {
                pc: 1,
                a: 250,
                b: 10,
            }
        }
    ));

    // 2. SETLIST C=0 followed by EXTRAARG with ax=0
    let mut p_sl_batch0 = TestProto::new();
    p_sl_batch0.code = vec![
        encode_xdi_abc(LuaOpcode::NewTable, 0, 0, 0),
        encode_xdi_abc(LuaOpcode::SetList, 0, 1, 0),
        encode_xdi_ax(LuaOpcode::ExtraArg, 0),
        encode_xdi_abc(LuaOpcode::NewTable, 4, 0, 0),
        encode_xdi_abx(LuaOpcode::Closure, 5, 0),
    ];
    let mut chunk_sl_batch0 = build_xdi_chunk(&p_sl_batch0);
    let err_sl_batch0 =
        parse_luatable_datacfg::<u16, NarrowRow>(&mut chunk_sl_batch0, &mut scratch).unwrap_err();
    assert!(matches!(
        err_sl_batch0,
        LuacError::DataCfg {
            source: DataCfgError::SetListBatchMismatch {
                pc: 1,
                expected_len: u32::MAX,
                actual_len: 0,
            }
        }
    ));
}

#[test]
fn reject_datacfg_missing_epilogue_and_mixed_table_kind() {
    let mut scratch = DataCfgScratch::with_limits(DataCfgLimits::DEFAULT);

    // 4. Missing mode_readonly boundary before Return
    let mut p_no_ro = TestProto::new();
    p_no_ro.code = vec![
        encode_xdi_abc(LuaOpcode::NewTable, 0, 0, 0),
        encode_xdi_abc(LuaOpcode::Return, 0, 2, 0),
    ];
    let mut chunk_no_ro = build_xdi_chunk(&p_no_ro);
    let err_no_ro =
        parse_luatable_datacfg::<u16, NarrowRow>(&mut chunk_no_ro, &mut scratch).unwrap_err();
    assert!(matches!(
        err_no_ro,
        LuacError::DataCfg {
            source: DataCfgError::MissingModeReadonlyBoundary
        }
    ));

    // 5. Mixed table kind (both SETLIST and SETTABLE on same table)
    let mut p_mixed = TestProto::new();
    p_mixed.consts = vec![TestConst::Int(1)];
    p_mixed.code = vec![
        encode_xdi_abc(LuaOpcode::NewTable, 0, 0, 0),
        encode_xdi_abx(LuaOpcode::LoadK, 1, 0),
        encode_xdi_abc(LuaOpcode::SetList, 0, 1, 1),
        encode_xdi_abc(LuaOpcode::SetTable, 0, 0x100, 0x100),
        encode_xdi_abc(LuaOpcode::NewTable, 4, 0, 0),
        encode_xdi_abx(LuaOpcode::Closure, 5, 0),
    ];
    let mut chunk_mixed = build_xdi_chunk(&p_mixed);
    let err_mixed =
        parse_luatable_datacfg::<u16, NarrowRow>(&mut chunk_mixed, &mut scratch).unwrap_err();
    assert!(matches!(
        err_mixed,
        LuacError::DataCfg {
            source: DataCfgError::MixedTableKind { .. }
        }
    ));

    let _: Option<DataTable<u16, NarrowRow>> = None;
}

#[test]
fn reject_unknown_datacfg_stem_and_malformed_nested_records() {
    let mut scratch = DataCfgScratch::with_limits(DataCfgLimits::DEFAULT);
    let mut empty_buf = Vec::new();
    let err_stem = super::tables::verify_all_datacfg_by_stem(
        "config_nonexistent_table",
        &mut empty_buf,
        &mut scratch,
    )
    .unwrap_err();
    assert!(matches!(
        err_stem,
        LuacError::DataCfg {
            source: DataCfgError::UnknownField {
                table: "<datacfg_corpus>",
                ..
            }
        }
    ));

    // Malformed nested field type on generated ConfigRarityEquipRow (AffixSlotNum expects list of RarityEquipAffixSlotNumEntry, given integer)
    let mut bad_nested = build_single_row_chunk(b"AffixSlotNum", TestConst::Int(99));
    let err_nested =
        super::tables::parse_config_rarity_equip(&mut bad_nested, &mut scratch).unwrap_err();
    assert!(matches!(
        err_nested,
        LuacError::DataCfg {
            source: DataCfgError::TypeMismatch {
                table: "config_rarity_equip",
                field: "AffixSlotNum",
                ..
            }
        }
    ));
}
