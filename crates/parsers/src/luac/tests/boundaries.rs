use crate::luac::opcode::LuaOpcode;
use crate::luac::tests::{TestConst, TestProto, build_xdi_chunk, encode_xdi_abc, encode_xdi_abx};
use crate::luac::transpile::transpile_to_lua53_into;
use crate::luac::{
    DataCfgLimits, DataCfgRow, DataCfgScratch, DecompileConfig, DecompileScratch, EvalContext,
    EvalVal, LuacError, LuacLimitKind, LuacLimits, decompile_luac, parse_luatable_datacfg,
};

#[derive(Debug, Clone, PartialEq, Default)]
struct BoundaryRow {
    id: u32,
    items: Box<[u32]>,
}

impl DataCfgRow for BoundaryRow {
    const TABLE_NAME: &'static str = "config_boundary";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Items" => self.items = ctx.decode_field(val, Self::TABLE_NAME, "Items")?,
            other => return Err(ctx.unknown_field(Self::TABLE_NAME, other)),
        }
        Ok(())
    }
}

#[test]
fn string_length_encoding_boundaries() {
    for len in [0_u32, 1, 39, 40, 41, 243, 244, 245, 252, 253, 254, 255, 512] {
        let payload = vec![b'a'; crate::u32_to_usize(len)];
        let mut proto = TestProto::new();
        proto.consts = vec![TestConst::Str(payload.clone())];
        proto.code = vec![
            encode_xdi_abx(LuaOpcode::LoadK, 0, 0),
            encode_xdi_abc(LuaOpcode::Return, 0, 2, 0),
            encode_xdi_abc(LuaOpcode::Return, 0, 1, 0),
        ];
        let mut chunk = build_xdi_chunk(&proto);
        let mut scratch = DecompileScratch::with_limits(DecompileConfig::DEFAULT.limits());
        let res = decompile_luac(&mut chunk, &mut scratch, &DecompileConfig::DEFAULT)
            .expect("decompile string boundary len");
        let expected_str = core::str::from_utf8(&payload).unwrap();
        assert!(res.source().contains(expected_str));
    }
}

#[test]
fn luac_limits_instructions_constants_upvalues_locvars_and_bytes_transitions() {
    let custom_limits = LuacLimits::DEFAULT
        .try_with_proto_depth_max(4)
        .unwrap()
        .with_protos_count_max(4)
        .with_instructions_count_max(5)
        .with_constants_count_max(3)
        .try_with_upvalues_count_max(2)
        .unwrap()
        .with_loc_vars_count_max(2)
        .with_transpile_bytes_max(1024);

    // 1. instructions_count_max: 0, MAX - 1 (4), MAX (5), MAX + 1 (6)
    for count in [0_u32, 4, 5, 6] {
        let mut proto = TestProto::new();
        proto.code = vec![encode_xdi_abc(LuaOpcode::Return, 0, 1, 0); crate::u32_to_usize(count)];
        let chunk = build_xdi_chunk(&proto);
        let mut out = crate::FixedVec::with_capacity(custom_limits.transpile_bytes_max());
        let res = transpile_to_lua53_into(&chunk, &mut out, custom_limits);
        if count <= 5 {
            assert!(res.is_ok(), "instructions count {count} should succeed");
        } else {
            assert!(matches!(
                res.unwrap_err(),
                LuacError::LimitExceeded {
                    kind: LuacLimitKind::InstructionsCount,
                    actual: 6,
                    max: 5,
                }
            ));
        }
    }

    // 2. constants_count_max: 0, MAX - 1 (2), MAX (3), MAX + 1 (4)
    for count in [0_u32, 2, 3, 4] {
        let mut proto = TestProto::new();
        proto.consts = vec![TestConst::Int(1); crate::u32_to_usize(count)];
        let chunk = build_xdi_chunk(&proto);
        let mut out = crate::FixedVec::with_capacity(custom_limits.transpile_bytes_max());
        let res = transpile_to_lua53_into(&chunk, &mut out, custom_limits);
        if count <= 3 {
            assert!(res.is_ok(), "constants count {count} should succeed");
        } else {
            assert!(matches!(
                res.unwrap_err(),
                LuacError::LimitExceeded {
                    kind: LuacLimitKind::ConstantsCount,
                    actual: 4,
                    max: 3,
                }
            ));
        }
    }

    // 3. transpile_bytes_max: exact_len - 1, exact_len, exact_len + 1
    let mut base_proto = TestProto::new();
    base_proto.code = vec![encode_xdi_abc(LuaOpcode::Return, 0, 1, 0)];
    let base_chunk = build_xdi_chunk(&base_proto);
    let mut ref_out = crate::FixedVec::with_capacity(1024);
    transpile_to_lua53_into(&base_chunk, &mut ref_out, custom_limits).unwrap();
    let exact_bytes = ref_out.len();
    for max_bytes in [exact_bytes - 1, exact_bytes, exact_bytes + 1] {
        let lim = custom_limits.with_transpile_bytes_max(max_bytes);
        let mut out = crate::FixedVec::with_capacity(lim.transpile_bytes_max());
        let res = transpile_to_lua53_into(&base_chunk, &mut out, lim);
        if max_bytes >= exact_bytes {
            assert!(res.is_ok());
        } else {
            assert!(matches!(
                res.unwrap_err(),
                LuacError::LimitExceeded {
                    kind: LuacLimitKind::TranspileBytes,
                    ..
                }
            ));
        }
    }
}

#[test]
fn luac_limits_upvalues_and_locvars_transitions() {
    let custom_limits = LuacLimits::DEFAULT
        .try_with_upvalues_count_max(2)
        .unwrap()
        .with_loc_vars_count_max(2)
        .with_transpile_bytes_max(1024);

    // 3. upvalues_count_max: 0, MAX - 1 (1), MAX (2), MAX + 1 (3)
    for count in [0_u32, 1, 2, 3] {
        let mut proto = TestProto::new();
        proto.upvalues = vec![(1, 0); crate::u32_to_usize(count)];
        let chunk = build_xdi_chunk(&proto);
        let mut out = crate::FixedVec::with_capacity(custom_limits.transpile_bytes_max());
        let res = transpile_to_lua53_into(&chunk, &mut out, custom_limits);
        if count <= 2 {
            assert!(res.is_ok(), "upvalues count {count} should succeed");
        } else {
            assert!(matches!(
                res.unwrap_err(),
                LuacError::LimitExceeded {
                    kind: LuacLimitKind::UpvaluesCount,
                    actual: 3,
                    max: 2,
                }
            ));
        }
    }

    // 4. loc_vars_count_max: 0, MAX - 1 (1), MAX (2), MAX + 1 (3)
    for count in [0_u32, 1, 2, 3] {
        let mut proto = TestProto::new();
        proto.loc_vars = vec![(b"x".to_vec(), 0, 1); crate::u32_to_usize(count)];
        let chunk = build_xdi_chunk(&proto);
        let mut out = crate::FixedVec::with_capacity(custom_limits.transpile_bytes_max());
        let res = transpile_to_lua53_into(&chunk, &mut out, custom_limits);
        if count <= 2 {
            assert!(res.is_ok(), "loc_vars count {count} should succeed");
        } else {
            assert!(matches!(
                res.unwrap_err(),
                LuacError::LimitExceeded {
                    kind: LuacLimitKind::LocVarsCount,
                    actual: 3,
                    max: 2,
                }
            ));
        }
    }
}

#[test]
fn luac_limits_proto_depth_and_total_count_transitions() {
    let custom_limits = LuacLimits::DEFAULT
        .try_with_proto_depth_max(3)
        .unwrap()
        .with_protos_count_max(3)
        .with_instructions_count_max(16)
        .with_constants_count_max(16)
        .try_with_upvalues_count_max(4)
        .unwrap()
        .with_loc_vars_count_max(4)
        .with_transpile_bytes_max(2048);

    // Total protos count: 1 (root), 2 (MAX - 1), 3 (MAX), 4 (MAX + 1)
    for sibling_children in [0_u32, 1, 2, 3] {
        let mut root = TestProto::new();
        root.children = (0..sibling_children).map(|_| TestProto::new()).collect();
        let chunk = build_xdi_chunk(&root);
        let mut out = crate::FixedVec::with_capacity(custom_limits.transpile_bytes_max());
        let res = transpile_to_lua53_into(&chunk, &mut out, custom_limits);
        if sibling_children < 3 {
            assert!(res.is_ok());
        } else {
            assert!(matches!(
                res.unwrap_err(),
                LuacError::LimitExceeded {
                    kind: LuacLimitKind::ProtosCount,
                    actual: 4,
                    max: 3,
                }
            ));
        }
    }

    // Proto nesting depth: depth 1..=4 with protos_count_max = 10
    let depth_limits = custom_limits.with_protos_count_max(10);
    for depth in [1_u32, 2, 3, 4] {
        let mut curr = TestProto::new();
        for _ in 1..depth {
            let mut parent = TestProto::new();
            parent.children.push(curr);
            curr = parent;
        }
        let chunk = build_xdi_chunk(&curr);
        let mut out = crate::FixedVec::with_capacity(depth_limits.transpile_bytes_max());
        let res = transpile_to_lua53_into(&chunk, &mut out, depth_limits);
        if depth <= 3 {
            assert!(res.is_ok(), "depth {depth} should succeed");
        } else {
            assert!(matches!(
                res.unwrap_err(),
                LuacError::LimitExceeded {
                    kind: LuacLimitKind::ProtoDepth,
                    actual: 4,
                    max: 3,
                }
            ));
        }
    }
}

fn build_chunk_with_list_len(list_len: u8) -> Vec<u8> {
    let mut proto = TestProto::new();
    proto.consts = vec![
        TestConst::Str(b"Id".to_vec()),
        TestConst::Str(b"Items".to_vec()),
        TestConst::Int(1),
    ];
    let mut code = vec![
        encode_xdi_abc(LuaOpcode::NewTable, 0, 0, 0),
        encode_xdi_abc(LuaOpcode::NewTable, 1, 0, 0),
        encode_xdi_abc(LuaOpcode::SetTable, 1, 0x100, 0x100 | 2),
        encode_xdi_abc(LuaOpcode::NewTable, 2, 0, 0),
    ];
    if list_len > 0 {
        for off in 1..=list_len {
            code.push(encode_xdi_abx(LuaOpcode::LoadK, 2 + off, 2));
        }
        code.push(encode_xdi_abc(
            LuaOpcode::SetList,
            2,
            u16::from(list_len),
            1,
        ));
    }
    code.extend_from_slice(&[
        encode_xdi_abc(LuaOpcode::SetTable, 1, 0x100 | 1, 2),
        encode_xdi_abc(LuaOpcode::SetTable, 0, 0x100 | 2, 1),
        encode_xdi_abc(LuaOpcode::NewTable, 1, 0, 0),
        encode_xdi_abc(LuaOpcode::NewTable, 2, 0, 0),
        encode_xdi_abc(LuaOpcode::NewTable, 3, 0, 0),
        encode_xdi_abc(LuaOpcode::NewTable, 4, 0, 0),
        encode_xdi_abx(LuaOpcode::Closure, 5, 0),
    ]);
    proto.code = code;
    let mut child = TestProto::new();
    child.code = vec![encode_xdi_abc(LuaOpcode::Return, 0, 1, 0)];
    proto.children.push(child);
    build_xdi_chunk(&proto)
}

#[test]
fn datacfg_limits_tables_list_items_and_map_entries_transitions() {
    // 1. List items limit = 3: test 0, 2 (MAX - 1), 3 (MAX), 4 (MAX + 1)
    let list_limits = DataCfgLimits::DEFAULT
        .with_tables_count_max(16)
        .with_list_items_max(3)
        .with_map_entries_max(16);
    let mut scratch = DataCfgScratch::with_limits(list_limits);
    for list_len in [0_u8, 2, 3, 4] {
        let mut chunk = build_chunk_with_list_len(list_len);
        let res = parse_luatable_datacfg::<u32, BoundaryRow>(&mut chunk, &mut scratch);
        if list_len <= 3 {
            let table = res.expect("list_len within max");
            assert_eq!(
                crate::bounded_len_u32(table.get(&1).unwrap().items.len()),
                u32::from(list_len)
            );
        } else {
            assert!(matches!(
                res.unwrap_err(),
                LuacError::LimitExceeded {
                    kind: LuacLimitKind::DataCfgListItems,
                    ..
                }
            ));
        }
    }

    // 2. Tables count limit: chunk allocates 6 tables (R0, row R1, items R2, most R1, default R2, lang R3)
    for max_tables in [5_u32, 6, 7] {
        let lim = DataCfgLimits::DEFAULT.with_tables_count_max(max_tables);
        let mut tbl_scratch = DataCfgScratch::with_limits(lim);
        let mut chunk = build_chunk_with_list_len(1);
        let res = parse_luatable_datacfg::<u32, BoundaryRow>(&mut chunk, &mut tbl_scratch);
        if max_tables >= 6 {
            assert!(res.is_ok());
        } else {
            assert!(matches!(
                res.unwrap_err(),
                LuacError::LimitExceeded {
                    kind: LuacLimitKind::DataCfgTablesCount,
                    ..
                }
            ));
        }
    }

    // 3. Map entries limit: chunk has 3 map entries (Id, Items, and R0[1] = row)
    for max_entries in [2_u32, 3, 4] {
        let lim = DataCfgLimits::DEFAULT.with_map_entries_max(max_entries);
        let mut map_scratch = DataCfgScratch::with_limits(lim);
        let mut chunk = build_chunk_with_list_len(1);
        let res = parse_luatable_datacfg::<u32, BoundaryRow>(&mut chunk, &mut map_scratch);
        if max_entries >= 3 {
            assert!(res.is_ok());
        } else {
            assert!(matches!(
                res.unwrap_err(),
                LuacError::LimitExceeded {
                    kind: LuacLimitKind::DataCfgMapEntries,
                    ..
                }
            ));
        }
    }
}
