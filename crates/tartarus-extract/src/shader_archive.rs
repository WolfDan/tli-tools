//! Unreal Engine 4.27 `FSerializedShaderArchive` (`.ushaderbytecode`, `Version = 2`) index parser
//! and Oodle Kraken shader entry decompressor.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

use crate::error::ExtractError;
use crate::pak::{CompressionKind, PakEntryMeta};

pub const SHADER_ARCHIVE_VERSION_UE427: u32 = 2;
const SHADER_MAPS_MAX: usize = 65_536;
const SHADERS_MAX: usize = 1_048_576;
const PRELOAD_ENTRIES_MAX: usize = 1_048_576;
const SHADER_INDICES_MAX: usize = 4_194_304;
const SHADER_BLOB_BYTES_MAX: usize = 16 * 1024 * 1024;
const PAK_INLINE_UNCOMP_HEADER_LEN: u64 = 54;

const _: () = {
    assert!(SHADER_ARCHIVE_VERSION_UE427 == 2);
    assert!(PAK_INLINE_UNCOMP_HEADER_LEN == 54);
};

/// Direct3D 11 / UE4 shader stage (`EShaderFrequency`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ShaderStage {
    Vertex,
    Hull,
    Domain,
    Pixel,
    Geometry,
    Compute,
    Unknown(u8),
}

impl ShaderStage {
    pub fn from_byte(raw: u8) -> Self {
        let stage = match raw {
            0 => Self::Vertex,
            1 => Self::Hull,
            2 => Self::Domain,
            3 => Self::Pixel,
            4 => Self::Geometry,
            5 => Self::Compute,
            other => Self::Unknown(other),
        };
        assert!(matches!(stage, Self::Vertex | Self::Unknown(_)) || raw > 0);
        stage
    }

    pub fn short_prefix(self) -> &'static str {
        let prefix = match self {
            Self::Vertex => "vs",
            Self::Hull => "hs",
            Self::Domain => "ds",
            Self::Pixel => "ps",
            Self::Geometry => "gs",
            Self::Compute => "cs",
            Self::Unknown(_) => "sh",
        };
        assert!(!prefix.is_empty());
        prefix
    }

    pub fn display_name(self) -> &'static str {
        let name = match self {
            Self::Vertex => "Vertex",
            Self::Hull => "Hull",
            Self::Domain => "Domain",
            Self::Pixel => "Pixel",
            Self::Geometry => "Geometry",
            Self::Compute => "Compute",
            Self::Unknown(_) => "Unknown",
        };
        assert!(!name.is_empty());
        name
    }
}

/// `FShaderMapEntry` (`16` bytes) in `FSerializedShaderArchive`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderMapEntry {
    pub shader_indices_offset: u32,
    pub num_shaders: u32,
    pub first_preload_index: u32,
    pub num_preload_entries: u32,
}

/// `FShaderCodeEntry` (`17` bytes) in `FSerializedShaderArchive`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderCodeEntry {
    pub offset: u64,
    pub size: u32,
    pub uncompressed_size: u32,
    pub frequency: ShaderStage,
}

/// Decompressed shader entry with its sliced DirectX `DXBC` container and UE4 resource wrapper.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecompressedShaderBlob {
    pub shader_index: usize,
    pub shader_hash_hex: String,
    pub stage: ShaderStage,
    pub compressed_size: usize,
    pub uncompressed_size: usize,
    pub ue4_header_prefix: Vec<u8>,
    pub dxbc_bytes: Vec<u8>,
    pub ue4_resource_counts_tail: Vec<u8>,
}

/// Parsed index header for a `ShaderArchive-*-PCD3D_SM5.ushaderbytecode` archive.
#[derive(Clone, Debug, Serialize)]
pub struct ShaderArchiveIndex {
    pub archive_virtual_path: String,
    pub pak_name: String,
    pub version: u32,
    pub payload_offset: u64,
    pub shader_map_hashes: Vec<[u8; 20]>,
    pub shader_hashes: Vec<[u8; 20]>,
    pub shader_map_entries: Vec<ShaderMapEntry>,
    pub shader_entries: Vec<ShaderCodeEntry>,
    pub shader_indices: Vec<u32>,
    #[serde(skip)]
    pub map_hash_to_index: BTreeMap<[u8; 20], usize>,
}

fn read_sha1_array<R: Read>(
    reader: &mut R,
    max_len: usize,
    label: &str,
) -> Result<Vec<[u8; 20]>, ExtractError> {
    assert!(max_len > 0);
    assert!(!label.is_empty());
    let count = reader.read_u32::<LittleEndian>()? as usize;
    if count > max_len {
        return Err(ExtractError::ShaderFormat(format!(
            "{label} count {count} exceeds limit {max_len}"
        )));
    }
    let mut raw = vec![0u8; count * 20];
    reader.read_exact(&mut raw)?;
    let mut out = Vec::with_capacity(count);
    for &chunk in raw.as_chunks::<20>().0 {
        out.push(chunk);
    }
    assert_eq!(out.len(), count);
    Ok(out)
}

fn read_shader_map_entries<R: Read>(reader: &mut R) -> Result<Vec<ShaderMapEntry>, ExtractError> {
    let count = reader.read_u32::<LittleEndian>()? as usize;
    if count > SHADER_MAPS_MAX {
        return Err(ExtractError::ShaderFormat(format!(
            "ShaderMapEntries count {count} exceeds {SHADER_MAPS_MAX}"
        )));
    }
    let mut raw = vec![0u8; count * 16];
    reader.read_exact(&mut raw)?;
    let mut out = Vec::with_capacity(count);
    for chunk in raw.as_chunks::<16>().0 {
        let mut cur = Cursor::new(chunk.as_slice());
        out.push(ShaderMapEntry {
            shader_indices_offset: cur.read_u32::<LittleEndian>()?,
            num_shaders: cur.read_u32::<LittleEndian>()?,
            first_preload_index: cur.read_u32::<LittleEndian>()?,
            num_preload_entries: cur.read_u32::<LittleEndian>()?,
        });
    }
    assert_eq!(out.len(), count);
    Ok(out)
}

fn read_shader_code_entries<R: Read>(reader: &mut R) -> Result<Vec<ShaderCodeEntry>, ExtractError> {
    let count = reader.read_u32::<LittleEndian>()? as usize;
    if count > SHADERS_MAX {
        return Err(ExtractError::ShaderFormat(format!(
            "ShaderEntries count {count} exceeds {SHADERS_MAX}"
        )));
    }
    let mut raw = vec![0u8; count * 17];
    reader.read_exact(&mut raw)?;
    let mut out = Vec::with_capacity(count);
    for chunk in raw.as_chunks::<17>().0 {
        let mut cur = Cursor::new(chunk.as_slice());
        let offset = cur.read_u64::<LittleEndian>()?;
        let size = cur.read_u32::<LittleEndian>()?;
        let uncompressed_size = cur.read_u32::<LittleEndian>()?;
        let frequency = ShaderStage::from_byte(cur.read_u8()?);
        out.push(ShaderCodeEntry {
            offset,
            size,
            uncompressed_size,
            frequency,
        });
    }
    assert_eq!(out.len(), count);
    Ok(out)
}

impl ShaderArchiveIndex {
    /// Parses the `FSerializedShaderArchive` header from any `Read + Seek` stream positioned at
    /// byte `0` of the `.ushaderbytecode` archive.
    pub fn parse_from_reader<R: Read + Seek>(
        reader: &mut R,
        archive_virtual_path: &str,
        pak_name: &str,
    ) -> Result<Self, ExtractError> {
        assert!(!archive_virtual_path.is_empty());
        let start_pos = reader.stream_position()?;
        let version = reader.read_u32::<LittleEndian>()?;
        if version != SHADER_ARCHIVE_VERSION_UE427 {
            return Err(ExtractError::ShaderFormat(format!(
                "Unsupported ShaderArchive version {version} in {archive_virtual_path}"
            )));
        }

        let shader_map_hashes = read_sha1_array(reader, SHADER_MAPS_MAX, "ShaderMapHashes")?;
        let shader_hashes = read_sha1_array(reader, SHADERS_MAX, "ShaderHashes")?;
        let shader_map_entries = read_shader_map_entries(reader)?;
        let shader_entries = read_shader_code_entries(reader)?;

        let preload_count = reader.read_u32::<LittleEndian>()? as usize;
        if preload_count > PRELOAD_ENTRIES_MAX {
            return Err(ExtractError::ShaderFormat(format!(
                "PreloadEntries count {preload_count} exceeds {PRELOAD_ENTRIES_MAX}"
            )));
        }
        reader.seek(SeekFrom::Current((preload_count as i64) * 16))?;

        let indices_count = reader.read_u32::<LittleEndian>()? as usize;
        if indices_count > SHADER_INDICES_MAX {
            return Err(ExtractError::ShaderFormat(format!(
                "ShaderIndices count {indices_count} exceeds {SHADER_INDICES_MAX}"
            )));
        }
        let mut idx_raw = vec![0u8; indices_count * 4];
        reader.read_exact(&mut idx_raw)?;
        let mut shader_indices = Vec::with_capacity(indices_count);
        for &chunk in idx_raw.as_chunks::<4>().0 {
            shader_indices.push(u32::from_le_bytes(chunk));
        }

        let payload_offset = reader.stream_position()? - start_pos;
        let mut map_hash_to_index = BTreeMap::new();
        for (i, h) in shader_map_hashes.iter().copied().enumerate() {
            map_hash_to_index.insert(h, i);
        }
        assert_eq!(shader_map_hashes.len(), shader_map_entries.len());
        assert_eq!(shader_hashes.len(), shader_entries.len());

        Ok(Self {
            archive_virtual_path: archive_virtual_path.to_string(),
            pak_name: pak_name.to_string(),
            version,
            payload_offset,
            shader_map_hashes,
            shader_hashes,
            shader_map_entries,
            shader_entries,
            shader_indices,
            map_hash_to_index,
        })
    }

    /// Parses a `ShaderArchiveIndex` from an in-memory `.ushaderbytecode` buffer.
    pub fn from_bytes(
        bytes: &[u8],
        archive_virtual_path: &str,
        pak_name: &str,
    ) -> Result<Self, ExtractError> {
        assert!(!bytes.is_empty());
        assert!(!archive_virtual_path.is_empty());
        let mut cur = Cursor::new(bytes);
        Self::parse_from_reader(&mut cur, archive_virtual_path, pak_name)
    }

    /// Returns the deduplicated list of shader indices belonging to `shader_map_index`.
    pub fn shader_indices_for_map(&self, shader_map_index: usize) -> Vec<usize> {
        assert!(shader_map_index < self.shader_map_entries.len());
        let entry = self.shader_map_entries[shader_map_index];
        let start = entry.shader_indices_offset as usize;
        let end = (start + entry.num_shaders as usize).min(self.shader_indices.len());
        let mut out = Vec::with_capacity(end.saturating_sub(start));
        for &idx in &self.shader_indices[start..end] {
            let si = idx as usize;
            if si < self.shader_entries.len() && !out.contains(&si) {
                out.push(si);
            }
        }
        assert!(out.len() <= entry.num_shaders as usize);
        out
    }

    /// Stage distribution summary `(Stage, count)` across all shaders in this archive.
    pub fn stage_counts(&self) -> BTreeMap<&'static str, usize> {
        assert_eq!(self.shader_entries.len(), self.shader_hashes.len());
        let mut counts = BTreeMap::new();
        for entry in &self.shader_entries {
            *counts.entry(entry.frequency.display_name()).or_insert(0) += 1;
        }
        assert!(counts.len() <= 8);
        counts
    }
}

/// Decompresses a single `FShaderCodeEntry` slice (Oodle Kraken or uncompressed) and splits it
/// into the UE4 header prefix, the `DXBC` container, and the UE4 resource counts trailer.
pub fn decompress_shader_code_blob(
    shader_index: usize,
    shader_hash: &[u8; 20],
    entry: &ShaderCodeEntry,
    comp_slice: &[u8],
) -> Result<DecompressedShaderBlob, ExtractError> {
    assert_eq!(comp_slice.len(), entry.size as usize);
    let uncomp_len = entry.uncompressed_size as usize;
    if uncomp_len == 0 || uncomp_len > SHADER_BLOB_BYTES_MAX {
        return Err(ExtractError::ShaderFormat(format!(
            "Invalid shader[{shader_index}] uncompressed_size={uncomp_len}"
        )));
    }

    let mut raw = vec![0u8; uncomp_len];
    if entry.size == entry.uncompressed_size {
        raw.copy_from_slice(comp_slice);
    } else {
        let mut oodle = oozextract::Extractor::new();
        let written = oodle.read_from_slice(comp_slice, &mut raw).map_err(|e| {
            ExtractError::Decompress(format!(
                "Oodle decompress failed on shader[{shader_index}]: {e:?}"
            ))
        })?;
        if written != uncomp_len {
            return Err(ExtractError::Decompress(format!(
                "Oodle short write on shader[{shader_index}]: expected {uncomp_len}, got {written}"
            )));
        }
    }

    let dxbc_off = memchr::memmem::find(&raw, b"DXBC").ok_or_else(|| {
        ExtractError::ShaderFormat(format!(
            "DXBC magic not found in decompressed shader[{shader_index}]"
        ))
    })?;
    if dxbc_off + 32 > raw.len() {
        return Err(ExtractError::ShaderFormat(format!(
            "Truncated DXBC header in shader[{shader_index}]"
        )));
    }
    let total_size =
        u32::from_le_bytes(raw[dxbc_off + 24..dxbc_off + 28].try_into().unwrap()) as usize;
    let dxbc_end = dxbc_off.checked_add(total_size).ok_or_else(|| {
        ExtractError::ShaderFormat(format!("DXBC size overflow in shader[{shader_index}]"))
    })?;
    if total_size < 32 || dxbc_end > raw.len() {
        return Err(ExtractError::ShaderFormat(format!(
            "Invalid DXBC total_size={total_size} in shader[{shader_index}] (len={})",
            raw.len()
        )));
    }

    assert_eq!(&raw[dxbc_off..dxbc_off + 4], b"DXBC");
    Ok(DecompressedShaderBlob {
        shader_index,
        shader_hash_hex: hex::encode(shader_hash),
        stage: entry.frequency,
        compressed_size: comp_slice.len(),
        uncompressed_size: uncomp_len,
        ue4_header_prefix: raw[..dxbc_off].to_vec(),
        dxbc_bytes: raw[dxbc_off..dxbc_end].to_vec(),
        ue4_resource_counts_tail: raw[dxbc_end..].to_vec(),
    })
}

/// Reads the `ShaderArchiveIndex` from a `PakEntryMeta` without loading the entire 606 MB
/// shader payload into memory when the archive is stored uncompressed and unencrypted.
pub fn read_shader_archive_index_from_pak(
    meta: &PakEntryMeta,
    vfs_bytes_fallback: impl FnOnce() -> Result<Vec<u8>, ExtractError>,
) -> Result<ShaderArchiveIndex, ExtractError> {
    assert!(!meta.virtual_path.is_empty());
    if meta.compression == CompressionKind::None && !meta.encrypted {
        let mut file = File::open(&meta.pak_path)?;
        file.seek(SeekFrom::Start(meta.offset + PAK_INLINE_UNCOMP_HEADER_LEN))?;
        return ShaderArchiveIndex::parse_from_reader(
            &mut file,
            &meta.virtual_path,
            &meta.pak_name,
        );
    }
    let bytes = vfs_bytes_fallback()?;
    assert!(!bytes.is_empty());
    ShaderArchiveIndex::from_bytes(&bytes, &meta.virtual_path, &meta.pak_name)
}

/// Reads and Oodle-decompresses a single `shader_index` from `arch` using direct file seeking
/// when the `.ushaderbytecode` archive is stored uncompressed in its `.pak` file.
pub fn extract_shader_from_pak_entry(
    meta: &PakEntryMeta,
    arch: &ShaderArchiveIndex,
    shader_index: usize,
) -> Result<DecompressedShaderBlob, ExtractError> {
    assert!(shader_index < arch.shader_entries.len());
    assert_eq!(arch.shader_entries.len(), arch.shader_hashes.len());
    let entry = &arch.shader_entries[shader_index];
    let hash = &arch.shader_hashes[shader_index];

    if meta.compression == CompressionKind::None && !meta.encrypted {
        let mut file = File::open(&meta.pak_path)?;
        let abs_off =
            meta.offset + PAK_INLINE_UNCOMP_HEADER_LEN + arch.payload_offset + entry.offset;
        file.seek(SeekFrom::Start(abs_off))?;
        let mut comp_buf = vec![0u8; entry.size as usize];
        file.read_exact(&mut comp_buf)?;
        return decompress_shader_code_blob(shader_index, hash, entry, &comp_buf);
    }

    Err(ExtractError::ShaderFormat(format!(
        "Archive {} requires full-buffer extraction",
        meta.virtual_path
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::WriteBytesExt;

    #[test]
    fn test_shader_archive_roundtrip_uncompressed() {
        let mut buf = Vec::new();
        buf.write_u32::<LittleEndian>(SHADER_ARCHIVE_VERSION_UE427)
            .unwrap();
        buf.write_u32::<LittleEndian>(1).unwrap();
        buf.extend_from_slice(&[0xAA; 20]);
        buf.write_u32::<LittleEndian>(1).unwrap();
        buf.extend_from_slice(&[0xBB; 20]);
        buf.write_u32::<LittleEndian>(1).unwrap();
        buf.write_u32::<LittleEndian>(0).unwrap();
        buf.write_u32::<LittleEndian>(1).unwrap();
        buf.write_u32::<LittleEndian>(0).unwrap();
        buf.write_u32::<LittleEndian>(0).unwrap();

        let mut fake_dxbc = vec![0u8; 40];
        fake_dxbc[8..12].copy_from_slice(b"DXBC");
        fake_dxbc[32..36].copy_from_slice(&32u32.to_le_bytes());

        buf.write_u32::<LittleEndian>(1).unwrap();
        buf.write_u64::<LittleEndian>(0).unwrap();
        buf.write_u32::<LittleEndian>(fake_dxbc.len() as u32)
            .unwrap();
        buf.write_u32::<LittleEndian>(fake_dxbc.len() as u32)
            .unwrap();
        buf.push(3); // Pixel

        buf.write_u32::<LittleEndian>(0).unwrap(); // 0 preload entries
        buf.write_u32::<LittleEndian>(1).unwrap(); // 1 shader index
        buf.write_u32::<LittleEndian>(0).unwrap();
        buf.extend_from_slice(&fake_dxbc);

        let arch = ShaderArchiveIndex::from_bytes(&buf, "test.ushaderbytecode", "test.pak")
            .expect("valid v2 shader archive");
        assert_eq!(arch.shader_map_hashes.len(), 1);
        assert_eq!(arch.shader_entries.len(), 1);
        assert_eq!(arch.shader_indices_for_map(0), vec![0]);

        let p_off = arch.payload_offset as usize;
        let blob = decompress_shader_code_blob(
            0,
            &arch.shader_hashes[0],
            &arch.shader_entries[0],
            &buf[p_off..],
        )
        .expect("uncompressed shader blob");
        assert_eq!(blob.stage, ShaderStage::Pixel);
        assert_eq!(&blob.dxbc_bytes[0..4], b"DXBC");
        assert_eq!(blob.dxbc_bytes.len(), 32);
    }
}
