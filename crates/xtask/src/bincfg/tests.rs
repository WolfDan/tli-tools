//! Unit tests for `xtask::bincfg` stem normalization, domain classification, and column resolution.

use crate::bincfg::domains::{MapKeyWire, WireType, classify_bin_domain, normalize_field_name};
use crate::bincfg::emit::{BinTableSpec, SubStructRegistry};
use crate::bincfg::luabridge::{LuaFieldInfo, resolve_column_names};
use crate::bincfg::pe::{ExtractedKeySchema, normalize_stem};
use quote::quote;
use std::collections::BTreeMap;

#[test]
fn normalize_stem_strips_prefixes_suffixes_and_case() {
    assert_eq!(normalize_stem("ItemBaseInfo.cpp"), "itembase");
    assert_eq!(normalize_stem("config_item_base_ChunkA0.bin"), "itembase");
    assert_eq!(
        normalize_stem("config_S11_Gameplay_Bounty.bin"),
        "s11gameplaybounty"
    );
    assert_eq!(normalize_stem("config_ui_style_CN_ChunkA0.bin"), "uistyle");
    assert_eq!(
        normalize_stem("config_ui_style_v2_EN_ChunkA0.bin"),
        "uistylev2"
    );
}

#[test]
fn classify_bin_domain_handles_map_season_and_fallback() {
    assert_eq!(
        classify_bin_domain("config_map_monster_point", true),
        "map_instance"
    );
    assert_eq!(
        classify_bin_domain("config_S11_Gameplay_Bounty", false),
        "season"
    );
    assert_eq!(classify_bin_domain("config_item_base", false), "item");
    assert_eq!(classify_bin_domain("config_modifier", false), "modifier");
    assert_eq!(classify_bin_domain("config_ui_imageV2", false), "ui");
    assert_eq!(
        classify_bin_domain("config_Level_loading_pic", false),
        "level"
    );
    assert_eq!(
        classify_bin_domain("config_future_new_feature", false),
        "system"
    );
}

#[test]
fn normalize_field_name_and_resolve_column_names() {
    assert_eq!(normalize_field_name("__Id__", 0), "id");
    assert_eq!(normalize_field_name("1stTier", 1), "f_1st_tier");
    assert_eq!(normalize_field_name("___", 2), "col_2");
    assert_eq!(normalize_field_name("type", 3), "r#type");

    let cols = vec![
        (WireType::I32, Some(0x10)),
        (WireType::Str, Some(0x18)),
        (WireType::Bool, Some(0x28)),
        (WireType::F64, None),
    ];
    let lua_fields = vec![
        LuaFieldInfo {
            offset: 0x10,
            name: "Id".to_owned(),
            is_translate_str: false,
        },
        LuaFieldInfo {
            offset: 0x18,
            name: "Name".to_owned(),
            is_translate_str: true,
        },
    ];
    let resolved = resolve_column_names(&cols, Some(&lua_fields));
    assert_eq!(
        resolved,
        vec![
            ("id".to_owned(), WireType::I32, false),
            ("name".to_owned(), WireType::Str, true),
            ("col_2".to_owned(), WireType::Bool, false),
            ("col_3".to_owned(), WireType::F64, false),
        ]
    );
    assert_eq!(WireType::F64.align_tier(), 0);
    assert_eq!(WireType::I32.align_tier(), 1);
    assert_eq!(WireType::Bool.align_tier(), 2);
}

#[test]
fn substruct_registry_tokens_parse_and_format_with_prettyplease() {
    let pair_wt = WireType::Struct(vec![WireType::I32, WireType::I32]);
    let spec = BinTableSpec {
        config_stem: "config_item_base".to_owned(),
        pascal: "ItemBase".to_owned(),
        is_map_table: false,
        shared_with_chunk_domain: None,
        key_schema: ExtractedKeySchema::Int(1),
        cols: vec![
            ("r#type".to_owned(), WireType::I32, false),
            (
                "pairs".to_owned(),
                WireType::Array(Box::new(pair_wt.clone())),
                false,
            ),
            (
                "weights".to_owned(),
                WireType::Map(MapKeyWire::Str, Box::new(pair_wt.clone())),
                false,
            ),
        ],
    };
    let mut domains = BTreeMap::new();
    domains.insert("item", vec![spec]);
    let reg = SubStructRegistry::from_schemas(&domains, &[]);
    assert_eq!(
        reg.struct_name(&[WireType::I32, WireType::I32]),
        "FConfigI32I32"
    );

    let pair_ty = reg.rust_type_tokens(&pair_wt);
    let read_expr = reg.read_expr_tokens(&WireType::Array(Box::new(pair_wt)));
    let file_tokens = quote! {
        pub fn sample(cur: &mut WireCursor<'_>) -> Result<Box<[#pair_ty]>, BinError> {
            Ok(#read_expr)
        }
    };
    let parsed: syn::File = syn::parse2(file_tokens).expect("valid Rust syntax tree");
    let formatted = prettyplease::unparse(&parsed);
    assert!(formatted.contains("FConfigI32I32::read_array(cur)?"));
}
