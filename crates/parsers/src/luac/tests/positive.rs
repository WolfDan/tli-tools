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
        let mut orig: Vec<u8> = (0..len)
            .map(|i| u8::try_from((i * 37 + 13) & 0xFF).unwrap())
            .collect();
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
        } else if scripts_decompiled < 50 || path_str.ends_with("Basics/Structure/String.out") {
            let res = decompile_luac(&mut buf, &mut dec_scratch, &DecompileConfig::DEFAULT)
                .expect("decompile Luac script");
            assert!(!res.source().is_empty());
            scripts_decompiled += 1;
        }
    }
    assert_eq!(datacfg_parsed, 343);
    assert_eq!(total_rows, 84_891);
    assert!(scripts_decompiled >= 50);
}
