use crate::luac::crypto::decrypt_luac_in_place;
use crate::luac::opcode::LuaOpcode;
use crate::luac::tests::{
    TestConst, TestProto, build_xdi_chunk, encode_xdi_abc, encode_xdi_abx, encode_xdi_ax,
    encrypt_xdi_in_place,
};
use crate::luac::types::CipherState;
use crate::luac::{
    DataCfgLimits, DataCfgRow, DataCfgScratch, DataTable, DecompileConfig, DecompileScratch,
    DecompiledChunkKind, EvalContext, EvalVal, LuacError, decompile_luac, parse_luatable_datacfg,
};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Default)]
struct SpellRow {
    id: u32,
    name: Box<str>,
    damage: u16,
    tags: Box<[u16]>,
    is_passive: bool,
}

impl DataCfgRow for SpellRow {
    const TABLE_NAME: &'static str = "config_spell";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Damage" => self.damage = ctx.decode_field(val, Self::TABLE_NAME, "Damage")?,
            b"Tags" => self.tags = ctx.decode_field(val, Self::TABLE_NAME, "Tags")?,
            b"IsPassive" => {
                self.is_passive = ctx.decode_field(val, Self::TABLE_NAME, "IsPassive")?;
            }
            other => return Err(ctx.unknown_field(Self::TABLE_NAME, other)),
        }
        Ok(())
    }
}

#[test]
fn cipher_round_trip_and_chunk_boundaries() {
    for len in [0_u32, 1, 2, 3, 4, 5, 23, 24, 25, 26, 27, 48, 49, 50, 100] {
        let mut orig: Vec<u8> = (0..len).map(|i| ((i * 37 + 13) & 0xFF) as u8).collect();
        if len >= 4 {
            orig[0..4].copy_from_slice(b"\x01XDI");
        }
        let mut encrypted = orig.clone();
        encrypt_xdi_in_place(&mut encrypted);
        if len >= 4 {
            assert_ne!(&encrypted[2..], &orig[2..]);
        }
        let state = decrypt_luac_in_place(&mut encrypted);
        assert_eq!(state, CipherState::DecryptedFromCipher);
        assert_eq!(encrypted, orig);

        if len >= 4 {
            let state_again = decrypt_luac_in_place(&mut encrypted);
            assert_eq!(state_again, CipherState::PlainXdi);
            assert_eq!(encrypted, orig);
        }
    }
}

#[test]
fn decompile_encrypted_and_plain_xdi_scripts() {
    let mut proto = TestProto::new();
    proto.consts = vec![TestConst::Int(42)];
    proto.code = vec![
        encode_xdi_abx(LuaOpcode::LoadK, 0, 0),
        encode_xdi_abc(LuaOpcode::Return, 0, 2, 0),
        encode_xdi_abc(LuaOpcode::Return, 0, 1, 0),
    ];
    let plain_xdi = build_xdi_chunk(&proto);
    let mut encrypted = plain_xdi.clone();
    encrypt_xdi_in_place(&mut encrypted);

    let mut scratch = DecompileScratch::with_limits(DecompileConfig::DEFAULT.limits());
    let res = decompile_luac(&mut encrypted, &mut scratch, &DecompileConfig::DEFAULT)
        .expect("decompile encrypted XDI");
    assert_eq!(res.kind(), DecompiledChunkKind::Source);
    assert!(res.source().contains("42"));
    assert_eq!(encrypted, plain_xdi);

    let res_plain = decompile_luac(&mut encrypted, &mut scratch, &DecompileConfig::DEFAULT)
        .expect("decompile plain XDI");
    assert_eq!(res_plain.source(), res.source());
}

#[test]
fn decompile_locals_upvalues_loops_conditionals_and_closures() {
    let mut child = TestProto::new();
    child.num_params = 1;
    child.is_vararg = 0;
    child.consts = vec![TestConst::Int(10)];
    child.loc_vars = vec![(b"x".to_vec(), 0, 3)];
    child.code = vec![
        encode_xdi_abc(LuaOpcode::Add, 1, 0, 0x100),
        encode_xdi_abc(LuaOpcode::Return, 1, 2, 0),
        encode_xdi_abc(LuaOpcode::Return, 0, 1, 0),
    ];

    let mut root = TestProto::new();
    root.upvalue_names = vec![b"_ENV".to_vec()];
    root.consts = vec![
        TestConst::Str(b"global_fn".to_vec()),
        TestConst::Int(1),
        TestConst::Int(5),
    ];
    root.loc_vars = vec![
        (b"acc".to_vec(), 1, 9),
        (b"(for index)".to_vec(), 4, 8),
        (b"(for limit)".to_vec(), 4, 8),
        (b"(for step)".to_vec(), 4, 8),
        (b"idx".to_vec(), 5, 7),
    ];
    root.code = vec![
        encode_xdi_abx(LuaOpcode::LoadK, 0, 1), // pc 0: acc = 1
        encode_xdi_abx(LuaOpcode::LoadK, 1, 1), // pc 1: for init = 1
        encode_xdi_abx(LuaOpcode::LoadK, 2, 2), // pc 2: for limit = 5
        encode_xdi_abx(LuaOpcode::LoadK, 3, 1), // pc 3: for step = 1
        encode_xdi_abx(LuaOpcode::ForPrep, 1, 131_071 + 2), // pc 4 -> pc 7
        encode_xdi_abc(LuaOpcode::Add, 0, 0, 4), // pc 5: acc = (acc + idx)
        encode_xdi_abx(LuaOpcode::Closure, 5, 0), // pc 6: r5 = function(x) ...
        encode_xdi_abx(LuaOpcode::ForLoop, 1, 131_071 - 3), // pc 7 -> pc 5
        encode_xdi_abc(LuaOpcode::SetTabUp, 0, 0x100, 5), // pc 8: global_fn = r5
        encode_xdi_abc(LuaOpcode::Return, 0, 2, 0), // pc 9: return acc
    ];
    root.children.push(child);

    let mut chunk = build_xdi_chunk(&root);
    let cfg = DecompileConfig::DEFAULT
        .with_strict_mode(true)
        .with_include_comments(true);
    let mut scratch = DecompileScratch::with_limits(cfg.limits());
    let res = decompile_luac(&mut chunk, &mut scratch, &cfg).expect("decompile structured lua");
    assert_eq!(res.kind(), DecompiledChunkKind::Source);
    let src = res.source();
    assert!(src.contains("-- proto 0"));
    assert!(src.contains("local acc = 1"));
    assert!(src.contains("for idx = 1, 5 do"));
    assert!(src.contains("acc = (acc + idx)"));
    assert!(src.contains("function(x)"));
    assert!(src.contains("return (x + 10)"));
    assert!(src.contains("global_fn = function(x)"));
    assert!(src.contains("return acc"));
}

#[test]
fn decompile_conditionals_if_else_while_and_comparisons() {
    let mut proto = TestProto::new();
    proto.num_params = 2;
    proto.loc_vars = vec![(b"a".to_vec(), 0, 20), (b"b".to_vec(), 0, 20)];
    proto.consts = vec![TestConst::Int(0), TestConst::Int(1), TestConst::Int(2)];
    proto.code = vec![
        // pc 0..5: while a < b do a = (a + 1) end
        encode_xdi_abc(LuaOpcode::Lt, 0, 0, 1), // pc 0: a < b
        encode_xdi_abx(LuaOpcode::Jmp, 0, 131_071 + 2), // pc 1 -> pc 4
        encode_xdi_abc(LuaOpcode::Add, 0, 0, 0x100 | 1), // pc 2: a = (a + 1)
        encode_xdi_abx(LuaOpcode::Jmp, 0, 131_071 - 4), // pc 3 -> pc 0
        // pc 4..9: if a == b then b = 1 else b = 2 end
        encode_xdi_abc(LuaOpcode::Eq, 0, 0, 1), // pc 4: a == b
        encode_xdi_abx(LuaOpcode::Jmp, 0, 131_071 + 2), // pc 5 -> pc 8 (else)
        encode_xdi_abx(LuaOpcode::LoadK, 1, 1), // pc 6: b = 1
        encode_xdi_abx(LuaOpcode::Jmp, 0, 131_071 + 1), // pc 7 -> pc 9 (end)
        encode_xdi_abx(LuaOpcode::LoadK, 1, 2), // pc 8: b = 2
        // pc 9..15: Le (>), Test (not a), TestSet
        encode_xdi_abc(LuaOpcode::Le, 1, 0, 0x100), // pc 9: a > 0
        encode_xdi_abx(LuaOpcode::Jmp, 0, 131_071 + 4), // pc 10 -> pc 15
        encode_xdi_abc(LuaOpcode::Test, 0, 0, 1),   // pc 11: not a
        encode_xdi_abx(LuaOpcode::Jmp, 0, 131_071 + 2), // pc 12 -> pc 15
        encode_xdi_abc(LuaOpcode::TestSet, 2, 1, 0), // pc 13: b
        encode_xdi_abx(LuaOpcode::Jmp, 0, 131_071 + 1), // pc 14 -> pc 16
        encode_xdi_abx(LuaOpcode::LoadK, 0, 0),     // pc 15: a = 0
        encode_xdi_abc(LuaOpcode::Return, 0, 3, 0), // pc 16: return a, b
    ];

    let mut chunk = build_xdi_chunk(&proto);
    let cfg = DecompileConfig::DEFAULT.with_strict_mode(true);
    let mut scratch = DecompileScratch::with_limits(cfg.limits());
    let res = decompile_luac(&mut chunk, &mut scratch, &cfg).expect("decompile conditionals");
    let src = res.source();
    assert!(src.contains("while a < b do"));
    assert!(src.contains("if a == b then"));
    assert!(src.contains("else"));
    assert!(src.contains("if a > 0 then"));
    assert!(src.contains("if not a then"));
    assert!(src.contains("return a, b"));
}

#[test]
fn decompile_generic_for_in_calls_self_tailcall_and_vararg() {
    let mut proto = TestProto::new();
    proto.upvalue_names = vec![b"_ENV".to_vec()];
    proto.consts = vec![
        TestConst::Str(b"pairs".to_vec()),
        TestConst::Str(b"step".to_vec()),
        TestConst::Str(b"unpack".to_vec()),
    ];
    proto.loc_vars = vec![
        (b"tbl".to_vec(), 0, 17),
        (b"(for generator)".to_vec(), 3, 9),
        (b"(for state)".to_vec(), 3, 9),
        (b"(for control)".to_vec(), 3, 9),
        (b"k".to_vec(), 4, 8),
        (b"v".to_vec(), 4, 8),
        (b"x".to_vec(), 12, 17),
        (b"y".to_vec(), 12, 17),
    ];
    proto.code = vec![
        encode_xdi_abc(LuaOpcode::GetTabUp, 1, 0, 0x100), // pc 0: r1 = pairs
        encode_xdi_abc(LuaOpcode::Move, 2, 0, 0),         // pc 1: r2 = tbl
        encode_xdi_abc(LuaOpcode::Call, 1, 2, 4),         // pc 2: pairs(tbl) for TForCall
        encode_xdi_abx(LuaOpcode::Jmp, 0, 131_071 + 3),   // pc 3 -> pc 7 (TForCall)
        encode_xdi_abc(LuaOpcode::SelfOp, 6, 5, 0x100 | 1), // pc 4: r6 = v:step, r7 = v
        encode_xdi_abc(LuaOpcode::Move, 8, 4, 0),         // pc 5: r8 = k
        encode_xdi_abc(LuaOpcode::Call, 6, 3, 1),         // pc 6: v:step(k)
        encode_xdi_abc(LuaOpcode::TForCall, 1, 0, 2),     // pc 7
        encode_xdi_abx(LuaOpcode::TForLoop, 3, 131_071 - 5), // pc 8 -> pc 4
        encode_xdi_abc(LuaOpcode::GetTabUp, 1, 0, 0x100 | 2), // pc 9: r1 = unpack
        encode_xdi_abc(LuaOpcode::VarArg, 2, 0, 0),       // pc 10: r2 = ... (open)
        encode_xdi_abc(LuaOpcode::Call, 1, 0, 3),         // pc 11: local x, y = unpack(...)
        encode_xdi_abc(LuaOpcode::GetTabUp, 3, 0, 0x100 | 2), // pc 12: r3 = unpack
        encode_xdi_abc(LuaOpcode::Move, 4, 1, 0),         // pc 13: r4 = x
        encode_xdi_abc(LuaOpcode::TailCall, 3, 2, 0),     // pc 14: return unpack(x)
        encode_xdi_abc(LuaOpcode::Return, 3, 0, 0),       // pc 15
    ];

    let mut chunk = build_xdi_chunk(&proto);
    let cfg = DecompileConfig::DEFAULT.with_strict_mode(true);
    let mut scratch = DecompileScratch::with_limits(cfg.limits());
    let res = decompile_luac(&mut chunk, &mut scratch, &cfg).expect("decompile for-in & calls");
    let src = res.source();
    assert!(src.contains("for k, v in pairs(tbl) do"));
    assert!(src.contains("v:step(k)"));
    assert!(src.contains("local x = unpack(...)"));
    assert!(src.contains("local y = select(2, unpack(...))"));
    assert!(src.contains("return unpack(x)"));
}

#[test]
fn decompile_tables_loads_concat_and_all_operators() {
    let mut proto = TestProto::new();
    proto.upvalues = vec![(1, 0), (1, 1)];
    proto.upvalue_names = vec![b"_ENV".to_vec(), b"up_counter".to_vec()];
    proto.loc_vars = vec![(b"t".to_vec(), 1, 30)];
    proto.consts = vec![
        TestConst::Str(b"valid_key".to_vec()),
        TestConst::Str(b"bad-key".to_vec()),
        TestConst::Float(1.0),
        TestConst::Float(1.25),
        TestConst::Str(b"hi\n\t\"\\\x07".to_vec()),
    ];
    proto.code = vec![
        encode_xdi_abc(LuaOpcode::NewTable, 0, 0, 0),
        encode_xdi_abx(LuaOpcode::LoadKx, 1, 0),
        encode_xdi_ax(LuaOpcode::ExtraArg, 2),
        encode_xdi_abx(LuaOpcode::LoadK, 2, 3),
        encode_xdi_abc(LuaOpcode::SetList, 0, 2, 0),
        encode_xdi_ax(LuaOpcode::ExtraArg, 1),
        encode_xdi_abc(LuaOpcode::SetTable, 0, 0x100, 0x100 | 4),
        encode_xdi_abc(LuaOpcode::SetTable, 0, 0x100 | 1, 1),
        encode_xdi_abc(LuaOpcode::GetTable, 3, 0, 0x100),
        encode_xdi_abc(LuaOpcode::GetTable, 4, 0, 0x100 | 1),
        encode_xdi_abc(LuaOpcode::GetUpval, 5, 1, 0),
        encode_xdi_abc(LuaOpcode::Sub, 5, 5, 1),
        encode_xdi_abc(LuaOpcode::Mul, 5, 5, 2),
        encode_xdi_abc(LuaOpcode::Mod, 5, 5, 1),
        encode_xdi_abc(LuaOpcode::Pow, 5, 5, 1),
        encode_xdi_abc(LuaOpcode::Div, 5, 5, 2),
        encode_xdi_abc(LuaOpcode::IDiv, 5, 5, 1),
        encode_xdi_abc(LuaOpcode::BAnd, 5, 5, 1),
        encode_xdi_abc(LuaOpcode::BOr, 5, 5, 1),
        encode_xdi_abc(LuaOpcode::BXor, 5, 5, 1),
        encode_xdi_abc(LuaOpcode::Shl, 5, 5, 1),
        encode_xdi_abc(LuaOpcode::Shr, 5, 5, 1),
        encode_xdi_abc(LuaOpcode::Unm, 6, 5, 0),
        encode_xdi_abc(LuaOpcode::BNot, 6, 6, 0),
        encode_xdi_abc(LuaOpcode::Not, 6, 6, 0),
        encode_xdi_abc(LuaOpcode::Len, 6, 0, 0),
        encode_xdi_abc(LuaOpcode::Concat, 7, 3, 4),
        encode_xdi_abc(LuaOpcode::LoadBool, 8, 1, 1),
        encode_xdi_abc(LuaOpcode::LoadBool, 8, 0, 0),
        encode_xdi_abc(LuaOpcode::LoadNil, 9, 1, 0),
        encode_xdi_abc(LuaOpcode::SetUpval, 5, 1, 0),
        encode_xdi_abc(LuaOpcode::Return, 5, 4, 0),
    ];

    let mut chunk = build_xdi_chunk(&proto);
    let cfg = DecompileConfig::DEFAULT.with_strict_mode(true);
    let mut scratch = DecompileScratch::with_limits(cfg.limits());
    let res = decompile_luac(&mut chunk, &mut scratch, &cfg).expect("decompile all ops");
    let src = res.source();
    assert!(src.contains("local t = {}"));
    assert!(src.contains("table.insert(t, 1.0)"));
    assert!(src.contains("table.insert(t, 1.25)"));
    assert!(src.contains("t.valid_key = \"hi\\n\\t\\\"\\\\\\x07\""));
    assert!(src.contains("t[\"bad-key\"] = 1.0"));
    assert!(src.contains("up_counter = "));
    assert!(src.contains(" .. "));
}

fn build_spell_datacfg_chunk() -> Vec<u8> {
    let mut proto = TestProto::new();
    proto.consts = vec![
        TestConst::Str(b"Id".to_vec()),        // 0
        TestConst::Str(b"Name".to_vec()),      // 1
        TestConst::Str(b"Damage".to_vec()),    // 2
        TestConst::Str(b"Tags".to_vec()),      // 3
        TestConst::Str(b"IsPassive".to_vec()), // 4
        TestConst::Int(1001),                  // 5
        TestConst::Str(b"Fireball".to_vec()),  // 6
        TestConst::Int(50),                    // 7
        TestConst::Int(1),                     // 8
        TestConst::Int(2),                     // 9
        TestConst::Int(1002),                  // 10
        TestConst::Str(b"Icebolt".to_vec()),   // 11
        TestConst::Str(Vec::new()),            // 12 ("")
        TestConst::Int(10),                    // 13
        TestConst::Int(9),                     // 14
        TestConst::Int(0),                     // 15
        TestConst::Str(b"Default".to_vec()),   // 16
    ];
    proto.code = vec![
        // R[0] = {}
        encode_xdi_abc(LuaOpcode::NewTable, 0, 0, 0),
        // Row 1001 in R[1]
        encode_xdi_abc(LuaOpcode::NewTable, 1, 0, 0),
        encode_xdi_abc(LuaOpcode::SetTable, 1, 0x100, 0x100 | 5),
        encode_xdi_abc(LuaOpcode::SetTable, 1, 0x100 | 1, 0x100 | 6),
        encode_xdi_abc(LuaOpcode::SetTable, 1, 0x100 | 2, 0x100 | 7),
        encode_xdi_abc(LuaOpcode::NewTable, 2, 0, 0),
        encode_xdi_abx(LuaOpcode::LoadK, 3, 8),
        encode_xdi_abx(LuaOpcode::LoadK, 4, 9),
        encode_xdi_abc(LuaOpcode::SetList, 2, 2, 1),
        encode_xdi_abc(LuaOpcode::SetTable, 1, 0x100 | 3, 2),
        encode_xdi_abc(LuaOpcode::LoadBool, 2, 0, 0),
        encode_xdi_abc(LuaOpcode::SetTable, 1, 0x100 | 4, 2),
        encode_xdi_abc(LuaOpcode::SetTable, 0, 0x100 | 5, 1),
        // Row 1002 in R[1] (Damage, Tags, IsPassive omitted -> inherited from R[1] most_value)
        encode_xdi_abc(LuaOpcode::NewTable, 1, 0, 0),
        encode_xdi_abc(LuaOpcode::SetTable, 1, 0x100, 0x100 | 10),
        encode_xdi_abc(LuaOpcode::SetTable, 1, 0x100 | 1, 0x100 | 11),
        encode_xdi_abc(LuaOpcode::SetTable, 0, 0x100 | 10, 1),
        // R[1] = most_value
        encode_xdi_abc(LuaOpcode::NewTable, 1, 0, 0),
        encode_xdi_abc(LuaOpcode::SetTable, 1, 0x100, 0x100 | 5),
        encode_xdi_abc(LuaOpcode::SetTable, 1, 0x100 | 1, 0x100 | 12),
        encode_xdi_abc(LuaOpcode::SetTable, 1, 0x100 | 2, 0x100 | 13),
        encode_xdi_abc(LuaOpcode::NewTable, 2, 0, 0),
        encode_xdi_abx(LuaOpcode::LoadK, 3, 14),
        encode_xdi_abc(LuaOpcode::SetList, 2, 1, 1),
        encode_xdi_abc(LuaOpcode::SetTable, 1, 0x100 | 3, 2),
        encode_xdi_abc(LuaOpcode::LoadBool, 2, 1, 0),
        encode_xdi_abc(LuaOpcode::SetTable, 1, 0x100 | 4, 2),
        // R[2] = default_value
        encode_xdi_abc(LuaOpcode::NewTable, 2, 0, 0),
        encode_xdi_abc(LuaOpcode::SetTable, 2, 0x100, 0x100 | 15),
        encode_xdi_abc(LuaOpcode::SetTable, 2, 0x100 | 1, 0x100 | 16),
        encode_xdi_abc(LuaOpcode::SetTable, 2, 0x100 | 2, 0x100 | 15),
        encode_xdi_abc(LuaOpcode::NewTable, 3, 0, 0),
        encode_xdi_abc(LuaOpcode::SetTable, 2, 0x100 | 3, 3),
        encode_xdi_abc(LuaOpcode::LoadBool, 3, 0, 0),
        encode_xdi_abc(LuaOpcode::SetTable, 2, 0x100 | 4, 3),
        // R[3] = language_key
        encode_xdi_abc(LuaOpcode::NewTable, 3, 0, 0),
        encode_xdi_abc(LuaOpcode::LoadBool, 4, 1, 0),
        encode_xdi_abc(LuaOpcode::SetTable, 3, 0x100 | 1, 4),
        // R[4] = mode_readonly + CLOSURE
        encode_xdi_abc(LuaOpcode::NewTable, 4, 0, 0),
        encode_xdi_abx(LuaOpcode::Closure, 5, 0),
        encode_xdi_abc(LuaOpcode::Return, 0, 1, 0),
    ];
    let mut child = TestProto::new();
    child.code = vec![encode_xdi_abc(LuaOpcode::Return, 0, 1, 0)];
    proto.children.push(child);
    build_xdi_chunk(&proto)
}

#[test]
fn datacfg_typed_table_most_value_inheritance_and_lookup() {
    let mut chunk = build_spell_datacfg_chunk();
    encrypt_xdi_in_place(&mut chunk);

    let mut scratch = DataCfgScratch::with_limits(DataCfgLimits::DEFAULT);
    let table: DataTable<u32, SpellRow> =
        parse_luatable_datacfg(&mut chunk, &mut scratch).expect("parse spell table");

    assert_eq!(table.len(), 2);
    assert!(table.rows().contains_key(&1001));
    assert!(table.rows().contains_key(&1002));
    assert_eq!(table.language_keys(), &[Box::<str>::from("Name")]);

    let r1001 = table.get(&1001).expect("row 1001");
    assert_eq!(r1001.id, 1001);
    assert_eq!(&*r1001.name, "Fireball");
    assert_eq!(r1001.damage, 50);
    assert_eq!(&*r1001.tags, &[1, 2]);
    assert!(!r1001.is_passive);

    let r1002 = table.get(&1002).expect("row 1002");
    assert_eq!(r1002.id, 1002);
    assert_eq!(&*r1002.name, "Icebolt");
    assert_eq!(r1002.damage, 10);
    assert_eq!(&*r1002.tags, &[9]);
    assert!(r1002.is_passive);

    assert_eq!(&*table.default_row().name, "Default");
    assert_eq!(table.most_row().damage, 10);
}

#[test]
fn datacfg_multi_batch_setlist_and_loadkx() {
    let mut proto = TestProto::new();
    proto.consts = vec![
        TestConst::Str(b"Id".to_vec()),   // 0
        TestConst::Str(b"Tags".to_vec()), // 1
        TestConst::Int(1),                // 2
        TestConst::Int(77),               // 3
    ];
    let mut code = vec![
        encode_xdi_abc(LuaOpcode::NewTable, 0, 0, 0),
        encode_xdi_abc(LuaOpcode::NewTable, 1, 0, 0),
        encode_xdi_abc(LuaOpcode::SetTable, 1, 0x100, 0x100 | 2),
        encode_xdi_abc(LuaOpcode::NewTable, 2, 0, 0),
    ];
    for off in 1..=50_u8 {
        code.push(encode_xdi_abx(LuaOpcode::LoadK, 2 + off, 3));
    }
    code.push(encode_xdi_abc(LuaOpcode::SetList, 2, 50, 1));
    code.push(encode_xdi_abx(LuaOpcode::LoadKx, 3, 0));
    code.push(encode_xdi_ax(LuaOpcode::ExtraArg, 3));
    code.push(encode_xdi_abc(LuaOpcode::SetList, 2, 1, 0));
    code.push(encode_xdi_ax(LuaOpcode::ExtraArg, 2));
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

    let mut chunk = build_xdi_chunk(&proto);
    let mut scratch = DataCfgScratch::with_limits(DataCfgLimits::DEFAULT);
    let table: DataTable<u32, SpellRow> =
        parse_luatable_datacfg(&mut chunk, &mut scratch).expect("parse multi-batch setlist");
    let row = table.get(&1).expect("row 1");
    assert_eq!(row.tags.len(), 51);
    assert!(row.tags.iter().all(|&x| x == 77));
}

#[test]
#[ignore = "requires TLI_GAME_DIR and TLI_AES_KEY"]
fn tli_game_dir_all_343_datacfg_tables_and_scripts() {
    let game_dir = std::env::var("TLI_GAME_DIR").expect("TLI_GAME_DIR must be set");
    let key_hex = std::env::var("TLI_AES_KEY").expect("TLI_AES_KEY must be set");
    let key = pak::AesKey::try_from_hex(&key_hex).expect("valid TLI_AES_KEY");
    let vfs = pak_vfs::PakVfs::open_tli_game_dir(Path::new(&game_dir), Some(key))
        .expect("open TLI_GAME_DIR PakVfs");

    let mut pak_scratch = pak::ReadScratch::with_limits(pak::ScratchLimits::DEFAULT);
    let mut cfg_scratch = DataCfgScratch::with_limits(DataCfgLimits::DEFAULT);
    let mut dec_scratch = DecompileScratch::with_limits(DecompileConfig::DEFAULT.limits());
    let mut buf = Vec::new();
    let mut path_buf = Vec::new();

    let mut datacfg_parsed = 0_u32;
    let mut total_rows = 0_u32;
    let mut scripts_decompiled = 0_u32;

    for fid in vfs.iter_files() {
        path_buf.clear();
        vfs.write_file_path(fid, &mut path_buf);
        let path_str = core::str::from_utf8(&path_buf).expect("valid vfs path");
        if !path_str.ends_with(".out") {
            continue;
        }
        let raw_len = u32::try_from(vfs.file_meta(fid).uncompressed_bytes())
            .expect("uncompressed .out size fits in u32");
        let len = crate::u32_to_usize(raw_len);
        buf.resize(len, 0);
        vfs.read_into(fid, &mut pak_scratch, &mut buf)
            .expect("read_into .out");

        if path_str.contains("DataCfg/config_") {
            let file_name = path_str.rsplit('/').next().expect("file_name");
            let stem = file_name.strip_suffix(".out").expect(".out suffix");
            let count = super::tables::verify_all_datacfg_by_stem(stem, &mut buf, &mut cfg_scratch)
                .expect("parse DataCfg table");
            total_rows += count;
            datacfg_parsed += 1;
        } else {
            let strict_cfg = DecompileConfig::DEFAULT.with_strict_mode(true);
            let res = decompile_luac(&mut buf, &mut dec_scratch, &strict_cfg)
                .expect("decompile Luac script in strict mode");
            assert_eq!(res.kind(), DecompiledChunkKind::Source);
            assert!(!res.source().is_empty());
            scripts_decompiled += 1;
        }
    }
    assert_eq!(datacfg_parsed, 343);
    assert_eq!(total_rows, 84_891);
    assert_eq!(scripts_decompiled, 3_318);
}
