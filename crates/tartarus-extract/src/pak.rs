//! Unreal Engine 4.27 Pak v12 (`0x6B2A56B8`) archive reader, index decryptor, and Oodle/Zlib decompressor.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;
use rayon::prelude::*;
use serde::Serialize;
use steamlocate::SteamDir;

use crate::aes::decrypt_aes256_ecb_in_place;

/// Torchlight: Infinite custom UE 4.27 Pak v12 footer magic (`0x6B2A56B8`).
pub const TLI_PAK_MAGIC: u32 = 0x6B2A_56B8;
/// Standard UE4 Pak footer magic (`0x5A6F12E1`).
pub const UE4_PAK_MAGIC: u32 = 0x5A6F_12E1;
/// Steam App ID for *Torchlight: Infinite*.
pub const TLI_STEAM_APP_ID: u32 = 1_974_050;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum CompressionKind {
    None,
    Oodle,
    Zlib,
    Unknown(u8),
}

#[derive(Clone, Debug, Serialize)]
pub struct PakEntryMeta {
    pub pak_path: PathBuf,
    pub pak_name: String,
    pub virtual_path: String,
    pub offset: u64,
    pub comp_size: u64,
    pub uncomp_size: u64,
    pub compression: CompressionKind,
    pub encrypted: bool,
    pub comp_block_size: u32,
    pub is_premount: bool,
}

#[derive(Clone, Debug)]
pub struct PakVfs {
    pub key: [u8; 32],
    pub entries: BTreeMap<String, PakEntryMeta>,
}

impl PakVfs {
    /// Discovers and indexes all `.pak` archives under a Torchlight: Infinite installation root
    /// (or a single `.pak` file / directory of `.pak` files) in parallel using Rayon.
    pub fn open_game_dir(root: &Path, key: [u8; 32]) -> Result<Self, String> {
        let mut pak_files: Vec<(PathBuf, bool)> = Vec::new();

        if root.is_file() {
            pak_files.push((root.to_path_buf(), false));
        } else {
            let candidates = [
                (root.join("UE_game/Content/Paks"), false),
                (root.join("Content/Paks"), false),
                (root.to_path_buf(), false),
                (
                    root.join("UE_game/TorchLight/Saved/PersistentDownloadDir/PreMountPak"),
                    true,
                ),
                (
                    root.join("TorchLight/Saved/PersistentDownloadDir/PreMountPak"),
                    true,
                ),
            ];
            for (dir, is_premount) in candidates {
                if let Ok(rd) = fs::read_dir(&dir) {
                    let mut list: Vec<PathBuf> = rd
                        .filter_map(|e| e.ok().map(|e| e.path()))
                        .filter(|p| p.extension().is_some_and(|ext| ext == "pak"))
                        .collect();
                    list.sort();
                    for p in list {
                        if !pak_files.iter().any(|(existing, _)| existing == &p) {
                            pak_files.push((p, is_premount));
                        }
                    }
                }
            }
        }

        if pak_files.is_empty() {
            return Err(format!("No .pak files found under {}", root.display()));
        }

        let per_pak: Vec<Vec<PakEntryMeta>> = pak_files
            .par_iter()
            .filter_map(
                |(path, is_premount)| match read_pak_index(path, &key, *is_premount) {
                    Ok(list) => Some(list),
                    Err(e) => {
                        eprintln!("WARN: skipping {}: {e}", path.display());
                        None
                    }
                },
            )
            .collect();

        let mut entries = BTreeMap::new();
        // Insert base paks first, then premount hotfix paks so premount overrides base
        for list in &per_pak {
            for item in list {
                if !item.is_premount {
                    entries.insert(item.virtual_path.clone(), item.clone());
                }
            }
        }
        for list in &per_pak {
            for item in list {
                if item.is_premount {
                    entries.insert(item.virtual_path.clone(), item.clone());
                }
            }
        }

        Ok(Self { key, entries })
    }

    /// Reads, decrypts, and decompresses a single file entry from its `.pak` archive into memory.
    pub fn read_entry(&self, meta: &PakEntryMeta) -> Result<Vec<u8>, String> {
        extract_pak_entry(meta, &self.key)
    }

    /// Reads a file by its virtual path (e.g. `"UIV2/S16Gameplay/UI/Textures/NoAtlas/..."`).
    pub fn read_path(&self, virtual_path: &str) -> Result<Vec<u8>, String> {
        let clean = virtual_path
            .trim_start_matches('/')
            .trim_start_matches("TorchLight/Content/");
        let meta = self
            .entries
            .get(clean)
            .or_else(|| self.entries.get(virtual_path.trim_start_matches('/')))
            .ok_or_else(|| format!("Virtual path not found in Pak VFS: {clean}"))?;
        self.read_entry(meta)
    }
}

fn read_fstring(cur: &mut Cursor<&[u8]>) -> Result<String, String> {
    let len = cur.read_i32::<LittleEndian>().map_err(|e| e.to_string())?;
    if len == 0 {
        return Ok(String::new());
    }
    let pos = cur.position() as usize;
    let buf = *cur.get_ref();
    if len < 0 {
        let chars = (-len) as usize;
        let byte_len = chars * 2;
        let slice = buf
            .get(pos..pos + byte_len)
            .ok_or_else(|| format!("Unexpected EOF reading UTF-16 FString at {pos}"))?;
        cur.set_position((pos + byte_len) as u64);
        let mut u16s = vec![0u16; chars];
        LittleEndian::read_u16_into(slice, &mut u16s);
        Ok(String::from_utf16_lossy(&u16s)
            .trim_end_matches('\0')
            .to_string())
    } else {
        let byte_len = len as usize;
        let slice = buf
            .get(pos..pos + byte_len)
            .ok_or_else(|| format!("Unexpected EOF reading UTF-8 FString at {pos}"))?;
        cur.set_position((pos + byte_len) as u64);
        Ok(String::from_utf8_lossy(slice)
            .trim_end_matches('\0')
            .to_string())
    }
}

fn normalize_virtual_path(mount_point: &str, dir_name: &str, file_name: &str) -> String {
    let mp = mount_point
        .trim_start_matches("../")
        .trim_start_matches('/')
        .trim_start_matches("TorchLight/Content/")
        .trim_start_matches('/');
    let dir = dir_name.trim_start_matches('/');
    let raw = if mp.is_empty() {
        format!("{dir}{file_name}")
    } else {
        format!("{mp}{dir}{file_name}")
    };
    raw.replace("//", "/").trim_start_matches('/').to_string()
}

pub fn read_pak_index(
    pak_path: &Path,
    key: &[u8; 32],
    is_premount: bool,
) -> Result<Vec<PakEntryMeta>, String> {
    let mut file =
        File::open(pak_path).map_err(|e| format!("Failed to open {}: {e}", pak_path.display()))?;
    let file_size = file
        .seek(SeekFrom::End(0))
        .map_err(|e| format!("Failed to seek {}: {e}", pak_path.display()))?;
    if file_size < 240 {
        return Ok(Vec::new());
    }

    let tail_len = file_size.min(300) as usize;
    file.seek(SeekFrom::End(-(tail_len as i64)))
        .map_err(|e| e.to_string())?;
    let mut tail = vec![0u8; tail_len];
    file.read_exact(&mut tail).map_err(|e| e.to_string())?;

    let mut magic_pos: Option<(usize, bool)> = None;
    for i in (1..=tail.len().saturating_sub(45)).rev() {
        let m = LittleEndian::read_u32(&tail[i..i + 4]);
        if m == TLI_PAK_MAGIC {
            magic_pos = Some((i, true));
            break;
        } else if m == UE4_PAK_MAGIC {
            magic_pos = Some((i, false));
            break;
        }
    }
    let Some((pos, is_tli_v12)) = magic_pos else {
        return Err(format!("Pak magic not found in {}", pak_path.display()));
    };

    let enc_index = tail[pos - 1] != 0;
    let version = LittleEndian::read_u32(&tail[pos + 4..]);
    // Torchlight: Infinite Pak v12 inserts a 1-byte flag right after `Version: u32`
    let extra_shift = if is_tli_v12 || version >= 12 { 1 } else { 0 };
    let index_offset = LittleEndian::read_u64(&tail[pos + 8 + extra_shift..]);
    let index_size = LittleEndian::read_u64(&tail[pos + 16 + extra_shift..]) as usize;

    let comp_table_start = pos + 44 + extra_shift;
    let mut comp_methods = vec![CompressionKind::None];
    for c_idx in 0..5 {
        let c_off = comp_table_start + c_idx * 32;
        if c_off + 32 > tail.len() {
            break;
        }
        let slice = &tail[c_off..c_off + 32];
        let nul = memchr::memchr(0, slice).unwrap_or(32);
        if nul == 0 {
            continue;
        }
        let name = String::from_utf8_lossy(&slice[..nul]);
        let kind = if name.eq_ignore_ascii_case("Oodle") {
            CompressionKind::Oodle
        } else if name.eq_ignore_ascii_case("Zlib") {
            CompressionKind::Zlib
        } else {
            CompressionKind::Unknown(c_idx as u8 + 1)
        };
        comp_methods.push(kind);
    }

    if index_offset + (index_size as u64) > file_size {
        return Err(format!(
            "Invalid index offset/size in {}",
            pak_path.display()
        ));
    }

    file.seek(SeekFrom::Start(index_offset))
        .map_err(|e| e.to_string())?;
    let mut idx_buf = vec![0u8; index_size];
    file.read_exact(&mut idx_buf).map_err(|e| e.to_string())?;
    if enc_index {
        decrypt_aes256_ecb_in_place(&mut idx_buf, key);
    }

    let mut idx_cur = Cursor::new(idx_buf.as_slice());
    let mount_point = read_fstring(&mut idx_cur)?;
    let _num_entries = idx_cur
        .read_i32::<LittleEndian>()
        .map_err(|e| e.to_string())?;
    let _path_hash_seed = idx_cur
        .read_u64::<LittleEndian>()
        .map_err(|e| e.to_string())?;

    let has_path_hash = idx_cur
        .read_i32::<LittleEndian>()
        .map_err(|e| e.to_string())?
        != 0;
    if has_path_hash {
        idx_cur.set_position(idx_cur.position() + 36);
    }

    let has_full_dir = idx_cur
        .read_i32::<LittleEndian>()
        .map_err(|e| e.to_string())?
        != 0;
    if !has_full_dir {
        return Ok(Vec::new());
    }
    let full_dir_offset = idx_cur
        .read_u64::<LittleEndian>()
        .map_err(|e| e.to_string())?;
    let full_dir_size = idx_cur
        .read_u64::<LittleEndian>()
        .map_err(|e| e.to_string())? as usize;
    idx_cur.set_position(idx_cur.position() + 20);

    let encoded_entries_size = idx_cur
        .read_i32::<LittleEndian>()
        .map_err(|e| e.to_string())? as usize;
    let off = idx_cur.position() as usize;
    let encoded_entries = idx_buf
        .get(off..off + encoded_entries_size)
        .ok_or_else(|| "Truncated EncodedPakEntries".to_string())?;

    file.seek(SeekFrom::Start(full_dir_offset))
        .map_err(|e| e.to_string())?;
    let mut dir_buf = vec![0u8; full_dir_size];
    file.read_exact(&mut dir_buf).map_err(|e| e.to_string())?;
    if enc_index {
        decrypt_aes256_ecb_in_place(&mut dir_buf, key);
    }

    let mut dir_cur = Cursor::new(dir_buf.as_slice());
    let dir_count = dir_cur
        .read_i32::<LittleEndian>()
        .map_err(|e| e.to_string())? as usize;
    let pak_name = pak_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut out = Vec::new();
    for _ in 0..dir_count {
        let dir_name = read_fstring(&mut dir_cur)?;
        let file_count = dir_cur
            .read_i32::<LittleEndian>()
            .map_err(|e| e.to_string())? as usize;
        for _ in 0..file_count {
            let file_name = read_fstring(&mut dir_cur)?;
            let enc_entry_off = dir_cur
                .read_i32::<LittleEndian>()
                .map_err(|e| e.to_string())?;
            if enc_entry_off < 0 {
                continue;
            }

            let vpath = normalize_virtual_path(&mount_point, &dir_name, &file_name);
            if let Ok(meta) = decode_compact_pak_entry(
                encoded_entries,
                enc_entry_off as usize,
                pak_path,
                &pak_name,
                vpath,
                &comp_methods,
                is_premount,
            ) {
                out.push(meta);
            }
        }
    }

    Ok(out)
}

fn decode_compact_pak_entry(
    buf: &[u8],
    off: usize,
    pak_path: &Path,
    pak_name: &str,
    virtual_path: String,
    comp_methods: &[CompressionKind],
    is_premount: bool,
) -> Result<PakEntryMeta, String> {
    let mut cur = Cursor::new(
        buf.get(off..)
            .ok_or("Compact pak entry offset out of bounds")?,
    );
    let bits = cur.read_u32::<LittleEndian>().map_err(|e| e.to_string())?;

    let comp_block_size_raw = bits & 0x3F;
    let _comp_blocks_count = (bits >> 6) & 0xFFFF;
    let encrypted = ((bits >> 22) & 1) != 0;
    let comp_method_idx = ((bits >> 23) & 0x3F) as usize;
    let size_32 = ((bits >> 29) & 1) != 0;
    let uncomp_32 = ((bits >> 30) & 1) != 0;
    let off_32 = ((bits >> 31) & 1) != 0;

    let comp_block_size = if comp_block_size_raw == 0x3F {
        cur.read_u32::<LittleEndian>().map_err(|e| e.to_string())?
    } else {
        comp_block_size_raw << 11
    };

    let offset = if off_32 {
        cur.read_u32::<LittleEndian>().map_err(|e| e.to_string())? as u64
    } else {
        cur.read_u64::<LittleEndian>().map_err(|e| e.to_string())?
    };

    let uncomp_size = if uncomp_32 {
        cur.read_u32::<LittleEndian>().map_err(|e| e.to_string())? as u64
    } else {
        cur.read_u64::<LittleEndian>().map_err(|e| e.to_string())?
    };

    let comp_size = if comp_method_idx != 0 {
        if size_32 {
            cur.read_u32::<LittleEndian>().map_err(|e| e.to_string())? as u64
        } else {
            cur.read_u64::<LittleEndian>().map_err(|e| e.to_string())?
        }
    } else {
        uncomp_size
    };

    let compression = comp_methods
        .get(comp_method_idx)
        .copied()
        .unwrap_or(CompressionKind::Unknown(comp_method_idx as u8));

    Ok(PakEntryMeta {
        pak_path: pak_path.to_path_buf(),
        pak_name: pak_name.to_string(),
        virtual_path,
        offset,
        comp_size,
        uncomp_size,
        compression,
        encrypted,
        comp_block_size,
        is_premount,
    })
}

pub fn extract_pak_entry(meta: &PakEntryMeta, key: &[u8; 32]) -> Result<Vec<u8>, String> {
    let mut file = File::open(&meta.pak_path)
        .map_err(|e| format!("Failed to open {}: {e}", meta.pak_path.display()))?;
    file.seek(SeekFrom::Start(meta.offset))
        .map_err(|e| e.to_string())?;

    // Read fixed part of inline Pak v12 FPakEntry header:
    // 0..8: i64 offset (0)
    // 8: u8 v12 custom byte (0)
    // 9..17: i64 comp_size
    // 17..25: i64 uncomp_size
    // 25..29: u32 comp_method
    // 29..49: [u8; 20] sha1
    let mut base_hdr = [0u8; 49];
    file.read_exact(&mut base_hdr).map_err(|e| e.to_string())?;
    let inline_comp_method = LittleEndian::read_u32(&base_hdr[25..29]);

    if inline_comp_method == 0 || meta.compression == CompressionKind::None {
        let mut tail_hdr = [0u8; 5]; // u8 encrypted + u32 comp_block_size
        file.read_exact(&mut tail_hdr).map_err(|e| e.to_string())?;
        let is_enc = (tail_hdr[0] != 0) || meta.encrypted;

        let uncomp_len = meta.uncomp_size as usize;
        let read_len = if is_enc {
            (uncomp_len + 15) & !15
        } else {
            uncomp_len
        };
        let mut data = vec![0u8; read_len];
        file.read_exact(&mut data).map_err(|e| e.to_string())?;
        if is_enc {
            decrypt_aes256_ecb_in_place(&mut data, key);
            data.truncate(uncomp_len);
        }
        return Ok(data);
    }

    let num_blocks = file.read_i32::<LittleEndian>().map_err(|e| e.to_string())? as usize;
    if num_blocks > 65536 {
        return Err(format!(
            "Corrupt inline FPakEntry num_blocks={num_blocks} for {}",
            meta.virtual_path
        ));
    }

    let mut blocks_raw = vec![0u8; num_blocks * 16 + 5];
    file.read_exact(&mut blocks_raw)
        .map_err(|e| e.to_string())?;

    let tail_off = num_blocks * 16;
    let is_enc = (blocks_raw[tail_off] != 0) || meta.encrypted;
    let inline_cbs = LittleEndian::read_u32(&blocks_raw[tail_off + 1..]);
    let block_stride = if inline_cbs > 0 {
        inline_cbs as usize
    } else if meta.comp_block_size > 0 {
        meta.comp_block_size as usize
    } else {
        meta.uncomp_size as usize
    };

    let mut out = vec![0u8; meta.uncomp_size as usize];
    let mut out_pos = 0usize;
    let mut comp_buf = Vec::new();
    let mut oodle_extractor = oozextract::Extractor::new();

    for i in 0..num_blocks {
        let b_start = LittleEndian::read_u64(&blocks_raw[i * 16..]);
        let b_end = LittleEndian::read_u64(&blocks_raw[i * 16 + 8..]);
        let raw_comp_len = (b_end.saturating_sub(b_start)) as usize;
        let abs_start = if b_start >= meta.offset {
            b_start
        } else {
            meta.offset + b_start
        };

        let disk_len = if is_enc {
            (raw_comp_len + 15) & !15
        } else {
            raw_comp_len
        };
        comp_buf.resize(disk_len, 0);
        file.seek(SeekFrom::Start(abs_start))
            .map_err(|e| e.to_string())?;
        file.read_exact(&mut comp_buf).map_err(|e| e.to_string())?;
        if is_enc {
            decrypt_aes256_ecb_in_place(&mut comp_buf, key);
        }
        let comp_slice = &comp_buf[..raw_comp_len];

        let uncomp_chunk_len = (out.len() - out_pos).min(block_stride);
        let dest_slice = &mut out[out_pos..out_pos + uncomp_chunk_len];

        match meta.compression {
            CompressionKind::Oodle => {
                let written = oodle_extractor
                    .read_from_slice(comp_slice, dest_slice)
                    .map_err(|e| {
                        format!(
                            "Oodle decompress failed on {} (block {i}/{num_blocks}, comp={raw_comp_len}, uncomp={uncomp_chunk_len}): {e:?}",
                            meta.virtual_path
                        )
                    })?;
                if written != uncomp_chunk_len {
                    return Err(format!(
                        "Oodle short write on {}: expected {uncomp_chunk_len}, got {written}",
                        meta.virtual_path
                    ));
                }
            }
            CompressionKind::Zlib => {
                let mut z = ZlibDecoder::new(comp_slice);
                z.read_exact(dest_slice)
                    .map_err(|e| format!("Zlib decompress failed on {}: {e}", meta.virtual_path))?;
            }
            CompressionKind::None => {
                dest_slice.copy_from_slice(&comp_slice[..uncomp_chunk_len]);
            }
            CompressionKind::Unknown(id) => {
                return Err(format!(
                    "Unsupported compression method id={id} on {}",
                    meta.virtual_path
                ));
            }
        }

        out_pos += uncomp_chunk_len;
    }

    Ok(out)
}

/// Automatically discovers the *Torchlight: Infinite* installation directory across Linux and Windows.
///
/// Checks `$TLI_GAME_DIR`, `$STEAM_DIR`, and all Steam libraries via `steamlocate` (App ID `1974050`).
pub fn discover_game_dir() -> Option<PathBuf> {
    if let Ok(custom) = std::env::var("TLI_GAME_DIR") {
        let p = PathBuf::from(custom);
        if p.exists() {
            return Some(p);
        }
    }

    let mut steam_dirs: Vec<SteamDir> = Vec::new();
    if let Ok(steam_env) = std::env::var("STEAM_DIR")
        && let Ok(sd) = SteamDir::from_dir(&PathBuf::from(steam_env))
    {
        steam_dirs.push(sd);
    }
    if let Ok(all) = steamlocate::locate_all() {
        steam_dirs.extend(all);
    }

    for sd in &steam_dirs {
        if let Ok(Some((app, lib))) = sd.find_app(TLI_STEAM_APP_ID) {
            let candidate = lib.resolve_app_dir(&app);
            if candidate.join("UE_game/Content/Paks").is_dir()
                || candidate.join("Content/Paks").is_dir()
            {
                return Some(candidate);
            }
        }
        if let Ok(libs) = sd.libraries() {
            for lib in libs.flatten() {
                let candidate = lib.path().join("steamapps/common/Torchlight Infinite");
                if candidate.join("UE_game/Content/Paks").is_dir()
                    || candidate.join("Content/Paks").is_dir()
                {
                    return Some(candidate);
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    for base in [
        "C:\\Program Files (x86)\\Torchlight Infinite",
        "C:\\Program Files\\Torchlight Infinite",
    ] {
        let candidate = PathBuf::from(base);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}
