//! Unreal Engine 4.27 Pak v12 (`0x6B2A56B8`) zero-materialization lazy VFS, SIMD path normalizer,
//! positional (`pread64` / `seek_read`) block reader, and Oodle/Zlib decompressor.
//!
//! # Design (TigerStyle)
//! 1. **Control Plane (`open_game_dir`, `< 0.25 ms`):** Stats `.pak` metadata (`file_size`,
//!    `mtime_nanos`) across `PreMountPak` and `Content/Paks` without opening any `.pak` file
//!    handles, and attaches the flat `/tmp/tartarus-pak-v6-<fingerprint>.idx` index (`PakFastIndex`).
//! 2. **Lazy Per-Archive File Handle (`OnceCell<File>`):** A `.pak` file is opened only when
//!    `read_record` extracts payload blocks from that specific archive.
//! 3. **Data Plane:** Single-path lookups (`get_record`) probe the on-disk hash table in `< 10 µs`,
//!    substring searches (`collect_substring_matches`) filter 4 KB arena chunks via 256-bit SIMD
//!    (`u64x4`) trigram Bloom filters, and payload blocks are extracted via `pread64` into the
//!    `PakVfs` instance's reusable [`ExtractScratch`] workspace.

use std::cell::{OnceCell, RefCell};
use std::fs::{self, File};
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::simd::prelude::*;
use std::time::UNIX_EPOCH;

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;
use serde::Serialize;
use steamlocate::SteamDir;

use crate::aes::Aes256KeySchedule;
use crate::error::ExtractError;
use crate::pak_cache::{PakFastIndex, build_and_save_fast_index, compute_vfs_fingerprint};

/// *Torchlight: Infinite* custom UE 4.27 Pak v12 footer magic (`0x6B2A56B8`).
pub const TLI_PAK_MAGIC: u32 = 0x6B2A_56B8;
/// Standard UE4 Pak footer magic (`0x5A6F12E1`).
pub const UE4_PAK_MAGIC: u32 = 0x5A6F_12E1;
/// Steam App ID for *Torchlight: Infinite*.
pub const TLI_STEAM_APP_ID: u32 = 1_974_050;

const PAK_ARCHIVES_MAX: usize = 4_096;
const PAK_DIR_COUNT_MAX: usize = 262_144;
const PAK_FILE_COUNT_MAX: usize = 1_048_576;
const PAK_BLOCKS_MAX: usize = 65_536;
pub(crate) const PATH_BYTES_MAX: usize = 4_096;
const INLINE_UNCOMP_HDR_LEN: usize = 54;
const SINGLE_READ_SPAN_MAX: usize = 16 * 1024 * 1024;
const FNV_OFFSET_BASIS_64: u64 = 0xCBF2_9CE4_8422_2325;
const FNV_PRIME_64: u64 = 0x0000_0100_0000_01B3;

/// Compression algorithm identifier for a `.pak` entry (`1` byte).
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub enum CompressionKind {
    #[default]
    None = 0,
    Oodle = 1,
    Zlib = 2,
    Unknown = 255,
}

/// Cache-line-aligned (`64` bytes) fixed-size descriptor for a single file in [`PakVfs`].
///
/// Fields are ordered from largest (`u64`) to smallest (`u8`) with explicit zeroed padding
/// to eliminate compiler-inserted holes and prevent buffer bleeds across cache lines.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PakEntryRecord {
    pub offset: u64,
    pub size_compressed: u64,
    pub size_uncompressed: u64,
    pub path_hash: u64,
    pub block_size_bytes: u32,
    pub dir_index: u32,
    pub file_name_off: u32,
    pub file_name_len: u16,
    pub blocks_count: u16,
    pub pak_index: u16,
    pub compression: CompressionKind,
    pub encrypted: bool,
    pub is_premount: bool,
    #[serde(skip)]
    pub reserved: [u8; 11],
}

impl PakEntryRecord {
    #[inline]
    pub const fn is_encrypted(&self) -> bool {
        assert!(self.reserved.len() == 11);
        self.encrypted
    }
}

/// Standalone metadata struct for callers that require owned path strings.
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
struct PakArchiveHeader {
    enc_index: bool,
    index_offset: u64,
    index_size: usize,
    mount_clean: String,
    full_dir_offset: u64,
    full_dir_size: usize,
    encoded_entries_rel_off: usize,
    encoded_entries_size: usize,
    comp_methods: [CompressionKind; 8],
}

/// Lazily opened file handle and metadata for a mounted `.pak` archive.
#[derive(Debug)]
pub struct PakArchive {
    pub pak_path: PathBuf,
    pub pak_name: String,
    pub file_size: u64,
    pub mtime_nanos: u64,
    pub is_premount: bool,
    file: OnceCell<File>,
}

/// Reusable decompression and I/O scratch workspace owned by [`PakVfs`].
pub struct ExtractScratch {
    pub oodle: oozextract::Extractor,
    pub io_buffer: Vec<u8>,
}

impl Default for ExtractScratch {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtractScratch {
    pub fn new() -> Self {
        let scratch = Self {
            oodle: oozextract::Extractor::new(),
            io_buffer: Vec::with_capacity(256 * 1024),
        };
        assert!(scratch.io_buffer.capacity() >= 256 * 1024);
        scratch
    }
}

/// Zero-materialization Virtual File System across all `.pak` archives (`Content/Paks` and
/// `PreMountPak` patches).
pub struct PakVfs {
    pub key: [u8; 32],
    pub key_schedule: Aes256KeySchedule,
    archives: Vec<PakArchive>,
    fast_index: PakFastIndex,
    scratch: RefCell<Option<ExtractScratch>>,
}

const _: () = {
    assert!(TLI_PAK_MAGIC == 0x6B2A_56B8);
    assert!(UE4_PAK_MAGIC == 0x5A6F_12E1);
    assert!(std::mem::size_of::<CompressionKind>() == 1);
    assert!(std::mem::size_of::<PakEntryRecord>() == 64);
    assert!(std::mem::align_of::<PakEntryRecord>() == 64);
};

/// Reads exact bytes from `file` at `offset` without mutating the OS file cursor (`pread64` /
/// `seek_read`).
pub fn read_exact_at_offset(
    file: &File,
    target: &mut [u8],
    offset: u64,
) -> Result<(), ExtractError> {
    assert!(target.len() <= 1024 * 1024 * 1024);
    assert!(offset <= u64::MAX / 2);

    #[cfg(unix)]
    {
        use std::os::unix::fs::FileExt;
        file.read_exact_at(target, offset)?;
    }

    #[cfg(windows)]
    {
        use std::os::windows::fs::FileExt;
        let mut read_total = 0usize;
        let mut iterations = 0usize;
        let iterations_max = target.len() + 1;
        while read_total < target.len() {
            iterations += 1;
            assert!(iterations <= iterations_max);
            let n = file.seek_read(&mut target[read_total..], offset + read_total as u64)?;
            if n == 0 {
                return Err(ExtractError::Io(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "unexpected EOF in positional seek_read",
                )));
            }
            read_total += n;
        }
        assert_eq!(read_total, target.len());
    }

    Ok(())
}

#[inline]
pub(crate) fn fnv1a_hash_64(bytes: &[u8]) -> u64 {
    assert!(bytes.len() <= PATH_BYTES_MAX * 4);
    let mut hash = FNV_OFFSET_BASIS_64;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME_64);
    }
    assert!(hash != 0 || !bytes.is_empty() || hash == FNV_OFFSET_BASIS_64);
    hash
}

/// Converts `source` ASCII uppercase bytes (`A..=Z`) to lowercase (`a..=z`) in `target` using
/// 32-byte `u8x32` SIMD vectors.
#[inline]
pub fn ascii_lowercase_into_simd(source: &[u8], target: &mut [u8]) {
    assert_eq!(source.len(), target.len());
    assert!(source.len() <= 64 * 1024 * 1024);

    let (src_chunks, src_tail) = source.as_chunks::<32>();
    let (dst_chunks, dst_tail) = target.as_chunks_mut::<32>();
    let upper_a = u8x32::splat(b'A');
    let upper_z = u8x32::splat(b'Z');
    let case_bit = u8x32::splat(0x20);

    for (src_32, dst_32) in src_chunks.iter().zip(dst_chunks.iter_mut()) {
        let v = u8x32::from_slice(src_32);
        let mask = v.simd_ge(upper_a) & v.simd_le(upper_z);
        let lower = mask.select(v | case_bit, v);
        lower.copy_to_slice(dst_32);
    }

    for (dst_b, &src_b) in dst_tail.iter_mut().zip(src_tail.iter()) {
        *dst_b = src_b.to_ascii_lowercase();
    }
}

pub(crate) fn split_normalized_dir_and_file(virtual_path: &str) -> (&str, &str) {
    assert!(!virtual_path.is_empty());
    let trimmed = virtual_path.trim_start_matches('/');
    let clean = trimmed
        .strip_prefix("TorchLight/Content/")
        .unwrap_or(trimmed)
        .trim_start_matches('/');
    assert!(!clean.is_empty());
    if let Some(slash_pos) = memchr::memrchr(b'/', clean.as_bytes()) {
        let dir = clean[..slash_pos].trim_end_matches('/');
        let file = &clean[slash_pos + 1..];
        (dir, file)
    } else {
        ("", clean)
    }
}

impl PakVfs {
    /// Discovers all UE4 `.pak` archives under `root` without opening any `.pak` file handles,
    /// and attaches the `/tmp/tartarus-pak-v6-<fingerprint>.idx` flat index cache in `< 0.25 ms`.
    pub fn open_game_dir(root: &Path, key: [u8; 32]) -> Result<Self, ExtractError> {
        assert!(!root.as_os_str().is_empty());
        assert_eq!(key.len(), 32);

        let archives = collect_pak_archives(root)?;
        if archives.is_empty() {
            return Err(ExtractError::PakFormat(format!(
                "No valid UE4 .pak archives found under {}",
                root.display()
            )));
        }
        let key_schedule = Aes256KeySchedule::new(key);
        let fingerprint = compute_vfs_fingerprint(&key, &archives);
        let fast_index = PakFastIndex::try_open(fingerprint)
            .or_else(|| build_and_save_fast_index(fingerprint, &archives, &key_schedule))
            .ok_or_else(|| {
                ExtractError::PakFormat(format!(
                    "Failed to build or open PakFastIndex for {}",
                    root.display()
                ))
            })?;

        assert!(!archives.is_empty());
        Ok(Self {
            key,
            key_schedule,
            archives,
            fast_index,
            scratch: RefCell::new(None),
        })
    }

    /// Returns the slice of mounted `.pak` archives in priority order (newest `PreMountPak` first).
    #[inline]
    pub fn archives(&self) -> &[PakArchive] {
        assert!(self.archives.len() <= PAK_ARCHIVES_MAX);
        &self.archives
    }

    /// Returns the mounted [`PakArchive`] owning `record`.
    #[inline]
    pub fn record_archive<'a>(&'a self, record: &PakEntryRecord) -> &'a PakArchive {
        let idx = record.pak_index as usize;
        assert!(idx < self.archives.len());
        &self.archives[idx]
    }

    /// Returns the `.pak` archive filename (e.g. `"pakchunk0-WindowsNoEditor.pak"`) for `record`.
    #[inline]
    pub fn record_pak_name<'a>(&'a self, record: &PakEntryRecord) -> &'a str {
        let idx = record.pak_index as usize;
        assert!(idx < self.archives.len());
        &self.archives[idx].pak_name
    }

    /// Resolves a `virtual_path` in `< 10 µs` via the disk-backed flat hash table (`PakFastIndex`).
    #[inline]
    pub fn get_record(&self, virtual_path: &str) -> Option<PakEntryRecord> {
        assert!(!virtual_path.is_empty());
        self.fast_index.get_record(virtual_path)
    }

    /// Resolves `virtual_path` in $O(1)$ via the flat index (`pak_index` retained for API
    /// compatibility with companion `.uexp`/`.ubulk` lookups).
    #[inline]
    pub fn get_record_in_archive(
        &self,
        pak_index: u16,
        virtual_path: &str,
    ) -> Option<PakEntryRecord> {
        assert!(!virtual_path.is_empty());
        assert!((pak_index as usize) < self.archives.len());
        self.fast_index.get_record(virtual_path)
    }

    /// Returns `true` if `virtual_path` exists in any mounted `.pak` archive.
    #[inline]
    pub fn contains_path(&self, virtual_path: &str) -> bool {
        assert!(!virtual_path.is_empty());
        self.get_record(virtual_path).is_some()
    }

    /// Hardware SIMD (`u64x4` trigram Bloom filter + `memchr::memmem::Finder`) substring search
    /// across all virtual paths matching `lower_needle`.
    pub fn collect_substring_matches(&self, lower_needle: &str) -> Vec<(String, PakEntryRecord)> {
        assert!(lower_needle.len() <= PATH_BYTES_MAX);
        if lower_needle.is_empty() {
            return self.collect_entries(|_| true, |_, _| true);
        }
        self.fast_index.collect_substring_matches(lower_needle)
    }

    /// Scans directories matching `dir_predicate(dir_lower)` and files matching
    /// `file_predicate(dir_lower, file_lower)`.
    pub fn collect_entries<D, F>(
        &self,
        dir_predicate: D,
        file_predicate: F,
    ) -> Vec<(String, PakEntryRecord)>
    where
        D: FnMut(&str) -> bool,
        F: FnMut(&str, &str) -> bool,
    {
        assert!(!self.archives.is_empty());
        self.fast_index
            .collect_entries(dir_predicate, file_predicate)
    }

    /// Collects `.ushaderbytecode` entries from `UE_game/Content`.
    pub fn collect_shader_archive_entries(&self) -> Vec<(String, PakEntryRecord)> {
        assert!(!self.archives.is_empty());
        self.fast_index.collect_entries(
            |dir| dir == "ue_game/content",
            |_, file| file.ends_with(".ushaderbytecode"),
        )
    }

    /// Reads, decrypts, and decompresses a [`PakEntryRecord`] into a newly allocated `Vec<u8>`.
    pub fn read_record(&self, record: &PakEntryRecord) -> Result<Vec<u8>, ExtractError> {
        assert!((record.pak_index as usize) < self.archives.len());
        let mut out = Vec::with_capacity(record.size_uncompressed as usize);
        self.read_record_into(record, &mut out)?;
        assert_eq!(out.len(), record.size_uncompressed as usize);
        Ok(out)
    }

    /// Borrows this [`PakVfs`] instance's reusable [`ExtractScratch`] workspace (`io_buffer` +
    /// `oodle` decompressor).
    #[inline]
    pub(crate) fn with_scratch<R>(&self, f: impl FnOnce(&mut ExtractScratch) -> R) -> R {
        assert!(!self.archives.is_empty());
        let mut guard = self.scratch.borrow_mut();
        let scratch = guard.get_or_insert_with(ExtractScratch::new);
        assert!(scratch.io_buffer.capacity() >= 256 * 1024);
        f(scratch)
    }

    /// Reads, decrypts, and decompresses a [`PakEntryRecord`] into a caller-supplied `target`
    /// buffer, reusing this [`PakVfs`] instance's [`ExtractScratch`] workspace.
    pub fn read_record_into(
        &self,
        record: &PakEntryRecord,
        target: &mut Vec<u8>,
    ) -> Result<(), ExtractError> {
        let archive = self.record_archive(record);
        assert!(record.offset <= archive.file_size);
        let file = archive.ensure_file()?;
        self.with_scratch(|scratch| {
            extract_record_with_scratch(
                file,
                archive.file_size,
                record,
                &archive.pak_name,
                &self.key_schedule,
                scratch,
                target,
            )
        })
    }

    /// Reads, decrypts, and decompresses a [`PakEntryMeta`] into a newly allocated `Vec<u8>`.
    pub fn read_entry(&self, meta: &PakEntryMeta) -> Result<Vec<u8>, ExtractError> {
        assert!(!meta.virtual_path.is_empty());
        if let Some(rec) = self.get_record(&meta.virtual_path) {
            return self.read_record(&rec);
        }
        extract_pak_entry(meta, &self.key)
    }

    /// Looks up `virtual_path` in the VFS and returns its decrypted, decompressed bytes.
    pub fn read_path(&self, virtual_path: &str) -> Result<Vec<u8>, ExtractError> {
        assert!(!virtual_path.is_empty());
        let record = self
            .get_record(virtual_path)
            .ok_or_else(|| ExtractError::NotFound(virtual_path.trim_start_matches('/').into()))?;
        self.read_record(&record)
    }
}

/// Auto-discovers the *Torchlight: Infinite* installation root via `$TLI_GAME_DIR`,
/// direct Steam paths, or `libraryfolders.vdf` (`1974050`).
pub fn discover_game_dir() -> Option<PathBuf> {
    if let Ok(val) = std::env::var("TLI_GAME_DIR") {
        let p = PathBuf::from(val);
        if p.exists() {
            assert!(!p.as_os_str().is_empty());
            return Some(p);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        for rel in [
            ".local/share/Steam/steamapps/common/Torchlight Infinite",
            ".steam/steam/steamapps/common/Torchlight Infinite",
        ] {
            let candidate = Path::new(&home).join(rel);
            if candidate.is_dir() {
                assert!(!candidate.as_os_str().is_empty());
                return Some(candidate);
            }
        }
    }
    let steam_dir = SteamDir::locate().ok()?;
    let (app, library) = steam_dir.find_app(TLI_STEAM_APP_ID).ok()??;
    let path = library.resolve_app_dir(&app);
    assert!(!path.as_os_str().is_empty());
    path.exists().then_some(path)
}

fn make_lazy_pak_archive(pak_path: PathBuf, is_premount: bool) -> Option<PakArchive> {
    assert!(!pak_path.as_os_str().is_empty());
    let meta = fs::metadata(&pak_path).ok()?;
    let file_size = meta.len();
    if file_size < 240 {
        return None;
    }
    let mtime_nanos = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_nanos() as u64);
    let pak_name = pak_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    assert!(!pak_name.is_empty());

    Some(PakArchive {
        pak_path,
        pak_name,
        file_size,
        mtime_nanos,
        is_premount,
        file: OnceCell::new(),
    })
}

fn collect_paks_in_flat_dir(dir: &Path, is_premount: bool, out: &mut Vec<PakArchive>) {
    assert!(!dir.as_os_str().is_empty());
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    let mut list: Vec<PathBuf> = rd
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|s| s.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("pak"))
        })
        .collect();
    if is_premount {
        list.sort_by(|a, b| b.cmp(a));
    } else {
        list.sort();
    }
    assert!(list.len() <= PAK_ARCHIVES_MAX);
    for p in list {
        if let Some(arch) = make_lazy_pak_archive(p, is_premount) {
            out.push(arch);
        }
    }
}

fn collect_pak_archives(root: &Path) -> Result<Vec<PakArchive>, ExtractError> {
    assert!(!root.as_os_str().is_empty());
    let mut archives = Vec::new();
    if root.is_file() {
        let is_premount = root.to_string_lossy().contains("PreMountPak");
        if let Some(arch) = make_lazy_pak_archive(root.to_path_buf(), is_premount) {
            archives.push(arch);
        }
        return Ok(archives);
    }

    let premount_candidates = [
        root.join("UE_game/TorchLight/Saved/PersistentDownloadDir/PreMountPak"),
        root.join("TorchLight/Saved/PersistentDownloadDir/PreMountPak"),
    ];
    let base_candidates = [root.join("UE_game/Content/Paks"), root.join("Content/Paks")];

    for pm_dir in &premount_candidates {
        if pm_dir.is_dir() {
            collect_paks_in_flat_dir(pm_dir, true, &mut archives);
            break;
        }
    }
    for base_dir in &base_candidates {
        if base_dir.is_dir() {
            collect_paks_in_flat_dir(base_dir, false, &mut archives);
            break;
        }
    }

    if archives.is_empty() {
        let is_premount = root.to_string_lossy().contains("PreMountPak");
        collect_paks_in_flat_dir(root, is_premount, &mut archives);
    }

    if archives.is_empty() {
        return Err(ExtractError::PakFormat(format!(
            "No .pak files found under {}",
            root.display()
        )));
    }
    assert!(!archives.is_empty());
    Ok(archives)
}

fn read_fstring_slice<'b, 's>(
    cur: &mut Cursor<&'b [u8]>,
    utf16_scratch: &'s mut String,
) -> Result<&'s str, ExtractError>
where
    'b: 's,
{
    let buf = *cur.get_ref();
    assert!((cur.position() as usize) <= buf.len());
    let len = cur.read_i32::<LittleEndian>()?;
    if len == 0 {
        return Ok("");
    }
    let pos = cur.position() as usize;
    if len < 0 {
        let chars = (-len) as usize;
        if chars > PATH_BYTES_MAX {
            return Err(ExtractError::PakFormat(format!(
                "UTF-16 FString length {chars} exceeds limit at {pos}"
            )));
        }
        let byte_len = chars * 2;
        let slice = buf.get(pos..pos + byte_len).ok_or_else(|| {
            ExtractError::PakFormat(format!("Unexpected EOF reading UTF-16 FString at {pos}"))
        })?;
        cur.set_position((pos + byte_len) as u64);
        utf16_scratch.clear();
        for chunk in slice.as_chunks::<2>().0 {
            let code = u16::from_le_bytes(*chunk);
            if code != 0
                && let Some(ch) = char::from_u32(u32::from(code))
            {
                utf16_scratch.push(ch);
            }
        }
        Ok(utf16_scratch.as_str())
    } else {
        let byte_len = len as usize;
        if byte_len > PATH_BYTES_MAX {
            return Err(ExtractError::PakFormat(format!(
                "UTF-8 FString length {byte_len} exceeds limit at {pos}"
            )));
        }
        let slice = buf.get(pos..pos + byte_len).ok_or_else(|| {
            ExtractError::PakFormat(format!("Unexpected EOF reading UTF-8 FString at {pos}"))
        })?;
        cur.set_position((pos + byte_len) as u64);
        let trimmed = slice.strip_suffix(&[0]).unwrap_or(slice);
        std::str::from_utf8(trimmed)
            .map_err(|e| ExtractError::PakFormat(format!("Invalid UTF-8 FString at {pos}: {e}")))
    }
}

struct PakFooterInfo {
    enc_index: bool,
    index_offset: u64,
    index_size: usize,
    comp_methods: [CompressionKind; 8],
}

fn parse_pak_footer(
    pak_path: &Path,
    tail: &[u8],
    file_size: u64,
) -> Result<PakFooterInfo, ExtractError> {
    assert!(tail.len() >= 45);
    assert!(file_size >= tail.len() as u64);

    let mut magic_pos: Option<(usize, bool)> = None;
    for i in (1..=tail.len().saturating_sub(45)).rev() {
        let magic = LittleEndian::read_u32(&tail[i..i + 4]);
        if magic == TLI_PAK_MAGIC {
            magic_pos = Some((i, true));
            break;
        } else if magic == UE4_PAK_MAGIC {
            magic_pos = Some((i, false));
            break;
        }
    }
    let Some((pos, is_tli_v12)) = magic_pos else {
        return Err(ExtractError::PakFormat(format!(
            "Pak magic not found in {}",
            pak_path.display()
        )));
    };

    let enc_index = tail[pos - 1] != 0;
    let version = LittleEndian::read_u32(&tail[pos + 4..]);
    let extra_shift = if is_tli_v12 || version >= 12 { 1 } else { 0 };
    let index_offset = LittleEndian::read_u64(&tail[pos + 8 + extra_shift..]);
    let index_size = LittleEndian::read_u64(&tail[pos + 16 + extra_shift..]) as usize;

    let comp_table_start = pos + 44 + extra_shift;
    let mut comp_methods = [CompressionKind::None; 8];
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
        let name = &slice[..nul];
        comp_methods[c_idx + 1] = if name.eq_ignore_ascii_case(b"Oodle") {
            CompressionKind::Oodle
        } else if name.eq_ignore_ascii_case(b"Zlib") {
            CompressionKind::Zlib
        } else {
            CompressionKind::Unknown
        };
    }

    if index_offset.saturating_add(index_size as u64) > file_size {
        return Err(ExtractError::PakFormat(format!(
            "Index out of bounds in {}",
            pak_path.display()
        )));
    }
    Ok(PakFooterInfo {
        enc_index,
        index_offset,
        index_size,
        comp_methods,
    })
}

fn read_pak_archive_header(
    pak_path: &Path,
    file: &File,
    file_size: u64,
    key_schedule: &Aes256KeySchedule,
) -> Result<Option<PakArchiveHeader>, ExtractError> {
    assert!(!pak_path.as_os_str().is_empty());
    if file_size < 240 {
        return Ok(None);
    }

    let tail_len = file_size.min(300) as usize;
    let mut tail = [0u8; 300];
    read_exact_at_offset(file, &mut tail[..tail_len], file_size - tail_len as u64)?;
    let footer = parse_pak_footer(pak_path, &tail[..tail_len], file_size)?;
    if footer.index_size < 32 {
        return Ok(None);
    }

    let hdr_probe_len = footer.index_size.min(256) & !15;
    if hdr_probe_len < 32 {
        return Ok(None);
    }
    let mut hdr_buf = [0u8; 256];
    read_exact_at_offset(file, &mut hdr_buf[..hdr_probe_len], footer.index_offset)?;
    if footer.enc_index {
        key_schedule.decrypt_in_place(&mut hdr_buf[..hdr_probe_len]);
    }

    let mut cur = Cursor::new(&hdr_buf[..hdr_probe_len]);
    let mut utf16_mp = String::new();
    let mount_point = read_fstring_slice(&mut cur, &mut utf16_mp)?;
    let mount_clean = mount_point
        .trim_start_matches("../")
        .trim_start_matches('/')
        .trim_start_matches("TorchLight/Content/")
        .trim_start_matches('/')
        .to_string();

    let _num_entries = cur.read_i32::<LittleEndian>()?;
    let _path_hash_seed = cur.read_u64::<LittleEndian>()?;
    if cur.read_i32::<LittleEndian>()? != 0 {
        cur.set_position(cur.position() + 36);
    }
    if cur.read_i32::<LittleEndian>()? == 0 {
        return Ok(None);
    }
    let full_dir_offset = cur.read_u64::<LittleEndian>()?;
    let full_dir_size = cur.read_u64::<LittleEndian>()? as usize;
    cur.set_position(cur.position() + 20);
    let encoded_entries_size_raw = cur.read_i32::<LittleEndian>()?;
    let encoded_entries_rel_off = cur.position() as usize;
    if full_dir_size == 0
        || full_dir_offset.saturating_add(full_dir_size as u64) > file_size
        || encoded_entries_size_raw < 0
        || encoded_entries_rel_off.saturating_add(encoded_entries_size_raw as usize)
            > footer.index_size
    {
        return Err(ExtractError::PakFormat(format!(
            "Invalid Primary Index header bounds in {}",
            pak_path.display()
        )));
    }

    Ok(Some(PakArchiveHeader {
        enc_index: footer.enc_index,
        index_offset: footer.index_offset,
        index_size: footer.index_size,
        mount_clean,
        full_dir_offset,
        full_dir_size,
        encoded_entries_rel_off,
        encoded_entries_size: encoded_entries_size_raw as usize,
        comp_methods: footer.comp_methods,
    }))
}

fn normalize_dir_into(
    dir_orig: &mut Vec<u8>,
    dir_lower: &mut Vec<u8>,
    mount_clean: &str,
    dir_name: &str,
) -> u64 {
    dir_orig.clear();
    dir_lower.clear();
    let dir_clean = dir_name.trim_matches('/');

    for segment in [mount_clean.as_bytes(), dir_clean.as_bytes()] {
        for &b in segment {
            if b == b'/' && (dir_orig.is_empty() || *dir_orig.last().unwrap() == b'/') {
                continue;
            }
            dir_orig.push(b);
        }
    }
    while dir_orig.last() == Some(&b'/') {
        dir_orig.pop();
    }
    dir_orig.truncate(PATH_BYTES_MAX);
    dir_lower.resize(dir_orig.len(), 0);
    ascii_lowercase_into_simd(dir_orig, dir_lower);
    assert_eq!(dir_orig.len(), dir_lower.len());
    fnv1a_hash_64(dir_lower)
}

impl PakArchive {
    /// Lazily opens and returns the OS file handle for this `.pak` archive.
    pub fn ensure_file(&self) -> Result<&File, ExtractError> {
        assert!(!self.pak_path.as_os_str().is_empty());
        if let Some(f) = self.file.get() {
            return Ok(f);
        }
        let opened = File::open(&self.pak_path)?;
        let _ = self.file.set(opened);
        assert!(self.file.get().is_some());
        Ok(self.file.get().unwrap())
    }

    pub(crate) fn scan_all_entries_for_cache<F>(
        &self,
        pak_idx: u16,
        key_schedule: &Aes256KeySchedule,
        cb: &mut F,
    ) where
        F: FnMut((&str, &str, u64), (&str, &str), PakEntryRecord),
    {
        assert!(self.file_size >= 240);
        let Ok(file) = self.ensure_file() else {
            return;
        };
        let Ok(Some(hdr)) =
            read_pak_archive_header(&self.pak_path, file, self.file_size, key_schedule)
        else {
            return;
        };
        let mut dir_buf = vec![0u8; hdr.full_dir_size];
        if read_exact_at_offset(file, &mut dir_buf, hdr.full_dir_offset).is_err() {
            return;
        }
        if hdr.enc_index {
            key_schedule.decrypt_in_place(&mut dir_buf);
        }
        let mut idx_buf = vec![0u8; hdr.index_size];
        if read_exact_at_offset(file, &mut idx_buf, hdr.index_offset).is_err() {
            return;
        }
        if hdr.enc_index {
            key_schedule.decrypt_in_place(&mut idx_buf);
        }
        let enc_end = (hdr.encoded_entries_rel_off + hdr.encoded_entries_size).min(idx_buf.len());
        let enc_entries = &idx_buf[hdr.encoded_entries_rel_off..enc_end];

        scan_raw_dir_buffer(&dir_buf, enc_entries, &hdr, self.is_premount, pak_idx, cb);
    }
}

fn scan_raw_dir_buffer<F>(
    dir_buf: &[u8],
    enc_entries: &[u8],
    hdr: &PakArchiveHeader,
    is_premount: bool,
    pak_idx: u16,
    cb: &mut F,
) where
    F: FnMut((&str, &str, u64), (&str, &str), PakEntryRecord),
{
    assert!(!dir_buf.is_empty());
    let mut cur = Cursor::new(dir_buf);
    let Ok(dir_count_raw) = cur.read_i32::<LittleEndian>() else {
        return;
    };
    if !(0..=PAK_DIR_COUNT_MAX as i32).contains(&dir_count_raw) {
        return;
    }
    let mut utf16_dir = String::new();
    let mut utf16_file = String::new();
    let mut dir_orig_buf = Vec::with_capacity(128);
    let mut dir_lower_buf = Vec::with_capacity(128);
    let mut file_lower_buf = [0u8; PATH_BYTES_MAX];

    for _ in 0..dir_count_raw {
        let Ok(dir_name) = read_fstring_slice(&mut cur, &mut utf16_dir) else {
            break;
        };
        let Ok(file_count_raw) = cur.read_i32::<LittleEndian>() else {
            break;
        };
        if !(0..=PAK_FILE_COUNT_MAX as i32).contains(&file_count_raw) {
            break;
        }
        let dir_hash = normalize_dir_into(
            &mut dir_orig_buf,
            &mut dir_lower_buf,
            &hdr.mount_clean,
            dir_name,
        );
        let dir_orig = std::str::from_utf8(&dir_orig_buf).unwrap_or("");
        let dir_lower = std::str::from_utf8(&dir_lower_buf).unwrap_or("");

        for _ in 0..file_count_raw {
            let Ok(file_orig) = read_fstring_slice(&mut cur, &mut utf16_file) else {
                return;
            };
            let Ok(enc_off) = cur.read_i32::<LittleEndian>() else {
                return;
            };
            if enc_off < 0 || file_orig.is_empty() || file_orig.len() > PATH_BYTES_MAX {
                continue;
            }
            let flen = file_orig.len();
            ascii_lowercase_into_simd(file_orig.as_bytes(), &mut file_lower_buf[..flen]);
            let file_lower = std::str::from_utf8(&file_lower_buf[..flen]).unwrap_or("");
            if let Ok(rec) = decode_compact_pak_record(
                enc_entries,
                enc_off as usize,
                &hdr.comp_methods,
                is_premount,
                pak_idx,
            ) {
                cb(
                    (dir_orig, dir_lower, dir_hash),
                    (file_orig, file_lower),
                    rec,
                );
            }
        }
    }
}

/// Decodes a UE4 compact `FPakEntry` bitfield into a 64-byte [`PakEntryRecord`].
fn decode_compact_pak_record(
    encoded_entries: &[u8],
    offset_in_encoded: usize,
    comp_methods: &[CompressionKind; 8],
    is_premount: bool,
    pak_index: u16,
) -> Result<PakEntryRecord, ExtractError> {
    assert!(!encoded_entries.is_empty());
    let slice = encoded_entries.get(offset_in_encoded..).ok_or_else(|| {
        ExtractError::PakFormat(format!("Invalid encoded entry offset {offset_in_encoded}"))
    })?;
    let mut cur = Cursor::new(slice);
    let bitfield = cur.read_u32::<LittleEndian>()?;

    let offset_32 = (bitfield & (1 << 31)) != 0;
    let uncomp_32 = (bitfield & (1 << 30)) != 0;
    let size_32 = (bitfield & (1 << 29)) != 0;
    let comp_method_idx = ((bitfield >> 23) & 0x3F) as usize;
    let encrypted = (bitfield & (1 << 22)) != 0;
    let comp_blocks_count = ((bitfield >> 6) & 0xFFFF) as u16;
    let block_size_raw = bitfield & 0x3F;
    let block_size_bytes = if block_size_raw == 0x3F {
        cur.read_u32::<LittleEndian>()?
    } else {
        block_size_raw << 11
    };

    let offset = if offset_32 {
        u64::from(cur.read_u32::<LittleEndian>()?)
    } else {
        cur.read_u64::<LittleEndian>()?
    };
    let size_uncompressed = if uncomp_32 {
        u64::from(cur.read_u32::<LittleEndian>()?)
    } else {
        cur.read_u64::<LittleEndian>()?
    };
    let size_compressed = if comp_method_idx != 0 {
        if size_32 {
            u64::from(cur.read_u32::<LittleEndian>()?)
        } else {
            cur.read_u64::<LittleEndian>()?
        }
    } else {
        size_uncompressed
    };
    let compression = comp_methods
        .get(comp_method_idx)
        .copied()
        .unwrap_or(CompressionKind::Unknown);

    assert!(size_uncompressed <= 64 * 1024 * 1024 * 1024);
    Ok(PakEntryRecord {
        offset,
        size_compressed,
        size_uncompressed,
        path_hash: 0,
        block_size_bytes,
        dir_index: 0,
        file_name_off: 0,
        file_name_len: 0,
        blocks_count: comp_blocks_count,
        pak_index,
        compression,
        encrypted,
        is_premount,
        reserved: [0u8; 11],
    })
}

/// Reads and parses the directory index of a single `.pak` file into a `Vec<PakEntryMeta>`.
pub fn read_pak_index(
    pak_path: &Path,
    key: &[u8; 32],
    is_premount: bool,
) -> Result<Vec<PakEntryMeta>, ExtractError> {
    assert!(!pak_path.as_os_str().is_empty());
    assert_eq!(key.len(), 32);
    let key_schedule = Aes256KeySchedule::new(*key);
    let Some(archive) = make_lazy_pak_archive(pak_path.to_path_buf(), is_premount) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    archive.scan_all_entries_for_cache(
        0,
        &key_schedule,
        &mut |(dir_orig, _, _), (file_orig, _), rec| {
            let virtual_path = if dir_orig.is_empty() {
                file_orig.to_string()
            } else {
                format!("{dir_orig}/{file_orig}")
            };
            out.push(PakEntryMeta {
                pak_path: archive.pak_path.clone(),
                pak_name: archive.pak_name.clone(),
                virtual_path,
                offset: rec.offset,
                comp_size: rec.size_compressed,
                uncomp_size: rec.size_uncompressed,
                compression: rec.compression,
                encrypted: rec.encrypted,
                comp_block_size: rec.block_size_bytes,
                is_premount,
            });
        },
    );
    Ok(out)
}

/// Standalone helper to extract a [`PakEntryMeta`] by opening its `.pak` file directly.
pub fn extract_pak_entry(meta: &PakEntryMeta, key: &[u8; 32]) -> Result<Vec<u8>, ExtractError> {
    assert!(!meta.virtual_path.is_empty());
    assert_eq!(key.len(), 32);

    let file = File::open(&meta.pak_path)?;
    let file_size = file.metadata()?.len();
    let key_schedule = Aes256KeySchedule::new(*key);
    let record = PakEntryRecord {
        offset: meta.offset,
        size_compressed: meta.comp_size,
        size_uncompressed: meta.uncomp_size,
        path_hash: 0,
        block_size_bytes: meta.comp_block_size,
        dir_index: 0,
        file_name_off: 0,
        file_name_len: 0,
        blocks_count: 1,
        pak_index: 0,
        compression: meta.compression,
        encrypted: meta.encrypted,
        is_premount: meta.is_premount,
        reserved: [0u8; 11],
    };

    let mut out = Vec::with_capacity(meta.uncomp_size as usize);
    let mut scratch = ExtractScratch::new();
    extract_record_with_scratch(
        &file,
        file_size,
        &record,
        &meta.virtual_path,
        &key_schedule,
        &mut scratch,
        &mut out,
    )?;
    assert_eq!(out.len(), meta.uncomp_size as usize);
    Ok(out)
}

fn decompress_single_block_into(
    compression: CompressionKind,
    comp_slice: &[u8],
    uncomp_slice: &mut [u8],
    virtual_path: &str,
    oodle: &mut oozextract::Extractor,
) -> Result<(), ExtractError> {
    assert!(!comp_slice.is_empty());
    assert!(!uncomp_slice.is_empty());

    match compression {
        CompressionKind::Oodle => {
            let written = oodle
                .read_from_slice(comp_slice, uncomp_slice)
                .map_err(|e| {
                    ExtractError::Decompress(format!("Oodle failed on {virtual_path}: {e:?}"))
                })?;
            if written != uncomp_slice.len() {
                return Err(ExtractError::Decompress(format!(
                    "Oodle short write on {virtual_path}: expected {}, got {written}",
                    uncomp_slice.len()
                )));
            }
        }
        CompressionKind::Zlib => {
            let mut decoder = ZlibDecoder::new(comp_slice);
            decoder.read_exact(uncomp_slice).map_err(|e| {
                ExtractError::Decompress(format!("Zlib failed on {virtual_path}: {e}"))
            })?;
        }
        CompressionKind::None | CompressionKind::Unknown => {
            let n = comp_slice.len().min(uncomp_slice.len());
            uncomp_slice[..n].copy_from_slice(&comp_slice[..n]);
        }
    }
    Ok(())
}

fn extract_uncompressed_record(
    file: &File,
    record: &PakEntryRecord,
    key_schedule: &Aes256KeySchedule,
    scratch: &mut ExtractScratch,
    target: &mut Vec<u8>,
) -> Result<(), ExtractError> {
    let uncomp_len = record.size_uncompressed as usize;
    let data_start = record.offset + INLINE_UNCOMP_HDR_LEN as u64;
    assert!(data_start >= record.offset);

    if !record.encrypted {
        target.clear();
        target.resize(uncomp_len, 0);
        read_exact_at_offset(file, target, data_start)?;
        assert_eq!(target.len(), uncomp_len);
        return Ok(());
    }

    let read_len = (uncomp_len + 15) & !15;
    scratch.io_buffer.resize(read_len, 0);
    read_exact_at_offset(file, &mut scratch.io_buffer[..read_len], data_start)?;
    key_schedule.decrypt_in_place(&mut scratch.io_buffer[..read_len]);

    target.clear();
    target.extend_from_slice(&scratch.io_buffer[..uncomp_len]);
    assert_eq!(target.len(), uncomp_len);
    Ok(())
}

fn try_decompress_from_prefetched_window(
    record: &PakEntryRecord,
    virtual_path: &str,
    key_schedule: &Aes256KeySchedule,
    scratch: &mut ExtractScratch,
    target: &mut Vec<u8>,
) -> Option<Result<(), ExtractError>> {
    assert!(scratch.io_buffer.len() >= 58);
    let num_blocks = LittleEndian::read_i32(&scratch.io_buffer[49..53]);
    if !(1..=PAK_BLOCKS_MAX as i32).contains(&num_blocks) {
        return None;
    }
    let nb = num_blocks as usize;
    let tail_off = 53 + nb * 16;
    if tail_off + 5 > scratch.io_buffer.len() {
        return None;
    }

    let is_enc = (scratch.io_buffer[tail_off] != 0) || record.encrypted;
    let inline_cbs =
        LittleEndian::read_u32(&scratch.io_buffer[tail_off + 1..tail_off + 5]) as usize;
    let uncomp_total = record.size_uncompressed as usize;
    let default_block_size = if inline_cbs > 0 {
        inline_cbs
    } else if record.block_size_bytes > 0 {
        record.block_size_bytes as usize
    } else {
        uncomp_total
    };

    for idx in 0..nb {
        let b_off = 53 + idx * 16;
        let b_start = LittleEndian::read_u64(&scratch.io_buffer[b_off..b_off + 8]);
        let b_end = LittleEndian::read_u64(&scratch.io_buffer[b_off + 8..b_off + 16]);
        if b_end <= b_start {
            return None;
        }
        let c_len = (b_end - b_start) as usize;
        let read_len = if is_enc { (c_len + 15) & !15 } else { c_len };
        let abs_start = if b_start >= record.offset {
            b_start
        } else {
            record.offset + b_start
        };
        let rel_start = abs_start.checked_sub(record.offset)? as usize;
        if rel_start.checked_add(read_len)? > scratch.io_buffer.len() {
            return None;
        }
    }

    target.clear();
    target.resize(uncomp_total, 0);
    let mut written_total = 0usize;

    for idx in 0..nb {
        let b_off = 53 + idx * 16;
        let b_start = LittleEndian::read_u64(&scratch.io_buffer[b_off..b_off + 8]);
        let b_end = LittleEndian::read_u64(&scratch.io_buffer[b_off + 8..b_off + 16]);
        let c_len = (b_end - b_start) as usize;
        let read_len = if is_enc { (c_len + 15) & !15 } else { c_len };
        let abs_start = if b_start >= record.offset {
            b_start
        } else {
            record.offset + b_start
        };
        let rel_start = (abs_start - record.offset) as usize;
        let comp_buf = &mut scratch.io_buffer[rel_start..rel_start + read_len];
        if is_enc {
            key_schedule.decrypt_in_place(comp_buf);
        }

        let u_len = (uncomp_total - written_total).min(default_block_size);
        if u_len == 0 {
            break;
        }
        let res = decompress_single_block_into(
            record.compression,
            &comp_buf[..c_len],
            &mut target[written_total..written_total + u_len],
            virtual_path,
            &mut scratch.oodle,
        );
        if let Err(err) = res {
            return Some(Err(err));
        }
        written_total += u_len;
    }

    assert_eq!(written_total, uncomp_total);
    Some(Ok(()))
}

fn extract_record_with_scratch(
    file: &File,
    file_size: u64,
    record: &PakEntryRecord,
    virtual_path: &str,
    key_schedule: &Aes256KeySchedule,
    scratch: &mut ExtractScratch,
    target: &mut Vec<u8>,
) -> Result<(), ExtractError> {
    assert!(!virtual_path.is_empty());
    assert!(record.offset <= file_size);

    if record.compression == CompressionKind::None {
        return extract_uncompressed_record(file, record, key_schedule, scratch, target);
    }

    let est_blocks = (record.blocks_count as usize).max(1);
    let est_hdr_len = 58 + est_blocks * 16;
    let est_span = (est_hdr_len + (record.size_compressed as usize) + est_blocks * 16 + 64)
        .min((file_size - record.offset) as usize);

    if (58..=SINGLE_READ_SPAN_MAX).contains(&est_span) {
        scratch.io_buffer.resize(est_span, 0);
        read_exact_at_offset(file, &mut scratch.io_buffer[..est_span], record.offset)?;
        if LittleEndian::read_u32(&scratch.io_buffer[25..29]) == 0 {
            return extract_uncompressed_record(file, record, key_schedule, scratch, target);
        }
        if let Some(res) = try_decompress_from_prefetched_window(
            record,
            virtual_path,
            key_schedule,
            scratch,
            target,
        ) {
            return res;
        }
    }

    extract_compressed_fallback(file, record, virtual_path, key_schedule, scratch, target)
}

fn extract_compressed_fallback(
    file: &File,
    record: &PakEntryRecord,
    virtual_path: &str,
    key_schedule: &Aes256KeySchedule,
    scratch: &mut ExtractScratch,
    target: &mut Vec<u8>,
) -> Result<(), ExtractError> {
    assert!(record.size_uncompressed > 0);
    let mut hdr_prefix = [0u8; 53];
    read_exact_at_offset(file, &mut hdr_prefix, record.offset)?;

    let comp_method = LittleEndian::read_u32(&hdr_prefix[25..29]);
    if comp_method == 0 {
        return extract_uncompressed_record(file, record, key_schedule, scratch, target);
    }
    let num_blocks = LittleEndian::read_i32(&hdr_prefix[49..53]);
    if !(1..=PAK_BLOCKS_MAX as i32).contains(&num_blocks) {
        return Err(ExtractError::PakFormat(format!(
            "Invalid block count {num_blocks} in inline header for {virtual_path}"
        )));
    }
    let nb = num_blocks as usize;
    let mut blocks_raw = vec![0u8; nb * 16 + 5];
    read_exact_at_offset(file, &mut blocks_raw, record.offset + 53)?;

    let tail_off = nb * 16;
    let is_enc = (blocks_raw[tail_off] != 0) || record.encrypted;
    let inline_cbs = LittleEndian::read_u32(&blocks_raw[tail_off + 1..tail_off + 5]) as usize;
    let uncomp_total = record.size_uncompressed as usize;
    let default_block_size = if inline_cbs > 0 {
        inline_cbs
    } else if record.block_size_bytes > 0 {
        record.block_size_bytes as usize
    } else {
        uncomp_total
    };

    target.clear();
    target.resize(uncomp_total, 0);
    let mut written_total = 0usize;

    for idx in 0..nb {
        let b_start = LittleEndian::read_u64(&blocks_raw[idx * 16..idx * 16 + 8]);
        let b_end = LittleEndian::read_u64(&blocks_raw[idx * 16 + 8..idx * 16 + 16]);
        if b_end <= b_start {
            return Err(ExtractError::PakFormat(format!(
                "Invalid block range {b_start}..{b_end} in {virtual_path}"
            )));
        }
        let c_len = (b_end - b_start) as usize;
        let read_len = if is_enc { (c_len + 15) & !15 } else { c_len };
        let abs_start = if b_start >= record.offset {
            b_start
        } else {
            record.offset + b_start
        };
        scratch.io_buffer.resize(read_len, 0);
        read_exact_at_offset(file, &mut scratch.io_buffer[..read_len], abs_start)?;
        if is_enc {
            key_schedule.decrypt_in_place(&mut scratch.io_buffer[..read_len]);
        }

        let u_len = (uncomp_total - written_total).min(default_block_size);
        if u_len == 0 {
            break;
        }
        decompress_single_block_into(
            record.compression,
            &scratch.io_buffer[..c_len],
            &mut target[written_total..written_total + u_len],
            virtual_path,
            &mut scratch.oodle,
        )?;
        written_total += u_len;
    }

    assert_eq!(written_total, uncomp_total);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_ascii_lowercase_and_dir_normalization() {
        let input = b"UE_game/Content/Art/UIV2/S16Gameplay/UIEffect/S16BoardCell/UI_MI_V2_S16BoardCell_101.uasset";
        let mut out = vec![0u8; input.len()];
        ascii_lowercase_into_simd(input, &mut out);
        assert_eq!(&out, &input.to_ascii_lowercase());

        let mut dir_orig = Vec::new();
        let mut dir_lower = Vec::new();
        let hash = normalize_dir_into(
            &mut dir_orig,
            &mut dir_lower,
            "UE_game/Content/",
            "/Art//UIV2/S16Gameplay/",
        );
        assert_eq!(
            std::str::from_utf8(&dir_orig).unwrap(),
            "UE_game/Content/Art/UIV2/S16Gameplay"
        );
        assert_eq!(
            std::str::from_utf8(&dir_lower).unwrap(),
            "ue_game/content/art/uiv2/s16gameplay"
        );
        assert_ne!(hash, 0);

        let (d, f) = split_normalized_dir_and_file(
            "/UE_game/Content/Art/UIV2/S16Gameplay/S16BoardCell.uasset",
        );
        assert_eq!(d, "UE_game/Content/Art/UIV2/S16Gameplay");
        assert_eq!(f, "S16BoardCell.uasset");
    }
}
