//! Pure-Rust decoders for Unreal Engine 4 texture formats (`PF_BC7`, `PF_DXT5`/`BC3`,
//! `PF_DXT1`/`BC1`, `PF_BC4`, `PF_BC5`, `PF_B8G8R8A8`, `PF_R8G8B8A8`, `PF_G8`) and RGBA8 PNG encoder.

use std::io::Write;

use flate2::Compression;
use flate2::write::ZlibEncoder;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum PixelFormat {
    BC7,
    BC3, // DXT5
    BC1, // DXT1
    BC4,
    BC5,
    B8G8R8A8,
    R8G8B8A8,
    G8,
}

impl PixelFormat {
    pub fn from_ue_name(s: &str) -> Option<Self> {
        match s {
            "PF_BC7" => Some(Self::BC7),
            "PF_DXT5" => Some(Self::BC3),
            "PF_DXT1" => Some(Self::BC1),
            "PF_BC4" => Some(Self::BC4),
            "PF_BC5" => Some(Self::BC5),
            "PF_B8G8R8A8" => Some(Self::B8G8R8A8),
            "PF_R8G8B8A8" => Some(Self::R8G8B8A8),
            "PF_G8" => Some(Self::G8),
            _ => None,
        }
    }

    pub fn from_name(s: &str) -> Option<Self> {
        Self::from_ue_name(s)
    }

    pub fn byte_size_for_dimensions(self, width: usize, height: usize) -> usize {
        let bx = (width + 3) / 4;
        let by = (height + 3) / 4;
        match self {
            Self::BC7 | Self::BC3 | Self::BC5 => bx * by * 16,
            Self::BC1 | Self::BC4 => bx * by * 8,
            Self::B8G8R8A8 | Self::R8G8B8A8 => width * height * 4,
            Self::G8 => width * height,
        }
    }

    pub fn mip_byte_size(self, width: usize, height: usize) -> usize {
        self.byte_size_for_dimensions(width, height)
    }
}

pub fn decode_mip_to_rgba8(
    src: &[u8],
    width: usize,
    height: usize,
    fmt: PixelFormat,
) -> Result<Vec<u8>, String> {
    decode_texture_to_rgba8(fmt, width, height, src)
}

pub fn decode_texture_to_rgba8(
    fmt: PixelFormat,
    width: usize,
    height: usize,
    src: &[u8],
) -> Result<Vec<u8>, String> {
    let expected = fmt.byte_size_for_dimensions(width, height);
    if src.len() < expected {
        return Err(format!(
            "Source buffer too small for {fmt:?} {width}x{height}: need {expected} bytes, got {}",
            src.len()
        ));
    }
    let mut out = vec![0u8; width * height * 4];
    match fmt {
        PixelFormat::B8G8R8A8 => {
            for (dst, px) in out.chunks_exact_mut(4).zip(src.chunks_exact(4)) {
                dst[0] = px[2];
                dst[1] = px[1];
                dst[2] = px[0];
                dst[3] = px[3];
            }
        }
        PixelFormat::R8G8B8A8 => {
            out.copy_from_slice(&src[..width * height * 4]);
        }
        PixelFormat::G8 => {
            for (dst, &g) in out.chunks_exact_mut(4).zip(src.iter().take(width * height)) {
                dst[0] = g;
                dst[1] = g;
                dst[2] = g;
                dst[3] = 255;
            }
        }
        PixelFormat::BC1 => decode_bc1(width, height, src, &mut out),
        PixelFormat::BC3 => decode_bc3(width, height, src, &mut out),
        PixelFormat::BC4 => decode_bc4(width, height, src, &mut out),
        PixelFormat::BC5 => decode_bc5(width, height, src, &mut out),
        PixelFormat::BC7 => decode_bc7(width, height, src, &mut out),
    }
    Ok(out)
}

#[inline]
fn rgb565_to_rgb8(c: u16) -> [u8; 3] {
    let r5 = ((c >> 11) & 0x1F) as u8;
    let g6 = ((c >> 5) & 0x3F) as u8;
    let b5 = (c & 0x1F) as u8;
    [
        (r5 << 3) | (r5 >> 2),
        (g6 << 2) | (g6 >> 4),
        (b5 << 3) | (b5 >> 2),
    ]
}

fn decode_bc1_color_block(block: &[u8; 8], allow_transparent: bool) -> [[u8; 4]; 16] {
    let c0 = u16::from_le_bytes([block[0], block[1]]);
    let c1 = u16::from_le_bytes([block[2], block[3]]);
    let [r0, g0, b0] = rgb565_to_rgb8(c0);
    let [r1, g1, b1] = rgb565_to_rgb8(c1);
    let mut palette = [[0u8; 4]; 4];
    palette[0] = [r0, g0, b0, 255];
    palette[1] = [r1, g1, b1, 255];
    if c0 > c1 || !allow_transparent {
        palette[2] = [
            ((2 * r0 as u16 + r1 as u16 + 1) / 3) as u8,
            ((2 * g0 as u16 + g1 as u16 + 1) / 3) as u8,
            ((2 * b0 as u16 + b1 as u16 + 1) / 3) as u8,
            255,
        ];
        palette[3] = [
            ((r0 as u16 + 2 * r1 as u16 + 1) / 3) as u8,
            ((g0 as u16 + 2 * g1 as u16 + 1) / 3) as u8,
            ((b0 as u16 + 2 * b1 as u16 + 1) / 3) as u8,
            255,
        ];
    } else {
        palette[2] = [
            ((r0 as u16 + r1 as u16) / 2) as u8,
            ((g0 as u16 + g1 as u16) / 2) as u8,
            ((b0 as u16 + b1 as u16) / 2) as u8,
            255,
        ];
        palette[3] = [0, 0, 0, 0];
    }
    let idx_bits = u32::from_le_bytes([block[4], block[5], block[6], block[7]]);
    let mut out = [[0u8; 4]; 16];
    for i in 0..16 {
        let code = ((idx_bits >> (i * 2)) & 3) as usize;
        out[i] = palette[code];
    }
    out
}

fn decode_bc4_channel_block(block: &[u8; 8]) -> [u8; 16] {
    let a0 = block[0];
    let a1 = block[1];
    let mut pal = [0u8; 8];
    pal[0] = a0;
    pal[1] = a1;
    if a0 > a1 {
        for i in 1..7 {
            pal[i + 1] = (((7 - i) as u16 * a0 as u16 + i as u16 * a1 as u16 + 3) / 7) as u8;
        }
    } else {
        for i in 1..5 {
            pal[i + 1] = (((5 - i) as u16 * a0 as u16 + i as u16 * a1 as u16 + 2) / 5) as u8;
        }
        pal[6] = 0;
        pal[7] = 255;
    }
    let mut bits = 0u64;
    for i in 0..6 {
        bits |= (block[2 + i] as u64) << (i * 8);
    }
    let mut out = [0u8; 16];
    for i in 0..16 {
        let code = ((bits >> (i * 3)) & 7) as usize;
        out[i] = pal[code];
    }
    out
}

fn write_4x4_block(
    width: usize,
    height: usize,
    bx: usize,
    by: usize,
    pixels: &[[u8; 4]; 16],
    out: &mut [u8],
) {
    for py in 0..4 {
        let y = by * 4 + py;
        if y >= height {
            continue;
        }
        for px in 0..4 {
            let x = bx * 4 + px;
            if x >= width {
                continue;
            }
            let dst = (y * width + x) * 4;
            out[dst..dst + 4].copy_from_slice(&pixels[py * 4 + px]);
        }
    }
}

fn decode_bc1(width: usize, height: usize, src: &[u8], out: &mut [u8]) {
    let blocks_x = (width + 3) / 4;
    let blocks_y = (height + 3) / 4;
    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            let off = (by * blocks_x + bx) * 8;
            let block: &[u8; 8] = src[off..off + 8].try_into().unwrap();
            let pixels = decode_bc1_color_block(block, true);
            write_4x4_block(width, height, bx, by, &pixels, out);
        }
    }
}

fn decode_bc3(width: usize, height: usize, src: &[u8], out: &mut [u8]) {
    let blocks_x = (width + 3) / 4;
    let blocks_y = (height + 3) / 4;
    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            let off = (by * blocks_x + bx) * 16;
            let alpha_blk: &[u8; 8] = src[off..off + 8].try_into().unwrap();
            let color_blk: &[u8; 8] = src[off + 8..off + 16].try_into().unwrap();
            let alphas = decode_bc4_channel_block(alpha_blk);
            let mut pixels = decode_bc1_color_block(color_blk, false);
            for i in 0..16 {
                pixels[i][3] = alphas[i];
            }
            write_4x4_block(width, height, bx, by, &pixels, out);
        }
    }
}

fn decode_bc4(width: usize, height: usize, src: &[u8], out: &mut [u8]) {
    let blocks_x = (width + 3) / 4;
    let blocks_y = (height + 3) / 4;
    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            let off = (by * blocks_x + bx) * 8;
            let r_blk: &[u8; 8] = src[off..off + 8].try_into().unwrap();
            let rs = decode_bc4_channel_block(r_blk);
            let mut pixels = [[0u8; 4]; 16];
            for i in 0..16 {
                pixels[i] = [rs[i], rs[i], rs[i], 255];
            }
            write_4x4_block(width, height, bx, by, &pixels, out);
        }
    }
}

fn decode_bc5(width: usize, height: usize, src: &[u8], out: &mut [u8]) {
    let blocks_x = (width + 3) / 4;
    let blocks_y = (height + 3) / 4;
    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            let off = (by * blocks_x + bx) * 16;
            let r_blk: &[u8; 8] = src[off..off + 8].try_into().unwrap();
            let g_blk: &[u8; 8] = src[off + 8..off + 16].try_into().unwrap();
            let rs = decode_bc4_channel_block(r_blk);
            let gs = decode_bc4_channel_block(g_blk);
            let mut pixels = [[0u8; 4]; 16];
            for i in 0..16 {
                pixels[i] = [rs[i], gs[i], 255, 255];
            }
            write_4x4_block(width, height, bx, by, &pixels, out);
        }
    }
}

// ============================================================================
// Khronos / Direct3D 11 BC7 Decoder (Modes 0..=7)
// ============================================================================

const BC7_WEIGHTS_2: [u8; 4] = [0, 21, 43, 64];
const BC7_WEIGHTS_3: [u8; 8] = [0, 9, 18, 27, 37, 46, 55, 64];
const BC7_WEIGHTS_4: [u8; 16] = [0, 4, 9, 13, 17, 21, 26, 30, 34, 38, 43, 47, 51, 55, 60, 64];

const BC7_PARTITIONS_2: [u16; 64] = [
    0xCCCC, 0x8888, 0xEEEE, 0xECC8, 0xC880, 0xFEEC, 0xFEC8, 0xEC80, 0xC800, 0xFFEC, 0xFE80, 0xE800,
    0xFFE8, 0xFF00, 0xFFF0, 0xF000, 0xF710, 0x008E, 0x7100, 0x08CE, 0x008C, 0x7310, 0x3100, 0x8CCE,
    0x088C, 0x3110, 0x6666, 0x366C, 0x17E8, 0x0FF0, 0x718E, 0x399C, 0xAAAA, 0xF0F0, 0x5A5A, 0x33CC,
    0x3C3C, 0x55AA, 0x9696, 0xA55A, 0x73CE, 0x13C8, 0x324C, 0x3BDC, 0x6996, 0xC33C, 0x9966, 0x0660,
    0x0272, 0x04E4, 0x4E40, 0x2720, 0xC936, 0x936C, 0x39C6, 0x639C, 0x9336, 0x9CC6, 0x817E, 0xE718,
    0xCCF0, 0x0FCC, 0x7744, 0xEE22,
];

const BC7_PARTITIONS_3: [u32; 64] = [
    0xAA685050, 0x6A5A5040, 0x5A5A4200, 0x5450A0A8, 0xA5A50000, 0xA0A05050, 0x5555A0A0, 0x5A5A5050,
    0xAA550000, 0xAA555500, 0xAAAA5500, 0x90909090, 0x94949494, 0xA4A4A4A4, 0xA9A59450, 0x2A0A4250,
    0xA5945040, 0x0A425054, 0xA5A5A500, 0x55A0A0A0, 0xA8A85454, 0x6A6A4040, 0xA4A45000, 0x1A1A0500,
    0x0050A4A4, 0xAAA59090, 0x14696914, 0x69691400, 0xA08585A0, 0xAA821414, 0x50A4A450, 0x6A5A0200,
    0xA9A58000, 0x5090A0A8, 0xA8A09050, 0x24242424, 0x00AA5500, 0x24924924, 0x24499224, 0x50A50A50,
    0x500AA550, 0xAAAA4444, 0x66660000, 0xA5A0A5A0, 0x50A050A0, 0x69286928, 0x44AAAA44, 0x66666600,
    0xAA444444, 0x54A854A8, 0x95809580, 0x96969600, 0xA85454A8, 0x80959580, 0xAA141414, 0x96960000,
    0xAAAA1414, 0xA05050A0, 0xA0A5A5A0, 0x96000000, 0x40804080, 0xA9A8A9A8, 0xAAAAAA44, 0x2A4A5254,
];

const BC7_ANCHOR_2: [u8; 64] = [
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 2, 8, 2, 2, 8, 8, 15, 2, 8,
    2, 2, 8, 8, 2, 2, 15, 15, 6, 8, 2, 8, 15, 15, 2, 8, 2, 2, 2, 15, 15, 6, 6, 2, 6, 8, 15, 15, 2,
    2, 15, 15, 15, 15, 15, 2, 2, 15,
];

const BC7_ANCHOR_3A: [u8; 64] = [
    3, 3, 15, 15, 8, 3, 15, 15, 8, 8, 6, 6, 6, 5, 3, 3, 3, 3, 8, 15, 3, 3, 6, 10, 5, 8, 8, 6, 8, 5,
    15, 15, 8, 15, 3, 5, 6, 10, 8, 15, 15, 3, 15, 5, 15, 15, 15, 15, 3, 15, 5, 5, 5, 8, 5, 10, 5,
    10, 8, 13, 15, 12, 3, 3,
];

const BC7_ANCHOR_3B: [u8; 64] = [
    15, 8, 8, 3, 15, 15, 3, 8, 15, 15, 15, 15, 15, 15, 15, 8, 15, 8, 15, 3, 15, 8, 15, 8, 3, 15, 6,
    10, 15, 15, 10, 8, 15, 3, 15, 10, 10, 8, 9, 10, 6, 15, 8, 15, 3, 6, 6, 8, 15, 3, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 3, 15, 15, 8,
];

#[derive(Clone, Copy)]
struct ModeSpec {
    subsets: usize,
    part_bits: u32,
    rotation_bits: u32,
    idx_sel_bits: u32,
    color_bits: u32,
    alpha_bits: u32,
    ep_pbits: u32, // 0 = none, 1 = per-endpoint, 2 = shared per-subset
    idx_bits: u32,
    idx2_bits: u32,
}

const BC7_MODES: [ModeSpec; 8] = [
    // Mode 0: 3 subsets, 4-bit part, R4G4B4 + per-endpoint P-bit, 3-bit index
    ModeSpec {
        subsets: 3,
        part_bits: 4,
        rotation_bits: 0,
        idx_sel_bits: 0,
        color_bits: 4,
        alpha_bits: 0,
        ep_pbits: 1,
        idx_bits: 3,
        idx2_bits: 0,
    },
    // Mode 1: 2 subsets, 6-bit part, R6G6B6 + shared P-bit per subset, 3-bit index
    ModeSpec {
        subsets: 2,
        part_bits: 6,
        rotation_bits: 0,
        idx_sel_bits: 0,
        color_bits: 6,
        alpha_bits: 0,
        ep_pbits: 2,
        idx_bits: 3,
        idx2_bits: 0,
    },
    // Mode 2: 3 subsets, 6-bit part, R5G5B5, 2-bit index
    ModeSpec {
        subsets: 3,
        part_bits: 6,
        rotation_bits: 0,
        idx_sel_bits: 0,
        color_bits: 5,
        alpha_bits: 0,
        ep_pbits: 0,
        idx_bits: 2,
        idx2_bits: 0,
    },
    // Mode 3: 2 subsets, 6-bit part, R7G7B7 + per-endpoint P-bit, 2-bit index
    ModeSpec {
        subsets: 2,
        part_bits: 6,
        rotation_bits: 0,
        idx_sel_bits: 0,
        color_bits: 7,
        alpha_bits: 0,
        ep_pbits: 1,
        idx_bits: 2,
        idx2_bits: 0,
    },
    // Mode 4: 1 subset, 2-bit rot, 1-bit idx_sel, R5G5B5 A6, 2-bit color idx, 3-bit alpha idx
    ModeSpec {
        subsets: 1,
        part_bits: 0,
        rotation_bits: 2,
        idx_sel_bits: 1,
        color_bits: 5,
        alpha_bits: 6,
        ep_pbits: 0,
        idx_bits: 2,
        idx2_bits: 3,
    },
    // Mode 5: 1 subset, 2-bit rot, R7G7B7 A8, 2-bit color idx, 2-bit alpha idx
    ModeSpec {
        subsets: 1,
        part_bits: 0,
        rotation_bits: 2,
        idx_sel_bits: 0,
        color_bits: 7,
        alpha_bits: 8,
        ep_pbits: 0,
        idx_bits: 2,
        idx2_bits: 2,
    },
    // Mode 6: 1 subset, R7G7B7A7 + per-endpoint P-bit, 4-bit index
    ModeSpec {
        subsets: 1,
        part_bits: 0,
        rotation_bits: 0,
        idx_sel_bits: 0,
        color_bits: 7,
        alpha_bits: 7,
        ep_pbits: 1,
        idx_bits: 4,
        idx2_bits: 0,
    },
    // Mode 7: 2 subsets, 6-bit part, R5G5B5A5 + per-endpoint P-bit, 2-bit index
    ModeSpec {
        subsets: 2,
        part_bits: 6,
        rotation_bits: 0,
        idx_sel_bits: 0,
        color_bits: 5,
        alpha_bits: 5,
        ep_pbits: 1,
        idx_bits: 2,
        idx2_bits: 0,
    },
];

#[inline]
fn read_bits_u128(val: u128, pos: &mut u32, count: u32) -> u8 {
    if count == 0 {
        return 0;
    }
    let mask = (1u128 << count) - 1;
    let res = ((val >> *pos) & mask) as u8;
    *pos += count;
    res
}

#[inline]
fn expand_quantized(val: u8, bits: u32) -> u8 {
    if bits == 0 {
        return 255;
    }
    if bits >= 8 {
        return val;
    }
    let shifted = val << (8 - bits);
    shifted | (shifted >> bits)
}

#[inline]
fn weight_for(bits: u32, idx: u8) -> u16 {
    match bits {
        2 => BC7_WEIGHTS_2[(idx & 3) as usize] as u16,
        3 => BC7_WEIGHTS_3[(idx & 7) as usize] as u16,
        4 => BC7_WEIGHTS_4[(idx & 15) as usize] as u16,
        _ => 0,
    }
}

#[inline]
fn interp_bc7(e0: u8, e1: u8, w: u16) -> u8 {
    (((64 - w) * (e0 as u16) + w * (e1 as u16) + 32) >> 6) as u8
}

fn decode_bc7_block(block: &[u8; 16]) -> [[u8; 4]; 16] {
    let bits = u128::from_le_bytes(*block);
    let mode = bits.trailing_zeros() as usize;
    if mode >= 8 {
        return [[0u8; 4]; 16];
    }
    let spec = BC7_MODES[mode];
    let mut pos = (mode + 1) as u32;

    let partition = read_bits_u128(bits, &mut pos, spec.part_bits) as usize;
    let rotation = read_bits_u128(bits, &mut pos, spec.rotation_bits);
    let idx_sel = read_bits_u128(bits, &mut pos, spec.idx_sel_bits);

    let num_ep = spec.subsets * 2;
    let mut ep = [[255u8; 4]; 6];

    // Color channels R, G, B
    for ch in 0..3 {
        for e in 0..num_ep {
            ep[e][ch] = read_bits_u128(bits, &mut pos, spec.color_bits);
        }
    }
    // Alpha channel if present
    if spec.alpha_bits > 0 {
        for e in 0..num_ep {
            ep[e][3] = read_bits_u128(bits, &mut pos, spec.alpha_bits);
        }
    }

    // P-bits
    let mut c_bits = spec.color_bits;
    let mut a_bits = spec.alpha_bits;
    if spec.ep_pbits == 1 {
        for e in 0..num_ep {
            let p = read_bits_u128(bits, &mut pos, 1);
            for ch in 0..3 {
                ep[e][ch] = (ep[e][ch] << 1) | p;
            }
            if spec.alpha_bits > 0 {
                ep[e][3] = (ep[e][3] << 1) | p;
            }
        }
        c_bits += 1;
        if a_bits > 0 {
            a_bits += 1;
        }
    } else if spec.ep_pbits == 2 {
        for s in 0..spec.subsets {
            let p = read_bits_u128(bits, &mut pos, 1);
            for e in [s * 2, s * 2 + 1] {
                for ch in 0..3 {
                    ep[e][ch] = (ep[e][ch] << 1) | p;
                }
            }
        }
        c_bits += 1;
    }

    // Expand endpoints to 8-bit
    for e in 0..num_ep {
        for ch in 0..3 {
            ep[e][ch] = expand_quantized(ep[e][ch], c_bits);
        }
        ep[e][3] = if a_bits > 0 {
            expand_quantized(ep[e][3], a_bits)
        } else {
            255
        };
    }

    // Read primary indices
    let mut idx1 = [0u8; 16];
    for i in 0..16 {
        let is_anchor = match spec.subsets {
            1 => i == 0,
            2 => i == 0 || i == BC7_ANCHOR_2[partition] as usize,
            3 => {
                i == 0
                    || i == BC7_ANCHOR_3A[partition] as usize
                    || i == BC7_ANCHOR_3B[partition] as usize
            }
            _ => false,
        };
        let nb = if is_anchor {
            spec.idx_bits - 1
        } else {
            spec.idx_bits
        };
        idx1[i] = read_bits_u128(bits, &mut pos, nb);
    }

    // Read secondary indices (Modes 4 & 5)
    let mut idx2 = [0u8; 16];
    if spec.idx2_bits > 0 {
        for i in 0..16 {
            let nb = if i == 0 {
                spec.idx2_bits - 1
            } else {
                spec.idx2_bits
            };
            idx2[i] = read_bits_u128(bits, &mut pos, nb);
        }
    }

    let mut out = [[0u8; 4]; 16];
    for i in 0..16 {
        let subset = match spec.subsets {
            1 => 0,
            2 => ((BC7_PARTITIONS_2[partition] >> i) & 1) as usize,
            3 => ((BC7_PARTITIONS_3[partition] >> (i * 2)) & 3) as usize,
            _ => 0,
        };
        let e0 = ep[subset * 2];
        let e1 = ep[subset * 2 + 1];

        let (wc, wa) = if spec.idx2_bits > 0 {
            if idx_sel != 0 {
                (
                    weight_for(spec.idx2_bits, idx2[i]),
                    weight_for(spec.idx_bits, idx1[i]),
                )
            } else {
                (
                    weight_for(spec.idx_bits, idx1[i]),
                    weight_for(spec.idx2_bits, idx2[i]),
                )
            }
        } else {
            let w = weight_for(spec.idx_bits, idx1[i]);
            (w, w)
        };

        let mut rgba = [
            interp_bc7(e0[0], e1[0], wc),
            interp_bc7(e0[1], e1[1], wc),
            interp_bc7(e0[2], e1[2], wc),
            if spec.alpha_bits > 0 {
                interp_bc7(e0[3], e1[3], wa)
            } else {
                255
            },
        ];

        match rotation {
            1 => rgba.swap(0, 3),
            2 => rgba.swap(1, 3),
            3 => rgba.swap(2, 3),
            _ => {}
        }
        out[i] = rgba;
    }

    out
}

fn decode_bc7(width: usize, height: usize, src: &[u8], out: &mut [u8]) {
    let blocks_x = (width + 3) / 4;
    let blocks_y = (height + 3) / 4;
    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            let off = (by * blocks_x + bx) * 16;
            let block: &[u8; 16] = src[off..off + 16].try_into().unwrap();
            let pixels = decode_bc7_block(block);
            write_4x4_block(width, height, bx, by, &pixels, out);
        }
    }
}

/// Crops a sub-rectangle `(x, y, w, h)` from an RGBA8 buffer.
pub fn crop_rgba8(
    src: &[u8],
    src_w: usize,
    src_h: usize,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
) -> Vec<u8> {
    let cw = w.min(src_w.saturating_sub(x));
    let ch = h.min(src_h.saturating_sub(y));
    let mut out = vec![0u8; cw * ch * 4];
    for row in 0..ch {
        let s_off = ((y + row) * src_w + x) * 4;
        let d_off = row * cw * 4;
        out[d_off..d_off + cw * 4].copy_from_slice(&src[s_off..s_off + cw * 4]);
    }
    out
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in bytes {
        crc ^= b as u32;
        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

fn write_png_chunk(out: &mut Vec<u8>, tag: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let crc_start = out.len();
    out.extend_from_slice(tag);
    out.extend_from_slice(data);
    let crc = crc32(&out[crc_start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

/// Encodes an RGBA8 buffer (`width * height * 4` bytes) into a standard lossless `.png` file.
pub fn encode_png_rgba8(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>, String> {
    let row_bytes = (width as usize) * 4;
    if rgba.len() < row_bytes * (height as usize) {
        return Err("RGBA buffer shorter than width * height * 4".to_string());
    }

    let mut raw_scanlines = Vec::with_capacity((row_bytes + 1) * (height as usize));
    for y in 0..(height as usize) {
        raw_scanlines.push(0u8); // Filter type 0 (None)
        let start = y * row_bytes;
        raw_scanlines.extend_from_slice(&rgba[start..start + row_bytes]);
    }

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(&raw_scanlines)
        .map_err(|e| e.to_string())?;
    let idat = encoder.finish().map_err(|e| e.to_string())?;

    let mut png = Vec::with_capacity(idat.len() + 64);
    png.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);

    let mut ihdr = [0u8; 13];
    ihdr[0..4].copy_from_slice(&width.to_be_bytes());
    ihdr[4..8].copy_from_slice(&height.to_be_bytes());
    ihdr[8] = 8; // 8 bits per channel
    ihdr[9] = 6; // Color type 6 = RGBA
    ihdr[10] = 0;
    ihdr[11] = 0;
    ihdr[12] = 0;

    write_png_chunk(&mut png, b"IHDR", &ihdr);
    write_png_chunk(&mut png, b"IDAT", &idat);
    write_png_chunk(&mut png, b"IEND", &[]);

    Ok(png)
}

#[inline]
fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let p = (a as i32) + (b as i32) - (c as i32);
    let pa = (p - (a as i32)).abs();
    let pb = (p - (b as i32)).abs();
    let pc = (p - (c as i32)).abs();
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

/// Decodes a standard 8-bit RGB or RGBA `.png` file into `(width, height, rgba8)`.
pub fn decode_png_rgba8(png: &[u8]) -> Result<(usize, usize, Vec<u8>), String> {
    use std::io::Read;
    if png.len() < 8 || &png[0..8] != &[137, 80, 78, 71, 13, 10, 26, 10] {
        return Err("Invalid PNG signature".to_string());
    }
    let mut pos = 8usize;
    let mut width = 0usize;
    let mut height = 0usize;
    let mut color_type = 0u8;
    let mut idat = Vec::new();

    while pos + 12 <= png.len() {
        let len = u32::from_be_bytes(png[pos..pos + 4].try_into().unwrap()) as usize;
        let tag = &png[pos + 4..pos + 8];
        if pos + 12 + len > png.len() {
            break;
        }
        let data = &png[pos + 8..pos + 8 + len];
        if tag == b"IHDR" && len >= 13 {
            width = u32::from_be_bytes(data[0..4].try_into().unwrap()) as usize;
            height = u32::from_be_bytes(data[4..8].try_into().unwrap()) as usize;
            let bit_depth = data[8];
            color_type = data[9];
            if bit_depth != 8 || (color_type != 2 && color_type != 6) {
                return Err(format!(
                    "Unsupported PNG bit_depth={bit_depth} color_type={color_type}"
                ));
            }
        } else if tag == b"IDAT" {
            idat.extend_from_slice(data);
        } else if tag == b"IEND" {
            break;
        }
        pos += 12 + len;
    }

    if width == 0 || height == 0 {
        return Err("Missing IHDR in PNG".to_string());
    }
    let bpp = if color_type == 6 { 4usize } else { 3usize };
    let stride = width * bpp;
    let mut raw = Vec::with_capacity((stride + 1) * height);
    flate2::read::ZlibDecoder::new(&idat[..])
        .read_to_end(&mut raw)
        .map_err(|e| format!("PNG zlib decode error: {e}"))?;
    if raw.len() < (stride + 1) * height {
        return Err("Decoded PNG scanlines shorter than expected".to_string());
    }

    let mut out = vec![0u8; width * height * 4];
    let mut prev_row = vec![0u8; stride];
    let mut cur_row = vec![0u8; stride];

    for y in 0..height {
        let row_start = y * (stride + 1);
        let filter = raw[row_start];
        let src = &raw[row_start + 1..row_start + 1 + stride];
        for i in 0..stride {
            let a = if i >= bpp { cur_row[i - bpp] } else { 0 };
            let b = prev_row[i];
            let c = if i >= bpp { prev_row[i - bpp] } else { 0 };
            cur_row[i] = match filter {
                0 => src[i],
                1 => src[i].wrapping_add(a),
                2 => src[i].wrapping_add(b),
                3 => src[i].wrapping_add(((a as u16 + b as u16) >> 1) as u8),
                4 => src[i].wrapping_add(paeth_predictor(a, b, c)),
                _ => src[i],
            };
        }
        let dst_row = &mut out[y * width * 4..(y + 1) * width * 4];
        if bpp == 4 {
            dst_row.copy_from_slice(&cur_row);
        } else {
            for x in 0..width {
                let d = x * 4;
                let s = x * 3;
                dst_row[d] = cur_row[s];
                dst_row[d + 1] = cur_row[s + 1];
                dst_row[d + 2] = cur_row[s + 2];
                dst_row[d + 3] = 255;
            }
        }
        prev_row.copy_from_slice(&cur_row);
    }

    Ok((width, height, out))
}
