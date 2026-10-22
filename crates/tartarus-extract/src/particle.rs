//! Cooked Unreal Engine 4.27 `UIParticleEmitterAsset` (`.uasset` + `.uexp`) parser and
//! deterministic `StartTimeOffset` (`T = 0` pre-warm) particle simulator.

use byteorder::{ByteOrder, LittleEndian};
use serde::{Deserialize, Serialize};

use crate::error::ExtractError;
use crate::material::game_path_to_virtual_path;
use crate::uasset::{
    PACKAGE_FILE_TAG, UassetImportEntry, format_fname_idx, parse_uasset_imports,
    parse_uasset_names, resolve_package_index,
};

const UASSET_BYTES_MAX: usize = 64 * 1024 * 1024;
const UEXP_BYTES_MAX: usize = 64 * 1024 * 1024;
const PROPS_ITER_MAX: usize = 256;
const CURVE_KEYS_MAX: usize = 64;
const RICH_CURVE_KEY_SIZE: usize = 27;

const _: () = {
    assert!(UASSET_BYTES_MAX == 67_108_864);
    assert!(RICH_CURVE_KEY_SIZE == 27);
};

/// Single keyframe `(time, value)` from a cooked `FRichCurveKey` (`27` bytes).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RichCurveKey {
    pub time: f32,
    pub value: f32,
}

/// 4-channel RGBA curve (`ColorCurves[0..4]`) from `FUIParticleLinearColorCurve`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LinearColorCurve {
    pub channels: [Vec<RichCurveKey>; 4],
}

impl LinearColorCurve {
    /// Evaluates the RGBA curve at normalized particle lifetime `t` (`0.0..=1.0`).
    pub fn evaluate(&self, t: f32) -> [f32; 4] {
        assert!(t.is_finite());
        let clamped_t = t.clamp(0.0, 1.0);
        [
            eval_curve_keys(&self.channels[0], clamped_t, 1.0),
            eval_curve_keys(&self.channels[1], clamped_t, 1.0),
            eval_curve_keys(&self.channels[2], clamped_t, 1.0),
            eval_curve_keys(&self.channels[3], clamped_t, 1.0),
        ]
    }
}

/// Parsed `UIParticleEmitterAsset` properties from `UI_P_V2_S16BoardCell_*.uasset`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiParticleEmitterAsset {
    pub virtual_path: String,
    pub auto_play: bool,
    pub start_time_offset: f32,
    pub emit_seconds: f32,
    pub max_particle_count: [f32; 2],
    pub spawn_per_second: [f32; 2],
    pub emit_angle_deg: f32,
    pub emit_pos_min: [f32; 2],
    pub emit_pos_max: [f32; 2],
    pub life_span: [f32; 2],
    pub size_range: [f32; 2],
    pub start_speed: [f32; 2],
    pub air_resistance: [f32; 2],
    pub direction_scale: [f32; 2],
    pub color_curve_min: LinearColorCurve,
    pub color_curve_max: LinearColorCurve,
    pub resource_material_path: Option<String>,
}

impl Default for UiParticleEmitterAsset {
    fn default() -> Self {
        Self {
            virtual_path: String::new(),
            auto_play: false,
            start_time_offset: 0.0,
            emit_seconds: -1.0,
            max_particle_count: [3.0, 5.0],
            spawn_per_second: [10.0, 10.0],
            emit_angle_deg: -90.0,
            emit_pos_min: [45.0, 30.0],
            emit_pos_max: [-45.0, -15.0],
            life_span: [2.5, 1.0],
            size_range: [25.0, 15.0],
            start_speed: [15.0, 40.0],
            air_resistance: [0.15, 0.05],
            direction_scale: [0.2, 2.0],
            color_curve_min: LinearColorCurve::default(),
            color_curve_max: LinearColorCurve::default(),
            resource_material_path: None,
        }
    }
}

/// A single active 2D UI particle at `T = 0` after simulating `StartTimeOffset`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SimulatedUiParticle {
    pub local_x: f32,
    pub local_y: f32,
    pub size: f32,
    pub linear_rgba: [f32; 4],
}

#[derive(Clone, Debug)]
struct PropertyTag {
    name: String,
    prop_type: String,
    size: usize,
    array_index: usize,
    bool_val: bool,
    value_offset: usize,
    next_offset: usize,
}

fn eval_curve_keys(keys: &[RichCurveKey], t: f32, default_val: f32) -> f32 {
    assert!(t.is_finite());
    if keys.is_empty() {
        return default_val;
    }
    if t <= keys[0].time || keys.len() == 1 {
        return keys[0].value;
    }
    let last = keys[keys.len() - 1];
    if t >= last.time {
        return last.value;
    }
    for pair in keys.windows(2) {
        let k0 = pair[0];
        let k1 = pair[1];
        if t >= k0.time && t <= k1.time {
            let span = (k1.time - k0.time).max(1e-6);
            let frac = ((t - k0.time) / span).clamp(0.0, 1.0);
            return k0.value + (k1.value - k0.value) * frac;
        }
    }
    last.value
}

fn read_fname_at(buf: &[u8], pos: usize, names: &[String]) -> Option<(String, usize)> {
    assert!(names.len() <= 65_536);
    let slice = buf.get(pos..pos + 8)?;
    let idx = LittleEndian::read_u32(&slice[0..4]);
    let num = LittleEndian::read_u32(&slice[4..8]);
    if idx as usize >= names.len() || num > 65_536 {
        return None;
    }
    Some((format_fname_idx(names, idx, num), pos + 8))
}

fn read_property_tag(buf: &[u8], start: usize, names: &[String]) -> Option<Option<PropertyTag>> {
    assert!(start <= buf.len());
    let (name, mut pos) = read_fname_at(buf, start, names)?;
    if name == "None" {
        return Some(None);
    }
    let (prop_type, next_pos) = read_fname_at(buf, pos, names)?;
    pos = next_pos;
    let hdr = buf.get(pos..pos + 8)?;
    let size = LittleEndian::read_i32(&hdr[0..4]);
    let array_index = LittleEndian::read_i32(&hdr[4..8]).max(0) as usize;
    if !(0..=16 * 1024 * 1024).contains(&size) {
        return None;
    }
    let size = size as usize;
    pos += 8;

    let mut bool_val = false;
    match prop_type.as_str() {
        "StructProperty" => {
            let (_, p2) = read_fname_at(buf, pos, names)?;
            let has_guid = *buf.get(p2 + 16)?;
            pos = p2 + 17 + if has_guid != 0 { 16 } else { 0 };
        }
        "BoolProperty" => {
            bool_val = *buf.get(pos)? != 0;
            let has_guid = *buf.get(pos + 1)?;
            pos += 2 + if has_guid != 0 { 16 } else { 0 };
        }
        "ByteProperty" | "EnumProperty" | "ArrayProperty" | "SetProperty" => {
            let (_, p2) = read_fname_at(buf, pos, names)?;
            let has_guid = *buf.get(p2)?;
            pos = p2 + 1 + if has_guid != 0 { 16 } else { 0 };
        }
        "MapProperty" => {
            let (_, p2) = read_fname_at(buf, pos, names)?;
            let (_, p3) = read_fname_at(buf, p2, names)?;
            let has_guid = *buf.get(p3)?;
            pos = p3 + 1 + if has_guid != 0 { 16 } else { 0 };
        }
        _ => {
            let has_guid = *buf.get(pos)?;
            pos += 1 + if has_guid != 0 { 16 } else { 0 };
        }
    }
    let next_offset = pos.checked_add(size)?;
    if next_offset > buf.len() {
        return None;
    }
    Some(Some(PropertyTag {
        name,
        prop_type,
        size,
        array_index,
        bool_val,
        value_offset: pos,
        next_offset,
    }))
}

fn parse_float_range_struct(
    buf: &[u8],
    start: usize,
    end: usize,
    names: &[String],
    default_range: [f32; 2],
) -> [f32; 2] {
    assert!(start <= end && end <= buf.len());
    let mut out = default_range;
    let mut pos = start;
    for _ in 0..8 {
        if pos >= end {
            break;
        }
        let Some(Some(tag)) = read_property_tag(buf, pos, names) else {
            break;
        };
        if tag.prop_type == "FloatProperty" && tag.size == 4 {
            let v = LittleEndian::read_f32(&buf[tag.value_offset..tag.value_offset + 4]);
            match tag.name.as_str() {
                "Min" => out[0] = v,
                "Max" => out[1] = v,
                _ => {}
            }
        }
        pos = tag.next_offset;
    }
    out
}

fn parse_ui_particle_float_prop(
    buf: &[u8],
    tag: &PropertyTag,
    names: &[String],
    default_range: [f32; 2],
) -> [f32; 2] {
    assert!(tag.value_offset <= tag.next_offset && tag.next_offset <= buf.len());
    let mut out = default_range;
    let mut float_val = None;
    let mut has_range = false;
    let mut pos = tag.value_offset;
    for _ in 0..16 {
        if pos >= tag.next_offset {
            break;
        }
        let Some(Some(sub)) = read_property_tag(buf, pos, names) else {
            break;
        };
        match (sub.name.as_str(), sub.prop_type.as_str()) {
            ("FloatValue", "FloatProperty") if sub.size == 4 => {
                float_val = Some(LittleEndian::read_f32(
                    &buf[sub.value_offset..sub.value_offset + 4],
                ));
            }
            ("FloatRangeValue", "StructProperty") => {
                out = parse_float_range_struct(
                    buf,
                    sub.value_offset,
                    sub.next_offset,
                    names,
                    default_range,
                );
                has_range = true;
            }
            _ => {}
        }
        pos = sub.next_offset;
    }
    if !has_range && let Some(v) = float_val {
        return [v, v];
    }
    out
}

fn parse_vec2_range_struct(
    buf: &[u8],
    tag: &PropertyTag,
    names: &[String],
    default_min: [f32; 2],
    default_max: [f32; 2],
) -> ([f32; 2], [f32; 2]) {
    assert!(tag.value_offset <= tag.next_offset && tag.next_offset <= buf.len());
    let mut min_v = default_min;
    let mut max_v = default_max;
    let mut pos = tag.value_offset;
    for _ in 0..8 {
        if pos >= tag.next_offset {
            break;
        }
        let Some(Some(sub)) = read_property_tag(buf, pos, names) else {
            break;
        };
        if sub.prop_type == "StructProperty" && sub.size == 8 {
            let x = LittleEndian::read_f32(&buf[sub.value_offset..sub.value_offset + 4]);
            let y = LittleEndian::read_f32(&buf[sub.value_offset + 4..sub.value_offset + 8]);
            match sub.name.as_str() {
                "Min" => min_v = [x, y],
                "Max" => max_v = [x, y],
                _ => {}
            }
        }
        pos = sub.next_offset;
    }
    (min_v, max_v)
}

fn parse_rich_curve_keys(
    buf: &[u8],
    start: usize,
    end: usize,
    names: &[String],
) -> Vec<RichCurveKey> {
    assert!(start <= end && end <= buf.len());
    let mut keys = Vec::new();
    let mut pos = start;
    for _ in 0..8 {
        if pos >= end {
            break;
        }
        let Some(Some(sub)) = read_property_tag(buf, pos, names) else {
            break;
        };
        if sub.name == "Keys" && sub.prop_type == "ArrayProperty" && sub.size >= 4 {
            let count = LittleEndian::read_i32(&buf[sub.value_offset..sub.value_offset + 4])
                .clamp(0, CURVE_KEYS_MAX as i32) as usize;
            if let Some(Some(proto)) = read_property_tag(buf, sub.value_offset + 4, names) {
                let mut kpos = proto.value_offset;
                for _ in 0..count {
                    let Some(s) = buf.get(kpos..kpos + RICH_CURVE_KEY_SIZE) else {
                        break;
                    };
                    let time = LittleEndian::read_f32(&s[3..7]);
                    let value = LittleEndian::read_f32(&s[7..11]);
                    if time.is_finite() && value.is_finite() {
                        keys.push(RichCurveKey { time, value });
                    }
                    kpos += RICH_CURVE_KEY_SIZE;
                }
            }
        }
        pos = sub.next_offset;
    }
    assert!(keys.len() <= CURVE_KEYS_MAX);
    keys
}

fn parse_linear_color_curve(
    buf: &[u8],
    start: usize,
    end: usize,
    names: &[String],
) -> LinearColorCurve {
    assert!(start <= end && end <= buf.len());
    let mut curve = LinearColorCurve::default();
    let mut pos = start;
    for _ in 0..16 {
        if pos >= end {
            break;
        }
        let Some(Some(sub)) = read_property_tag(buf, pos, names) else {
            break;
        };
        if sub.name == "ColorCurves" && sub.prop_type == "StructProperty" && sub.array_index < 4 {
            curve.channels[sub.array_index] =
                parse_rich_curve_keys(buf, sub.value_offset, sub.next_offset, names);
        }
        pos = sub.next_offset;
    }
    curve
}

fn parse_color_property(
    buf: &[u8],
    tag: &PropertyTag,
    names: &[String],
) -> (LinearColorCurve, LinearColorCurve) {
    assert!(tag.value_offset <= tag.next_offset && tag.next_offset <= buf.len());
    let mut min_c = LinearColorCurve::default();
    let mut max_c = LinearColorCurve::default();
    let mut pos = tag.value_offset;
    for _ in 0..16 {
        if pos >= tag.next_offset {
            break;
        }
        let Some(Some(sub)) = read_property_tag(buf, pos, names) else {
            break;
        };
        if sub.name == "LinearColorCurveRangeValue" && sub.prop_type == "StructProperty" {
            let mut rpos = sub.value_offset;
            for _ in 0..8 {
                if rpos >= sub.next_offset {
                    break;
                }
                let Some(Some(rsub)) = read_property_tag(buf, rpos, names) else {
                    break;
                };
                match rsub.name.as_str() {
                    "Min" => {
                        min_c = parse_linear_color_curve(
                            buf,
                            rsub.value_offset,
                            rsub.next_offset,
                            names,
                        );
                    }
                    "Max" => {
                        max_c = parse_linear_color_curve(
                            buf,
                            rsub.value_offset,
                            rsub.next_offset,
                            names,
                        );
                    }
                    _ => {}
                }
                rpos = rsub.next_offset;
            }
        }
        pos = sub.next_offset;
    }
    (min_c, max_c)
}

/// Parses a cooked `UIParticleEmitterAsset` (`.uasset` + `.uexp`).
pub fn parse_ui_particle_asset(
    virtual_path: &str,
    uasset: &[u8],
    uexp: &[u8],
) -> Result<UiParticleEmitterAsset, ExtractError> {
    assert!(uasset.len() <= UASSET_BYTES_MAX);
    assert!(uexp.len() <= UEXP_BYTES_MAX);
    if uasset.len() < 8 || LittleEndian::read_u32(uasset) != PACKAGE_FILE_TAG {
        return Err(ExtractError::UassetFormat(format!(
            "Invalid UIParticleEmitterAsset .uasset tag in {virtual_path}"
        )));
    }
    let names = parse_uasset_names(uasset);
    let imports: Vec<UassetImportEntry> = parse_uasset_imports(uasset, &names);
    let mut asset = UiParticleEmitterAsset {
        virtual_path: virtual_path.to_string(),
        ..Default::default()
    };

    let limit = uexp.len().saturating_sub(4);
    let mut pos = 0usize;
    for _ in 0..PROPS_ITER_MAX {
        if pos >= limit {
            break;
        }
        let Some(tag_opt) = read_property_tag(uexp, pos, &names) else {
            break;
        };
        let Some(tag) = tag_opt else {
            break;
        };
        match (tag.name.as_str(), tag.prop_type.as_str()) {
            ("AutoPlay", "BoolProperty") => asset.auto_play = tag.bool_val,
            ("StartTimeOffset", "FloatProperty") if tag.size == 4 => {
                asset.start_time_offset =
                    LittleEndian::read_f32(&uexp[tag.value_offset..tag.value_offset + 4]);
            }
            ("EmitSeconds", "FloatProperty") if tag.size == 4 => {
                asset.emit_seconds =
                    LittleEndian::read_f32(&uexp[tag.value_offset..tag.value_offset + 4]);
            }
            ("MaxParticleCount", "StructProperty") => {
                asset.max_particle_count =
                    parse_ui_particle_float_prop(uexp, &tag, &names, asset.max_particle_count);
            }
            ("SpawnParticlePerSecond", "StructProperty") => {
                asset.spawn_per_second =
                    parse_ui_particle_float_prop(uexp, &tag, &names, asset.spawn_per_second);
            }
            ("ParticleEmitAngle", "StructProperty") => {
                let r = parse_ui_particle_float_prop(
                    uexp,
                    &tag,
                    &names,
                    [asset.emit_angle_deg, asset.emit_angle_deg],
                );
                asset.emit_angle_deg = r[0];
            }
            ("EmitPosRange", "StructProperty") => {
                let (mn, mx) = parse_vec2_range_struct(
                    uexp,
                    &tag,
                    &names,
                    asset.emit_pos_min,
                    asset.emit_pos_max,
                );
                asset.emit_pos_min = mn;
                asset.emit_pos_max = mx;
            }
            ("LifeSpan", "StructProperty") => {
                asset.life_span = parse_ui_particle_float_prop(uexp, &tag, &names, asset.life_span);
            }
            ("Size", "StructProperty") => {
                asset.size_range =
                    parse_ui_particle_float_prop(uexp, &tag, &names, asset.size_range);
            }
            ("StartSpeed", "StructProperty") => {
                asset.start_speed =
                    parse_ui_particle_float_prop(uexp, &tag, &names, asset.start_speed);
            }
            ("AirResistance", "StructProperty") => {
                asset.air_resistance =
                    parse_ui_particle_float_prop(uexp, &tag, &names, asset.air_resistance);
            }
            ("DirectionScale", "StructProperty") => {
                asset.direction_scale =
                    parse_ui_particle_float_prop(uexp, &tag, &names, asset.direction_scale);
            }
            ("Color", "StructProperty") => {
                let (cmin, cmax) = parse_color_property(uexp, &tag, &names);
                asset.color_curve_min = cmin;
                asset.color_curve_max = cmax;
            }
            ("ResourceObject", "ObjectProperty") if tag.size == 4 => {
                let pidx = LittleEndian::read_i32(&uexp[tag.value_offset..tag.value_offset + 4]);
                asset.resource_material_path =
                    resolve_package_index(&imports, pidx).map(|s| game_path_to_virtual_path(&s));
            }
            _ => {}
        }
        pos = tag.next_offset;
    }
    assert!(pos <= uexp.len());
    Ok(asset)
}

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(0x9E37_79B9_7F4A_7C15),
        }
    }

    fn next_f32(&mut self) -> f32 {
        let mut z = self.state;
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        ((z >> 40) as u32 as f32) * (1.0 / ((1u32 << 24) as f32))
    }

    fn range(&mut self, a: f32, b: f32) -> f32 {
        let lo = a.min(b);
        let hi = a.max(b);
        lo + (hi - lo) * self.next_f32()
    }
}

#[derive(Clone, Copy)]
struct LiveParticle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    age: f32,
    life: f32,
    size: f32,
    air_drag: f32,
    color_lerp: f32,
}

impl UiParticleEmitterAsset {
    /// Simulates the emitter over `[0, StartTimeOffset]` at `60 Hz` using a deterministic seed
    /// and returns the live particles at `T = 0`.
    pub fn simulate_prewarm_t0(&self, seed: u64) -> Vec<SimulatedUiParticle> {
        assert!(self.start_time_offset.is_finite());
        if !self.auto_play || self.start_time_offset <= 0.0 {
            return Vec::new();
        }
        let mut rng = SplitMix64::new(seed);
        let max_count = rng
            .range(self.max_particle_count[0], self.max_particle_count[1])
            .round()
            .clamp(1.0, 64.0) as usize;
        let dt = 1.0f32 / 60.0;
        let steps = ((self.start_time_offset / dt).round() as usize).clamp(1, 600);
        let mut live: Vec<LiveParticle> = Vec::with_capacity(max_count);
        let mut spawn_accum = 0.85f32;

        for _ in 0..steps {
            live.retain_mut(|p| {
                p.age += dt;
                if p.age >= p.life {
                    return false;
                }
                let damp = (1.0 - p.air_drag * dt).clamp(0.0, 1.0);
                p.vx *= damp;
                p.vy *= damp;
                p.x += p.vx * dt;
                p.y += p.vy * dt;
                true
            });

            let rate = rng
                .range(self.spawn_per_second[0], self.spawn_per_second[1])
                .max(0.5);
            spawn_accum += rate * dt;
            while spawn_accum >= 1.0 {
                spawn_accum -= 1.0;
                if live.len() < max_count {
                    let x = rng.range(self.emit_pos_min[0], self.emit_pos_max[0]);
                    let y = rng.range(self.emit_pos_min[1], self.emit_pos_max[1]);
                    let speed = rng.range(self.start_speed[0], self.start_speed[1]);
                    let angle_rad = self.emit_angle_deg.to_radians();
                    let vx = angle_rad.cos() * speed;
                    let vy = angle_rad.sin() * speed;
                    let life = rng.range(self.life_span[0], self.life_span[1]).max(0.2);
                    let size = rng.range(self.size_range[0], self.size_range[1]).max(2.0);
                    let air_drag = rng.range(self.air_resistance[0], self.air_resistance[1]);
                    let color_lerp = rng.next_f32();
                    live.push(LiveParticle {
                        x,
                        y,
                        vx,
                        vy,
                        age: 0.0,
                        life,
                        size,
                        air_drag,
                        color_lerp,
                    });
                }
            }
        }

        let mut out = Vec::with_capacity(live.len());
        for p in live {
            let norm_age = (p.age / p.life).clamp(0.0, 1.0);
            let c0 = self.color_curve_min.evaluate(norm_age);
            let c1 = self.color_curve_max.evaluate(norm_age);
            let mut rgba = [0.0f32; 4];
            for ch in 0..4 {
                rgba[ch] = (c0[ch] + (c1[ch] - c0[ch]) * p.color_lerp).clamp(0.0, 4.0);
            }
            if rgba[3] > 1e-3 {
                out.push(SimulatedUiParticle {
                    local_x: p.x,
                    local_y: p.y,
                    size: p.size,
                    linear_rgba: rgba,
                });
            }
        }
        assert!(out.len() <= 64);
        out
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_and_simulate_board_cell_particles_if_game_installed() {
        let Some(game_dir) = crate::discover_game_dir() else {
            return;
        };
        let Ok(vfs) = crate::PakVfs::open_game_dir(&game_dir, crate::DEFAULT_AES_KEY) else {
            return;
        };
        let p016 = vfs
            .read_ui_particle_asset(
                "UE_game/Content/Art/UIV2/S16Gameplay/UIEffect/S16BoardCell/Particle/UI_P_V2_S16BoardCell_016",
            )
            .expect("UI_P_V2_S16BoardCell_016");
        assert!(p016.auto_play);
        assert_eq!(p016.start_time_offset, 2.0);
        let sim = p016.simulate_prewarm_t0(0x5160_0016);
        assert!(
            !sim.is_empty(),
            "Expected live pre-warmed particles at T=0 for UI_P_V2_S16BoardCell_016"
        );
    }
}
