//! UnLua / Xindong Lua 5.3 bytecode (`\x01XDI`) stream-cipher decryptor (`0x20210507`),
//! `DataCfg/config_*.out` VM table evaluator, and `config_multi_language_*.bin` parser.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Xindong Lua 5.3 bytecode magic after decryption (`\x01XDI\x11\r\n\x1a\n\x01`).
pub const XDI_LUA_HEADER: &[u8; 10] = b"\x01XDI\x11\r\n\x1a\n\x01";

/// Decrypts a `.out` Lua bytecode buffer using *Torchlight: Infinite*'s `0x20210507` stream cipher.
/// If the buffer is already decrypted (starts with `\x01XDI`), returns a clone unchanged.
pub fn decrypt_luac_bytes(binary_data: &[u8]) -> Vec<u8> {
    if binary_data.starts_with(b"\x01XDI") {
        return binary_data.to_vec();
    }
    let mut out = Vec::with_capacity(binary_data.len());
    let mut v11: u64 = 0;
    for (read_count, &b) in binary_data.iter().enumerate() {
        let rc = read_count as u64;
        let dec = if read_count <= 1 {
            b
        } else if read_count % 3 == 0 {
            b ^ ((rc.wrapping_add(v11 & 1).wrapping_add(v11 >> 28)) as u8)
        } else if read_count % 3 == 1 {
            b ^ ((((v11 >> 16) & 0xFF).wrapping_sub(rc)) as u8)
        } else {
            b ^ ((rc | (v11 >> 21)) as u8)
        };
        out.push(dec);
        v11 = v11.wrapping_add(0x2021_0507);
    }
    out
}

/// Dynamic Lua constant / table value reconstructed from `DataCfg/config_*.out` bytecode.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LuaValue {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(Vec<LuaValue>),
    Map(BTreeMap<String, LuaValue>),
}

impl LuaValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            LuaValue::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            LuaValue::Int(v) => Some(*v),
            LuaValue::Float(v) => Some(*v as i64),
            _ => None,
        }
    }
}

/// Reconstructed read-only config table from `UE_game/Content/Script/Luac/Config/LuaTable/DataCfg/config_*.out`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LuaConfigTable {
    pub source_name: String,
    pub default_values: BTreeMap<String, LuaValue>,
    pub rows: BTreeMap<i64, BTreeMap<String, LuaValue>>,
}

fn read_u32_le(buf: &[u8], pos: &mut usize) -> Result<u32, String> {
    if *pos + 4 > buf.len() {
        return Err(format!("Unexpected EOF reading u32 at 0x{:x}", *pos));
    }
    let v = u32::from_le_bytes(buf[*pos..*pos + 4].try_into().unwrap());
    *pos += 4;
    Ok(v)
}

fn read_u64_le(buf: &[u8], pos: &mut usize) -> Result<u64, String> {
    if *pos + 8 > buf.len() {
        return Err(format!("Unexpected EOF reading u64 at 0x{:x}", *pos));
    }
    let v = u64::from_le_bytes(buf[*pos..*pos + 8].try_into().unwrap());
    *pos += 8;
    Ok(v)
}

fn read_i64_le(buf: &[u8], pos: &mut usize) -> Result<i64, String> {
    if *pos + 8 > buf.len() {
        return Err(format!("Unexpected EOF reading i64 at 0x{:x}", *pos));
    }
    let v = i64::from_le_bytes(buf[*pos..*pos + 8].try_into().unwrap());
    *pos += 8;
    Ok(v)
}

fn read_f64_le(buf: &[u8], pos: &mut usize) -> Result<f64, String> {
    if *pos + 8 > buf.len() {
        return Err(format!("Unexpected EOF reading f64 at 0x{:x}", *pos));
    }
    let v = f64::from_le_bytes(buf[*pos..*pos + 8].try_into().unwrap());
    *pos += 8;
    Ok(v)
}

/// Reads an XDI Lua 5.3 string where the length prefix has `+ 10` added to the byte length.
fn read_xdi_string(buf: &[u8], pos: &mut usize) -> Result<String, String> {
    if *pos >= buf.len() {
        return Err(format!(
            "Unexpected EOF reading string length at 0x{:x}",
            *pos
        ));
    }
    let lb = buf[*pos];
    *pos += 1;
    let slen = if lb == 0xFF {
        let raw = read_u64_le(buf, pos)? as usize;
        raw.saturating_sub(10)
    } else if lb == 0 {
        0
    } else {
        (lb as usize).saturating_sub(10)
    };
    if *pos + slen > buf.len() {
        return Err(format!(
            "String length {} exceeds buffer at 0x{:x} (len={})",
            slen,
            *pos,
            buf.len()
        ));
    }
    let s = String::from_utf8_lossy(&buf[*pos..*pos + slen]).into_owned();
    *pos += slen;
    Ok(s)
}

/// Parses a decrypted `\x01XDI` Lua 5.3 `DataCfg/config_*.out` file by decoding its constant pool
/// and executing the top-level table-construction instructions (`NEWTABLE`, `LOADK`, `SETTABLE`, `SETLIST`),
/// merging the metatable `default_values` (`__index`) into every row.
pub fn parse_xdi_config_table(raw_or_dec: &[u8]) -> Result<LuaConfigTable, String> {
    let dec = decrypt_luac_bytes(raw_or_dec);
    if dec.len() < 16 || &dec[..10] != XDI_LUA_HEADER {
        return Err("Buffer does not start with valid \\x01XDI Lua 5.3 header".to_string());
    }
    let mut pos = 10usize;
    let source_name = read_xdi_string(&dec, &mut pos)?;

    // Skip linedefined (4), lastlinedefined (4), numparams (1), is_vararg (1), maxstacksize (1)
    if pos + 11 > dec.len() {
        return Err("Unexpected EOF in Lua proto header".to_string());
    }
    pos += 11;

    let sizecode = read_u32_le(&dec, &mut pos)? as usize;
    if pos + sizecode * 4 > dec.len() {
        return Err("Unexpected EOF in Lua code section".to_string());
    }
    let mut code = Vec::with_capacity(sizecode);
    for _ in 0..sizecode {
        code.push(read_u32_le(&dec, &mut pos)?);
    }

    let sizek = read_u32_le(&dec, &mut pos)? as usize;
    let mut consts = Vec::with_capacity(sizek);
    for _ in 0..sizek {
        if pos >= dec.len() {
            return Err("Unexpected EOF in Lua constant pool".to_string());
        }
        let tag = dec[pos];
        pos += 1;
        match tag {
            0x00 => consts.push(LuaValue::Nil),
            0x01 => {
                if pos >= dec.len() {
                    return Err("Unexpected EOF reading bool constant".to_string());
                }
                consts.push(LuaValue::Bool(dec[pos] != 0));
                pos += 1;
            }
            0x03 => consts.push(LuaValue::Float(read_f64_le(&dec, &mut pos)?)),
            0x13 => consts.push(LuaValue::Int(read_i64_le(&dec, &mut pos)?)),
            0x04 | 0x14 => consts.push(LuaValue::Str(read_xdi_string(&dec, &mut pos)?)),
            other => {
                return Err(format!(
                    "Unsupported Lua constant tag 0x{:02x} at 0x{:x}",
                    other,
                    pos - 1
                ));
            }
        }
    }

    // Evaluate the table-building instructions in the main chunk
    let mut regs: Vec<LuaValue> = vec![LuaValue::Nil; 256];
    let rk = |val: usize, regs: &[LuaValue], consts: &[LuaValue]| -> LuaValue {
        if (val & 0x100) != 0 {
            consts.get(val & 0xFF).cloned().unwrap_or(LuaValue::Nil)
        } else {
            regs.get(val).cloned().unwrap_or(LuaValue::Nil)
        }
    };

    // Track row maps inserted into R(0) by integer ID
    let mut raw_rows: BTreeMap<i64, BTreeMap<String, LuaValue>> = BTreeMap::new();

    for w in code {
        let op = (w >> 26) & 0x3F;
        let a = (w & 0xFF) as usize;
        let c = ((w >> 8) & 0x1FF) as usize;
        let b = ((w >> 17) & 0x1FF) as usize;
        let bx = ((w >> 8) & 0x3FFFF) as usize;

        match op {
            4 => {
                // NEWTABLE
                regs[a] = LuaValue::Map(BTreeMap::new());
            }
            6 => {
                // LOADK
                regs[a] = consts.get(bx).cloned().unwrap_or(LuaValue::Nil);
            }
            2 => {
                // SETTABLE A, B, C
                let key = rk(b, &regs, &consts);
                let val = rk(c, &regs, &consts);
                if a == 0 {
                    if let (Some(id), LuaValue::Map(row_map)) = (key.as_i64(), val) {
                        raw_rows.insert(id, row_map);
                    }
                } else if let LuaValue::Map(ref mut m) = regs[a] {
                    let key_str = match key {
                        LuaValue::Str(s) => s,
                        LuaValue::Int(i) => i.to_string(),
                        _ => continue,
                    };
                    m.insert(key_str, val);
                }
            }
            28 => {
                // SETLIST A, B, C
                let mut list = Vec::with_capacity(b);
                for i in 1..=b {
                    if a + i < regs.len() {
                        list.push(regs[a + i].clone());
                    }
                }
                regs[a] = LuaValue::List(list);
            }
            _ => {}
        }
    }

    let default_values = match &regs[1] {
        LuaValue::Map(m) => m.clone(),
        _ => BTreeMap::new(),
    };

    let mut rows = BTreeMap::new();
    for (id, row_map) in raw_rows {
        let mut merged = default_values.clone();
        for (k, v) in row_map {
            merged.insert(k, v);
        }
        rows.insert(id, merged);
    }

    Ok(LuaConfigTable {
        source_name,
        default_values,
        rows,
    })
}

/// Recursively walks all function protos inside a decrypted `\x01XDI` Lua 5.3 bytecode buffer
/// and returns `(source_name, all_string_constants)`.
pub fn extract_xdi_strings(raw_or_dec: &[u8]) -> Result<(String, Vec<String>), String> {
    let dec = decrypt_luac_bytes(raw_or_dec);
    if dec.len() < 16 || &dec[..10] != XDI_LUA_HEADER {
        return Err("Buffer does not start with valid \\x01XDI Lua 5.3 header".to_string());
    }
    let mut pos = 10usize;
    let mut strings = Vec::new();
    let source_name = parse_proto_strings(&dec, &mut pos, &mut strings)?;
    Ok((source_name, strings))
}

fn parse_proto_strings(
    dec: &[u8],
    pos: &mut usize,
    out_strings: &mut Vec<String>,
) -> Result<String, String> {
    let src = read_xdi_string(dec, pos)?;
    if *pos + 11 > dec.len() {
        return Err("EOF in proto header".to_string());
    }
    *pos += 11; // linedefined (4), lastlinedefined (4), numparams (1), is_vararg (1), maxstacksize (1)

    let sizecode = read_u32_le(dec, pos)? as usize;
    *pos += sizecode * 4;

    let sizek = read_u32_le(dec, pos)? as usize;
    for _ in 0..sizek {
        if *pos >= dec.len() {
            break;
        }
        let tag = dec[*pos];
        *pos += 1;
        match tag {
            0x00 => {}
            0x01 => *pos += 1,
            0x03 | 0x13 => *pos += 8,
            0x04 | 0x14 => {
                if let Ok(s) = read_xdi_string(dec, pos)
                    && !s.is_empty()
                {
                    out_strings.push(s);
                }
            }
            _ => break,
        }
    }

    // Upvalues: sizeupvalues (u32), each 2 bytes (instack: u8, idx: u8)
    if let Ok(sizeup) = read_u32_le(dec, pos) {
        let up_bytes = (sizeup as usize) * 2;
        if *pos + up_bytes <= dec.len() {
            *pos += up_bytes;
            // Child protos: sizep (u32)
            if let Ok(sizep) = read_u32_le(dec, pos) {
                for _ in 0..sizep {
                    let _ = parse_proto_strings(dec, pos, out_strings);
                }
            }
            // Debug info: sizelineinfo (u32 * 4), sizelocvars, sizeupvalue_names
            if let Ok(sizeline) = read_u32_le(dec, pos) {
                *pos = (*pos + (sizeline as usize) * 4).min(dec.len());
                if let Ok(sizelocvars) = read_u32_le(dec, pos) {
                    for _ in 0..sizelocvars {
                        let _ = read_xdi_string(dec, pos);
                        *pos = (*pos + 8).min(dec.len());
                    }
                }
                if let Ok(sizeupnames) = read_u32_le(dec, pos) {
                    for _ in 0..sizeupnames {
                        let _ = read_xdi_string(dec, pos);
                    }
                }
            }
        }
    }

    Ok(src)
}

/// Parses `UE_game/Content/Script/Data/language/config_multi_language_*.bin`.
///
/// Each record in the language binary has the layout:
/// `<u32 klen><key_bytes><u32 klen><key_bytes><u32 0><u32 vlen><utf8_value_bytes>`.
/// If `prefix_filter` is `Some(prefix)`, only keys starting with `prefix` are collected.
pub fn parse_multi_language_bin(
    data: &[u8],
    prefix_filter: Option<&str>,
) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let prefix_bytes = prefix_filter.map(|s| s.as_bytes()).unwrap_or(&[]);
    let n = data.len();
    if n < 20 {
        return map;
    }

    let mut i = 0usize;
    while i + 16 <= n {
        let klen = u32::from_le_bytes(data[i..i + 4].try_into().unwrap()) as usize;
        if (4..=256).contains(&klen) && i + 8 + 2 * klen + 8 <= n {
            let k1_start = i + 4;
            let k1_end = k1_start + klen;
            if prefix_bytes.is_empty() || data[k1_start..k1_end].starts_with(prefix_bytes) {
                let klen2 =
                    u32::from_le_bytes(data[k1_end..k1_end + 4].try_into().unwrap()) as usize;
                if klen2 == klen {
                    let k2_start = k1_end + 4;
                    let k2_end = k2_start + klen;
                    if data[k1_start..k1_end] == data[k2_start..k2_end] {
                        let flag = u32::from_le_bytes(data[k2_end..k2_end + 4].try_into().unwrap());
                        let vlen =
                            u32::from_le_bytes(data[k2_end + 4..k2_end + 8].try_into().unwrap())
                                as usize;
                        let val_start = k2_end + 8;
                        let val_end = val_start.saturating_add(vlen);
                        if flag == 0 && vlen <= 32768 && val_end <= n {
                            if let (Ok(k), Ok(v)) = (
                                std::str::from_utf8(&data[k1_start..k1_end]),
                                std::str::from_utf8(&data[val_start..val_end]),
                            ) {
                                map.insert(k.to_string(), v.to_string());
                                i = val_end;
                                continue;
                            }
                        }
                    }
                }
            }
        }
        i += 1;
    }

    map
}

/// Strips `<RichText...>` / `</>` / `<Img...>` markup tags and normalizes escaped newlines.
pub fn strip_rich_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.replace("\\n", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
