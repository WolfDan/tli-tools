//! Unit, boundary, and negative tests for `xtask` CLI parsing and `DataCfg` schema inference.

use crate::datacfg::domains::{
    classify_domain, infer_kv_field_names, to_pascal_case, to_snake_case,
};
use crate::datacfg::emit::{CodegenMode, write_or_check_file};
use crate::datacfg::schema::{
    ColType, IntBounds, RecordType, ScalarOrList, ScalarType, analyze_luatable, analyze_preload,
};
use crate::datacfg::vm::{Cursor, GEN_INSTRUCTIONS_MAX, RootProto, Val, eval_root_slice};
use crate::error::XtaskError;
use crate::parse_cli_args;
use std::collections::BTreeMap;
use std::fs;

fn encode_xdi_abc(xdi_op: u32, a: u32, b: u32, c: u32) -> u32 {
    assert!(xdi_op < 47);
    (xdi_op << 26) | ((b & 0x1FF) << 17) | ((c & 0x1FF) << 8) | (a & 0xFF)
}

fn encode_xdi_abx(xdi_op: u32, a: u32, bx: u32) -> u32 {
    assert!(xdi_op < 47);
    (xdi_op << 26) | ((bx & 0x3FFFF) << 8) | (a & 0xFF)
}

fn push_xdi_str(out: &mut Vec<u8>, bytes: &[u8]) {
    let len = u32::try_from(bytes.len()).expect("len") + 10;
    if len < 0xFF {
        out.push(u8::try_from(len).expect("u8"));
    } else {
        out.push(0xFF);
        out.extend_from_slice(&len.to_le_bytes());
    }
    out.extend_from_slice(bytes);
}

#[test]
fn cli_args_positive_and_negative_rejections() {
    assert_eq!(parse_cli_args(["codegen"]).unwrap(), CodegenMode::Write);
    assert_eq!(
        parse_cli_args(["codegen", "--check"]).unwrap(),
        CodegenMode::Check
    );

    let err_empty = parse_cli_args(core::iter::empty::<&str>()).unwrap_err();
    assert!(matches!(err_empty, XtaskError::Usage { .. }));

    let err_cmd = parse_cli_args(["invalid"]).unwrap_err();
    assert!(matches!(
        err_cmd,
        XtaskError::UnknownCommand { ref command } if &**command == "invalid"
    ));

    let err_flag = parse_cli_args(["codegen", "--unknown"]).unwrap_err();
    assert!(matches!(
        err_flag,
        XtaskError::UnknownFlag { ref flag } if &**flag == "--unknown"
    ));
}

#[test]
fn int_bounds_exact_integer_width_transitions() {
    assert_eq!(IntBounds::single(0).rust_type(), "u8");
    assert_eq!(IntBounds::single(255).rust_type(), "u8");
    assert_eq!(IntBounds::single(256).rust_type(), "u16");
    assert_eq!(IntBounds::single(65_535).rust_type(), "u16");
    assert_eq!(IntBounds::single(65_536).rust_type(), "u32");
    assert_eq!(IntBounds::single(4_294_967_295).rust_type(), "u32");
    assert_eq!(IntBounds::single(4_294_967_296).rust_type(), "u64");

    assert_eq!(
        IntBounds {
            min: -128,
            max: 127
        }
        .rust_type(),
        "i8"
    );
    assert_eq!(IntBounds { min: -129, max: 0 }.rust_type(), "i16");
    assert_eq!(IntBounds { min: -1, max: 128 }.rust_type(), "i16");
    assert_eq!(
        IntBounds {
            min: -32_768,
            max: 32_767
        }
        .rust_type(),
        "i16"
    );
    assert_eq!(
        IntBounds {
            min: -32_769,
            max: 0
        }
        .rust_type(),
        "i32"
    );
    assert_eq!(
        IntBounds {
            min: -2_147_483_648,
            max: 2_147_483_647
        }
        .rust_type(),
        "i32"
    );
    assert_eq!(
        IntBounds {
            min: -2_147_483_649,
            max: 0
        }
        .rust_type(),
        "i64"
    );
}

#[test]
fn schema_type_merging_and_incompatible_rejection() {
    let u8_ty = ScalarOrList::Scalar(ScalarType::Int(IntBounds::single(10)));
    let list_u16 = ScalarOrList::List {
        elem: ScalarType::Int(IntBounds::single(500)),
        depth: 1,
    };
    let merged = ScalarOrList::merge(u8_ty, list_u16).unwrap();
    assert_eq!(merged.rust_type(), "IntOrIntList<u16>");

    // StringRecord with divergent keys degrades to FxHashMap<Box<str>, V>
    let mut m1 = BTreeMap::new();
    m1.insert("Alpha".to_owned(), u8_ty);
    let mut m2 = BTreeMap::new();
    m2.insert("Beta".to_owned(), u8_ty);
    let col_merged = ColType::merge(
        ColType::Record(RecordType::StringRecord(m1)),
        ColType::Record(RecordType::StringRecord(m2)),
    )
    .unwrap();
    assert_eq!(
        col_merged.rust_type(None, "config_test").unwrap(),
        "FxHashMap<Box<str>, u8>"
    );

    // Incompatible scalar types rejected
    let err = ScalarType::merge(ScalarType::Bool, ScalarType::Str).unwrap_err();
    assert!(matches!(err, XtaskError::IncompatibleTypes { .. }));
}

#[test]
fn domains_and_naming_conventions() {
    assert_eq!(to_pascal_case("item_gold_base_id"), "ItemGoldBaseId");
    assert_eq!(to_snake_case("BaseID"), "base_id");
    assert_eq!(to_snake_case("Type"), "r#type");
    assert_eq!(classify_domain("config_preload_foo"), "preload");
    assert_eq!(classify_domain("config_s14_gameplay"), "season");
    assert_eq!(classify_domain("config_equip_base"), "equip");
    assert_eq!(classify_domain("config_unknown_xyz"), "system");

    let kv = infer_kv_field_names("config_shop", "Tab", false, false, None);
    assert_eq!(kv.key_name, "first_type");
    assert_eq!(kv.val_names, vec!["second_type"]);

    let tuple_kv = infer_kv_field_names("config_test", "Shape", false, false, Some(2));
    assert_eq!(tuple_kv.key_name, "part_index");
    assert_eq!(tuple_kv.val_names, vec!["offset_x", "offset_y"]);
}

#[test]
fn analyze_synthetic_luatable_and_preload_chunks() {
    // Build a synthetic preload list chunk: R0 = { 10, 300 }; Return R0
    // XDI op 4 = NEWTABLE(11), XDI op 6 = LOADK(1), XDI op 28 = SETLIST(43), XDI op 21 = RETURN(38)
    let mut preload_buf = Vec::new();
    preload_buf.extend_from_slice(b"\x01XDI\x11\r\n\x1a\n\x01");
    preload_buf.push(0); // source = None
    preload_buf.extend_from_slice(&[0u8; 11]); // meta
    let preload_code = [
        encode_xdi_abc(4, 0, 0, 0),  // NEWTABLE R0
        encode_xdi_abx(6, 1, 0),     // LOADK R1, K0 (10)
        encode_xdi_abx(6, 2, 1),     // LOADK R2, K1 (300)
        encode_xdi_abc(28, 0, 2, 1), // SETLIST R0, 2, 1
        encode_xdi_abc(21, 0, 2, 0), // RETURN R0, 2
    ];
    preload_buf.extend_from_slice(&5_u32.to_le_bytes());
    for w in preload_code {
        preload_buf.extend_from_slice(&w.to_le_bytes());
    }
    preload_buf.extend_from_slice(&2_u32.to_le_bytes());
    preload_buf.push(0x13);
    preload_buf.extend_from_slice(&10_i64.to_le_bytes());
    preload_buf.push(0x13);
    preload_buf.extend_from_slice(&300_i64.to_le_bytes());

    let p_spec = analyze_preload("config_preload_test".to_owned(), &mut preload_buf).unwrap();
    assert_eq!(p_spec.pascal, "ConfigPreloadTest");

    // Build a synthetic LuaTable chunk: R0 (main), R1 (most), R2 (def)
    // XDI op 2 = SETTABLE(10)
    let mut lt_buf = Vec::new();
    lt_buf.extend_from_slice(b"\x01XDI\x11\r\n\x1a\n\x01");
    lt_buf.push(0);
    lt_buf.extend_from_slice(&[0u8; 11]);
    let lt_code = [
        encode_xdi_abc(4, 0, 0, 0),             // NEWTABLE R0 (main)
        encode_xdi_abc(4, 1, 0, 0),             // NEWTABLE R1 (most)
        encode_xdi_abc(4, 2, 0, 0),             // NEWTABLE R2 (def)
        encode_xdi_abc(2, 1, 0x100, 0x100 | 1), // R1["Id"] = 42
        encode_xdi_abc(21, 0, 2, 0),            // RETURN
    ];
    lt_buf.extend_from_slice(&5_u32.to_le_bytes());
    for w in lt_code {
        lt_buf.extend_from_slice(&w.to_le_bytes());
    }
    lt_buf.extend_from_slice(&2_u32.to_le_bytes());
    lt_buf.push(0x04);
    push_xdi_str(&mut lt_buf, b"Id");
    lt_buf.push(0x13);
    lt_buf.extend_from_slice(&42_i64.to_le_bytes());

    let lt_spec = analyze_luatable("config_item_test".to_owned(), &mut lt_buf).unwrap();
    assert_eq!(lt_spec.pascal, "ItemTest");
    assert_eq!(
        lt_spec
            .cols
            .get("Id")
            .unwrap()
            .rust_type(None, "s")
            .unwrap(),
        "u8"
    );
}

#[test]
fn reject_malformed_bytecode_and_step_limit_exceeded() {
    let bad = b"BAD_MAGIC_BYTES";
    let mut cur = Cursor::new(bad, "bad");
    let err = crate::datacfg::vm::parse_root_proto(&mut cur).unwrap_err();
    assert!(matches!(err, XtaskError::MalformedBytecode { .. }));

    // StepLimitExceeded when code has GEN_INSTRUCTIONS_MAX + 1 NOP-like instructions without Return
    let proto = RootProto {
        code: vec![encode_xdi_abc(12, 0, 0, 0); crate::u32_to_usize(GEN_INSTRUCTIONS_MAX + 1)]
            .into_boxed_slice(),
        consts: Box::new([]),
    };
    let mut regs: [Val; 256] = core::array::from_fn(|_| Val::Nil);
    let mut tables = Vec::new();
    let step_err = eval_root_slice("loop_test", &proto, &mut regs, &mut tables).unwrap_err();
    assert!(matches!(
        step_err,
        XtaskError::StepLimitExceeded {
            max: GEN_INSTRUCTIONS_MAX,
            ..
        }
    ));
}

#[test]
fn write_or_check_file_detects_drift() {
    let tmp_dir = std::env::temp_dir().join(format!("tli_xtask_test_{}", std::process::id()));
    fs::create_dir_all(&tmp_dir).unwrap();
    let file_path = tmp_dir.join("sample.rs");

    // Check fails when file does not exist
    let missing_err =
        write_or_check_file(&file_path, "pub mod a;\n", CodegenMode::Check).unwrap_err();
    assert!(matches!(missing_err, XtaskError::GeneratedFileDrift { .. }));

    // Write creates file atomically, then Check succeeds on identical content
    write_or_check_file(&file_path, "pub mod a;\n", CodegenMode::Write).unwrap();
    write_or_check_file(&file_path, "pub mod a;\n", CodegenMode::Check).unwrap();

    // Check fails when content drifts
    let drift_err =
        write_or_check_file(&file_path, "pub mod b;\n", CodegenMode::Check).unwrap_err();
    assert!(matches!(drift_err, XtaskError::GeneratedFileDrift { .. }));

    fs::remove_dir_all(&tmp_dir).unwrap();
}
