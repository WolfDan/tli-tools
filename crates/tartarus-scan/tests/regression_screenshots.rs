use std::fs;
use std::path::{Path, PathBuf};

use tartarus_core::{BoardStep, CellKind, ObjectiveId, StartEventId, ToolId};
use tartarus_scan::{
    CaptureTrigger, ImageView, RunTracker, ScanArena, ScanOptions, ScannedCellKind, TitleRead,
    read_objective_title_with_slot, scan_image_with_arena, scan_image_with_options_and_arena,
    scan_pick_with_slot,
};

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn fixture_path(name: &str) -> PathBuf {
    workspace_root().join("tests").join("fixtures").join(name)
}

struct BitReader<'a> {
    data: &'a [u8],
    byte_pos: usize,
    bit_buf: u64,
    bit_cnt: u32,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            byte_pos: 0,
            bit_buf: 0,
            bit_cnt: 0,
        }
    }

    #[inline]
    fn ensure(&mut self, n: u32) {
        while self.bit_cnt < n && self.byte_pos < self.data.len() {
            self.bit_buf |= (self.data[self.byte_pos] as u64) << self.bit_cnt;
            self.byte_pos += 1;
            self.bit_cnt += 8;
        }
    }

    #[inline]
    fn read_bits(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        self.ensure(n);
        let val = (self.bit_buf & ((1u64 << n) - 1)) as u32;
        self.bit_buf >>= n;
        self.bit_cnt -= n;
        val
    }

    fn align_byte(&mut self) {
        let rem = self.bit_cnt & 7;
        self.bit_buf >>= rem;
        self.bit_cnt -= rem;
    }
}

struct HuffTable {
    counts: [u16; 16],
    symbols: Vec<u16>,
}

impl HuffTable {
    fn build(lengths: &[u8]) -> Self {
        let mut counts = [0u16; 16];
        for &l in lengths {
            if l > 0 {
                counts[l as usize] += 1;
            }
        }
        let mut offsets = [0u16; 16];
        let mut total = 0u16;
        for bits in 1..16 {
            offsets[bits] = total;
            total += counts[bits];
        }
        let mut symbols = vec![0u16; total as usize];
        for (sym, &l) in lengths.iter().enumerate() {
            if l > 0 {
                let idx = offsets[l as usize] as usize;
                symbols[idx] = sym as u16;
                offsets[l as usize] += 1;
            }
        }
        Self { counts, symbols }
    }

    #[inline]
    fn decode(&self, br: &mut BitReader<'_>) -> u16 {
        br.ensure(15);
        let mut code = 0i32;
        let mut first = 0i32;
        let mut index = 0i32;
        for len in 1..=15 {
            code |= (br.bit_buf & 1) as i32;
            br.bit_buf >>= 1;
            br.bit_cnt -= 1;
            let count = self.counts[len] as i32;
            if code - count < first {
                return self.symbols[(index + (code - first)) as usize];
            }
            index += count;
            first += count;
            first <<= 1;
            code <<= 1;
        }
        panic!("invalid DEFLATE Huffman code");
    }
}

fn inflate_zlib(zlib: &[u8], capacity: usize) -> Vec<u8> {
    assert!(zlib.len() >= 6, "zlib stream too short");
    let cmf = zlib[0];
    let flg = zlib[1];
    assert_eq!(cmf & 0x0F, 8, "unsupported zlib compression method");
    let start = if (flg & 0x20) != 0 { 6 } else { 2 };
    let mut br = BitReader::new(&zlib[start..]);
    let mut out = Vec::with_capacity(capacity);

    const LEN_BASE: [u16; 29] = [
        3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115,
        131, 163, 195, 227, 258,
    ];
    const LEN_EXTRA: [u8; 29] = [
        0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
    ];
    const DIST_BASE: [u16; 30] = [
        1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
        2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
    ];
    const DIST_EXTRA: [u8; 30] = [
        0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12,
        13, 13,
    ];
    const CL_ORDER: [usize; 19] = [
        16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
    ];

    loop {
        let bfinal = br.read_bits(1);
        let btype = br.read_bits(2);
        match btype {
            0 => {
                br.align_byte();
                let len = br.read_bits(16) as usize;
                let _nlen = br.read_bits(16);
                for _ in 0..len {
                    out.push(br.read_bits(8) as u8);
                }
            }
            1 | 2 => {
                let (lit_table, dist_table) = if btype == 1 {
                    let mut ll = vec![0u8; 288];
                    ll[0..=143].fill(8);
                    ll[144..=255].fill(9);
                    ll[256..=279].fill(7);
                    ll[280..=287].fill(8);
                    let dl = vec![5u8; 32];
                    (HuffTable::build(&ll), HuffTable::build(&dl))
                } else {
                    let hlit = br.read_bits(5) as usize + 257;
                    let hdist = br.read_bits(5) as usize + 1;
                    let hclen = br.read_bits(4) as usize + 4;
                    let mut cl_lens = [0u8; 19];
                    for i in 0..hclen {
                        cl_lens[CL_ORDER[i]] = br.read_bits(3) as u8;
                    }
                    let cl_table = HuffTable::build(&cl_lens);
                    let mut lens = Vec::with_capacity(hlit + hdist);
                    while lens.len() < hlit + hdist {
                        let s = cl_table.decode(&mut br);
                        match s {
                            0..=15 => lens.push(s as u8),
                            16 => {
                                let prev = *lens.last().expect("repeat previous length");
                                let rep = br.read_bits(2) as usize + 3;
                                lens.extend(std::iter::repeat_n(prev, rep));
                            }
                            17 => {
                                let rep = br.read_bits(3) as usize + 3;
                                lens.extend(std::iter::repeat_n(0, rep));
                            }
                            18 => {
                                let rep = br.read_bits(7) as usize + 11;
                                lens.extend(std::iter::repeat_n(0, rep));
                            }
                            _ => unreachable!(),
                        }
                    }
                    (
                        HuffTable::build(&lens[..hlit]),
                        HuffTable::build(&lens[hlit..hlit + hdist]),
                    )
                };

                loop {
                    let sym = lit_table.decode(&mut br) as usize;
                    if sym < 256 {
                        out.push(sym as u8);
                    } else if sym == 256 {
                        break;
                    } else {
                        let l_idx = sym - 257;
                        let length = LEN_BASE[l_idx] as usize
                            + br.read_bits(LEN_EXTRA[l_idx] as u32) as usize;
                        let d_sym = dist_table.decode(&mut br) as usize;
                        let dist = DIST_BASE[d_sym] as usize
                            + br.read_bits(DIST_EXTRA[d_sym] as u32) as usize;
                        let start_idx = out.len() - dist;
                        for i in 0..length {
                            let b = out[start_idx + i];
                            out.push(b);
                        }
                    }
                }
            }
            _ => panic!("invalid DEFLATE block type {btype}"),
        }
        if bfinal != 0 {
            break;
        }
    }
    out
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

fn load_png_rgba8(path: &Path) -> (usize, usize, Vec<u8>) {
    let png = fs::read(path).unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    assert!(
        png.len() >= 8 && &png[0..8] == &[137, 80, 78, 71, 13, 10, 26, 10],
        "invalid PNG signature in {}",
        path.display()
    );
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
            assert!(
                bit_depth == 8 && (color_type == 2 || color_type == 6),
                "unsupported PNG format in {}: bit_depth={bit_depth} color_type={color_type}",
                path.display()
            );
        } else if tag == b"IDAT" {
            idat.extend_from_slice(data);
        } else if tag == b"IEND" {
            break;
        }
        pos += 12 + len;
    }

    let bpp = if color_type == 6 { 4usize } else { 3usize };
    let stride = width * bpp;
    let raw = inflate_zlib(&idat, (stride + 1) * height);

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

    (width, height, out)
}

#[test]
fn test_board_cell06_and_cell10_warden_localization() {
    let mut arena = ScanArena::new();

    // Cell 06 (0-indexed 5)
    let (w6, h6, rgba6) = load_png_rgba8(&fixture_path("board_cell06_warden.png"));
    let view6 = ImageView {
        width: w6,
        height: h6,
        data: &rgba6,
    };
    let res6 = scan_image_with_arena(&view6, None, &mut arena);
    assert!(
        res6.ok,
        "board must be localized when warden is at cell 06 (idx 5)"
    );
    assert_eq!(
        res6.warden.map(|w| w.cell),
        Some(5),
        "warden must be detected at cell 06 (0-indexed 5)"
    );

    // Cell 10 (0-indexed 9)
    let (w10, h10, rgba10) = load_png_rgba8(&fixture_path("board_cell10_warden.png"));
    let view10 = ImageView {
        width: w10,
        height: h10,
        data: &rgba10,
    };
    let res10 = scan_image_with_arena(&view10, None, &mut arena);
    assert!(
        res10.ok,
        "board must be localized when warden is at cell 10 (idx 9)"
    );
    assert_eq!(
        res10.warden.map(|w| w.cell),
        Some(9),
        "warden must be detected at cell 10 (0-indexed 9)"
    );
}

#[test]
fn test_chain_consume_screens_never_recommend_end_day() {
    let mut arena = ScanArena::new();
    for filename in [
        "chain_consume_targets.png",
        "chain_consume_end_day_prompt.png",
    ] {
        let (w, h, rgba) = load_png_rgba8(&fixture_path(filename));
        let view = ImageView {
            width: w,
            height: h,
            data: &rgba,
        };
        let mut tracker = RunTracker::new();
        let (ok, _, _) = tracker.process_frame(&view, CaptureTrigger::Manual, &mut arena);
        assert!(ok, "{filename}: should recognize board / chain consume screen");
        assert!(
            tracker.model.step == BoardStep::Chain || tracker.model.step == BoardStep::Manage,
            "{filename}: expected Chain or Manage step, got {:?}",
            tracker.model.step
        );
        let snap = tracker.model.evaluate(4000, 80, 42);
        assert!(
            !snap.move_text.to_lowercase().contains("end the day"),
            "{filename}: chain screen must never recommend 'End the day', got '{}'",
            snap.move_text
        );
    }
}

#[test]
fn test_pick_three_relics_and_start_event_popup() {
    let mut arena = ScanArena::new();

    // 1. Iron Pestle (Pestle), Felon's Dog Tag (Dogtag), Hangman's Noose (Rope)
    let (w1, h1, rgba1) =
        load_png_rgba8(&fixture_path("pick_three_relics_pestle_dogtag_noose.png"));
    let view1 = ImageView {
        width: w1,
        height: h1,
        data: &rgba1,
    };
    let pick1 = scan_pick_with_slot(&view1, &mut arena.slots[0]).expect("pick1 detected");
    let tools1: Vec<ToolId> = pick1.cards.iter().map(|c| c.sim).collect();
    assert_eq!(
        tools1,
        vec![ToolId::Pestle, ToolId::Dogtag, ToolId::Rope],
        "expected [Iron Pestle, Felon's Dog Tag, Hangman's Noose]"
    );

    let mut tracker1 = RunTracker::new();
    let (ok1, _, _) = tracker1.process_frame(
        &view1,
        CaptureTrigger::Instruments { fresh: true },
        &mut arena,
    );
    assert!(ok1, "pick1 should be processed by RunTracker");
    let snap1 = tracker1.model.evaluate(4000, 80, 42);
    assert!(
        !snap1.move_text.contains("Skewered Rat Heads"),
        "must never recommend an unoffered relic, got '{}'",
        snap1.move_text
    );
    assert!(
        snap1.move_text.contains("Iron Pestle")
            || snap1.move_text.contains("Felon's Dog Tag")
            || snap1.move_text.contains("Hangman's Noose")
            || snap1.move_text.to_lowercase().contains("reroll"),
        "expected one of the 3 offered relics, got '{}'",
        snap1.move_text
    );

    // 2. Start Event: Tyranny scroll popup
    let (w2, h2, rgba2) = load_png_rgba8(&fixture_path("start_event_tyranny_popup.png"));
    let view2 = ImageView {
        width: w2,
        height: h2,
        data: &rgba2,
    };
    let title = read_objective_title_with_slot(&view2, &mut arena.slots[0]);
    assert!(
        matches!(title, Some(TitleRead::StartEvent(StartEventId::Tyranny))),
        "expected StartEvent(Tyranny) from start_event_tyranny_popup.png, got {:?}",
        title
    );
    let mut tracker2 = RunTracker::new();
    let (ok2, status2, _) = tracker2.process_frame(&view2, CaptureTrigger::Manual, &mut arena);
    assert!(
        ok2,
        "Manual Picture capture on Tyranny popup must be recognized"
    );
    assert_eq!(tracker2.model.start_event, StartEventId::Tyranny);
    assert!(
        status2.as_deref().unwrap_or("").contains("Tyranny"),
        "expected status to mention Tyranny, got {:?}",
        status2
    );

    // 3. Objective: Behind Cell Doors popup
    let (w3, h3, rgba3) = load_png_rgba8(&fixture_path("objective_behind_cell_doors_popup.png"));
    let view3 = ImageView {
        width: w3,
        height: h3,
        data: &rgba3,
    };
    let title3 = read_objective_title_with_slot(&view3, &mut arena.slots[0]);
    assert!(
        matches!(title3, Some(TitleRead::Objective(ObjectiveId::Doors))),
        "expected Objective(Doors) from objective_behind_cell_doors_popup.png, got {:?}",
        title3
    );
    let mut tracker3 = RunTracker::new();
    let (ok3, status3, _) = tracker3.process_frame(&view3, CaptureTrigger::Manual, &mut arena);
    assert!(
        ok3,
        "Manual Picture capture on Behind Cell Doors popup must be recognized"
    );
    assert_eq!(tracker3.model.objective, ObjectiveId::Doors);
    assert!(
        status3.as_deref().unwrap_or("").contains("Behind Cell Doors"),
        "expected status to mention Behind Cell Doors, got {:?}",
        status3
    );
}

#[test]
fn test_day5_and_day13_supreme_jaws_chromatic_and_sequence() {
    let (w5, h5, rgba5) = load_png_rgba8(&fixture_path("board_day5_supreme_lion_no_jaws.png"));
    let (wp, hp, rgbap) = load_png_rgba8(&fixture_path("start_event_tyranny_popup.png"));
    let (w13, h13, rgba13) =
        load_png_rgba8(&fixture_path("board_day13_supreme_jaws_chromatic7.png"));

    let mut arena = ScanArena::new();

    let view5 = ImageView {
        width: w5,
        height: h5,
        data: &rgba5,
    };
    let viewp = ImageView {
        width: wp,
        height: hp,
        data: &rgbap,
    };
    let view13 = ImageView {
        width: w13,
        height: h13,
        data: &rgba13,
    };

    let mut tracker = RunTracker::new();

    // Step 1: Day 5 board (15 days until Riot, Cell 10 = 0-indexed 9)
    let scan5 = scan_image_with_arena(&view5, None, &mut arena);
    assert!(scan5.ok, "Day 5 board must localize");
    assert_eq!(
        scan5.days.map(|(d, _)| d),
        Some(15),
        "Day 5 screenshot shows '15 day(s) until Riot'"
    );
    assert_eq!(
        scan5.chromatic,
        Some(0),
        "Day 5 screenshot shows 0 Chromatic Prisoners"
    );
    let cells5 = scan5.cells.expect("Day 5 cells");
    assert_eq!(cells5[9].kind, ScannedCellKind::Inmate);
    assert_eq!(cells5[9].tier, 4, "Cell 10 (idx 9) is a Supreme Lion");

    let (ok5, _, _) =
        tracker.process_frame(&view5, CaptureTrigger::Board { treasure_view: false }, &mut arena);
    assert!(ok5);
    assert_eq!(
        tracker.model.checks, 0,
        "Day 5 board should have 0 cells to check (Supreme Lion red aura must not flag cell 10), got mask {:#06x}",
        tracker.model.checks
    );
    let (_, probs5) = tracker.model.build_state(42);
    assert!(
        probs5.is_empty(),
        "Day 5 board should have 0 validation problems, got {:?}",
        probs5
    );

    // Step 2: Start Event popup in the middle of the run must NOT corrupt board cells
    let (okp, _, _) = tracker.process_frame(&viewp, CaptureTrigger::Objective, &mut arena);
    assert!(okp);
    assert_eq!(tracker.model.start_event, StartEventId::Tyranny);
    assert_eq!(
        tracker.model.checks, 0,
        "Start event popup must not corrupt underlying board cells"
    );

    // Step 3: Day 13 board (7 days until Riot, Cell 04 = idx 3, Cell 10 = idx 9, Cell 15 = idx 14)
    // Note: `board_day5` (2531x1353) and `board_day13` (2550x1368) have different manual region-crop offsets.
    tracker.hint = None;
    let scan13 = scan_image_with_arena(&view13, None, &mut arena);
    assert!(scan13.ok, "Day 13 board must localize");
    assert_eq!(
        scan13.days.map(|(d, _)| d),
        Some(7),
        "Day 13 screenshot shows '7 day(s) until Riot'"
    );
    assert_eq!(
        scan13.chromatic,
        Some(7),
        "Day 13 screenshot shows 7 Chromatic Prisoners"
    );
    let cells13 = scan13.cells.expect("Day 13 cells");
    assert!(cells13[3].jaws, "Cell 04 (idx 3) on Day 13 has a jaws badge");
    assert_eq!(cells13[9].tier, 4);
    assert!(
        cells13[9].jaws,
        "Cell 10 (idx 9) Supreme Lion on Day 13 has a jaws badge"
    );
    assert_eq!(cells13[14].tier, 4);
    assert!(
        !cells13[14].jaws,
        "Cell 15 (idx 14) Supreme Wolf on Day 13 does NOT have a jaws badge"
    );

    let (ok13, _, _) = tracker.process_frame(
        &view13,
        CaptureTrigger::Board { treasure_view: false },
        &mut arena,
    );
    assert!(ok13);
    assert_eq!(tracker.model.cells[3].kind, CellKind::Inmate);
    assert_eq!(tracker.model.cells[9].kind, CellKind::Inmate);
    assert_eq!(tracker.model.cells[9].tier, 4);
    assert_eq!(tracker.model.cells[14].kind, CellKind::Inmate);
    assert_eq!(tracker.model.cells[14].tier, 4);
    assert_eq!(
        tracker.model.checks, 0,
        "Day 13 board should have 0 cells to check, got mask {:#06x}",
        tracker.model.checks
    );
    let (_, probs13) = tracker.model.build_state(42);
    assert!(
        probs13.is_empty(),
        "Day 13 board should have 0 validation problems, got {:?}",
        probs13
    );
}

#[test]
fn test_phantom_tool_slot_regression() {
    let (w, h, rgba) = load_png_rgba8(&fixture_path("phantom_slot.png"));
    let view = ImageView {
        width: w,
        height: h,
        data: &rgba,
    };
    let mut arena = ScanArena::new();
    let res = scan_image_with_arena(&view, None, &mut arena);
    assert!(
        res.tools.len() <= 2,
        "phantom_slot.png must not detect phantom tool slots, got {:?}",
        res.tools
    );
}

#[test]
fn test_cell06_above_supreme_jaws_cell07() {
    let (w, h, rgba) = load_png_rgba8(&fixture_path("board_day2_cell06_check.png"));
    let view = ImageView {
        width: w,
        height: h,
        data: &rgba,
    };
    let mut arena = ScanArena::new();
    let scan = scan_image_with_arena(&view, None, &mut arena);
    assert!(scan.ok, "board_day2_cell06_check.png must localize");
    let cells = scan.cells.expect("cells");
    assert_eq!(
        cells[5].kind,
        ScannedCellKind::Locked,
        "Cell 06 (idx 5) must be Locked, got {:?}",
        cells[5]
    );
    assert!(
        !cells[5].nest,
        "Cell 06 (idx 5) must not have a false nest mark, got {:?}",
        cells[5]
    );
    assert!(
        cells[5].conf >= 0.85,
        "Cell 06 (idx 5) lock confidence must be >= 0.85 despite Supreme + jaws red glow from Cell 07 below, got {:?}",
        cells[5]
    );
    assert_eq!(cells[6].kind, ScannedCellKind::Inmate);
    assert_eq!(cells[6].tier, 4, "Cell 07 (idx 6) is a Supreme Mammoth");
    assert!(cells[6].jaws, "Cell 07 (idx 6) has a red jaws badge");

    let mut tracker = RunTracker::new();
    let (ok, _, _) =
        tracker.process_frame(&view, CaptureTrigger::Board { treasure_view: false }, &mut arena);
    assert!(ok);
    assert_eq!(
        tracker.model.checks, 0,
        "board_day2_cell06_check.png must have 0 cells to check, got mask {:#06x} (cell 06 = {:?})",
        tracker.model.checks, tracker.model.cells[5]
    );
}

#[test]
fn test_office_instigate_bestow_decision() {
    let (w, h, rgba) = load_png_rgba8(&fixture_path("board_day4_office_instigate_bestow.png"));
    let view = ImageView {
        width: w,
        height: h,
        data: &rgba,
    };
    let mut arena = ScanArena::new();
    let scan = scan_image_with_arena(&view, None, &mut arena);
    assert!(scan.ok, "board_day4_office_instigate_bestow.png must localize");
    assert_eq!(scan.warden.map(|w| w.cell), Some(0));
    assert_eq!(
        scan.decision.map(|d| d.office),
        Some(true),
        "expected office=true on Instigate/Bestow screen, got {:?}",
        scan.decision
    );

    let mut tracker = RunTracker::new();
    let (ok, _, _) =
        tracker.process_frame(&view, CaptureTrigger::Board { treasure_view: false }, &mut arena);
    assert!(ok);
    assert!(tracker.last_board_had_cards);
    assert_eq!(tracker.model.warden, 0);
    assert_eq!(tracker.model.step, BoardStep::Office);
    assert_eq!(tracker.model.checks, 0);
    let advice = tracker.model.evaluate(4000, 80, 42);
    assert!(
        advice.move_text.contains("Instigate") || advice.move_text.contains("Bestow"),
        "expected Instigate or Bestow recommendation at Office, got '{}'",
        advice.move_text
    );

    // Simulate an early mid-animation frame where "Make Your Decision" is visible
    // at y ~ 0.62..0.66*h, but the "Instigate" / "Bestow" card labels at y > 0.72*h
    // have not faded in yet.
    let mut early_rgba = rgba.clone();
    let y_cut = (h * 70) / 100;
    for y in y_cut..h {
        for x in 0..w {
            let idx = (y * w + x) * 4;
            early_rgba[idx] /= 4;
            early_rgba[idx + 1] /= 4;
            early_rgba[idx + 2] /= 4;
        }
    }
    let early_view = ImageView {
        width: w,
        height: h,
        data: &early_rgba,
    };
    let early_scan = scan_image_with_arena(&early_view, None, &mut arena);
    assert!(early_scan.ok);
    assert!(
        early_scan.decision.is_none(),
        "buttonless Make Your Decision header must return decision=None, got {:?}",
        early_scan.decision
    );
    let mut early_tracker = RunTracker::new();
    let (ok_early, _, _) = early_tracker.process_frame(
        &early_view,
        CaptureTrigger::Board { treasure_view: false },
        &mut arena,
    );
    assert!(ok_early);
    assert!(
        !early_tracker.last_board_had_cards,
        "early frame without decision cards must set last_board_had_cards=false so worker retries"
    );
    assert_eq!(
        early_tracker.model.step,
        BoardStep::Office,
        "arriving at OFFICE without decision cards must still default to BoardStep::Office, never Manage/End"
    );
    let early_advice = early_tracker.model.evaluate(4000, 80, 42);
    assert!(
        early_advice.move_text.contains("Instigate") || early_advice.move_text.contains("Bestow"),
        "even on an early frame at OFFICE, must recommend Instigate or Bestow, got '{}'",
        early_advice.move_text
    );
}

#[test]
fn test_hud_objective_and_scan_latency_benchmark() {
    let mut arena = ScanArena::new();

    // 1. Verify read_hud_objective_with_slot recovers ObjectiveId::Walk ("Move 18 tiles 5/18")
    // on board_cell06_warden.png even though the "Objective: ..." title is covered by the overlay.
    let (w, h, rgba) = load_png_rgba8(&fixture_path("board_cell06_warden.png"));
    let view = ImageView {
        width: w,
        height: h,
        data: &rgba,
    };
    let mut tracker = RunTracker::new();
    let (ok, _, _) =
        tracker.process_frame(&view, CaptureTrigger::Board { treasure_view: false }, &mut arena);
    assert!(ok);
    assert_eq!(
        tracker.model.objective,
        ObjectiveId::Walk,
        "Expected HUD objective fallback to recover ObjectiveId::Walk from 'Move 18 tiles 5/18'"
    );

    // 2. Benchmark scan_image_with_arena (and skip_days) across all 8 full-board fixtures
    let fixtures = [
        "board_cell06_warden.png",
        "board_cell10_warden.png",
        "board_day5_supreme_lion_no_jaws.png",
        "board_day13_supreme_jaws_chromatic7.png",
        "board_day2_cell06_check.png",
        "board_day4_office_instigate_bestow.png",
        "chain_consume_targets.png",
        "chain_consume_end_day_prompt.png",
    ];
    let mut sum_loc = 0.0;
    let mut sum_cells = 0.0;
    let mut sum_ocr = 0.0;
    let mut sum_total = 0.0;
    let mut sum_ocr_skip_days = 0.0;
    let mut sum_total_skip_days = 0.0;

    for name in fixtures {
        let (fw, fh, frgba) = load_png_rgba8(&fixture_path(name));
        let fview = ImageView {
            width: fw,
            height: fh,
            data: &frgba,
        };
        // Warmup & get hint
        let warm = scan_image_with_arena(&fview, None, &mut arena);
        assert!(warm.ok, "Fixture {name} should scan ok");
        let hint = warm.loc;

        let iters = 5usize;
        let mut loc_ms = 0.0;
        let mut cells_ms = 0.0;
        let mut ocr_ms = 0.0;
        let mut total_ms = 0.0;
        let mut ocr_skip_ms = 0.0;
        let mut total_skip_ms = 0.0;
        for _ in 0..iters {
            let res = scan_image_with_arena(&fview, hint, &mut arena);
            loc_ms += res.profile.loc_ms;
            cells_ms += res.profile.cells_ms;
            ocr_ms += res.profile.ocr_ms;
            total_ms += res.profile.total_ms;

            let res_skip = scan_image_with_options_and_arena(
                &fview,
                hint,
                ScanOptions { skip_days: true },
                &mut arena,
            );
            ocr_skip_ms += res_skip.profile.ocr_ms;
            total_skip_ms += res_skip.profile.total_ms;
        }
        let k = iters as f64;
        loc_ms /= k;
        cells_ms /= k;
        ocr_ms /= k;
        total_ms /= k;
        ocr_skip_ms /= k;
        total_skip_ms /= k;
        println!(
            "Scan [{name:<30}]: loc={loc_ms:5.2}ms cells={cells_ms:5.2}ms ocr={ocr_ms:5.2}ms (skip_days={ocr_skip_ms:5.2}ms) total={total_ms:5.2}ms (skip_days={total_skip_ms:5.2}ms)"
        );
        sum_loc += loc_ms;
        sum_cells += cells_ms;
        sum_ocr += ocr_ms;
        sum_total += total_ms;
        sum_ocr_skip_days += ocr_skip_ms;
        sum_total_skip_days += total_skip_ms;
    }
    let n = fixtures.len() as f64;
    println!(
        "Scan Average (8 board fixtures): loc={:.2}ms cells={:.2}ms ocr={:.2}ms (skip_days={:.2}ms) total={:.2}ms (skip_days={:.2}ms)",
        sum_loc / n,
        sum_cells / n,
        sum_ocr / n,
        sum_ocr_skip_days / n,
        sum_total / n,
        sum_total_skip_days / n
    );
}

#[test]
fn test_user_three_screenshots_office_door_and_lion_hunt_replace() {
    let uploads = [
        (
            "/home/sif/.gemini/antigravity/brain/66ed5a1e-1f00-4e28-9c58-e1b90ec71444/.user_uploaded/media_1792465130664_1e9a0342.png",
            "board_day3_doors_7_11_12.png",
        ),
        (
            "/home/sif/.gemini/antigravity/brain/66ed5a1e-1f00-4e28-9c58-e1b90ec71444/.user_uploaded/media_1792465130681_eb909aaf.png",
            "board_day4_doors_15_16_office.png",
        ),
        (
            "/home/sif/.gemini/antigravity/brain/66ed5a1e-1f00-4e28-9c58-e1b90ec71444/.user_uploaded/media_1792465130705_b1773cb3.png",
            "board_day11_cell11_lion_hunt_replace.png",
        ),
    ];
    for (src, dst_name) in uploads {
        let src_p = std::path::Path::new(src);
        let dst_p = fixture_path(dst_name);
        if src_p.exists() && !dst_p.exists() {
            let _ = std::fs::copy(src_p, &dst_p);
        }
    }

    let mut arena = ScanArena::new();
    for (_, name) in uploads {
        let path = fixture_path(name);
        let (w, h, rgba) = load_png_rgba8(&path);
        let view = ImageView {
            width: w,
            height: h,
            data: &rgba,
        };
        let res = scan_image_with_arena(&view, None, &mut arena);
        println!("\n=== {name} ({w}x{h}) ===");
        println!(
            "profile: loc={:.2}ms cells={:.2}ms ocr={:.2}ms total={:.2}ms",
            res.profile.loc_ms, res.profile.cells_ms, res.profile.ocr_ms, res.profile.total_ms
        );
        println!("warden={:?} days={:?} chromatic={:?}", res.warden, res.days, res.chromatic);
        println!("loc={:?}", res.loc);
        if let Some(loc) = res.loc {
            let (t_list, lbl) = tartarus_scan::read_tools_with_slot(&view, &loc, &mut arena.slots[0]);
            println!("read_tools label={:?} tools={:?}", lbl, t_list.as_slice());
        }
        println!("doors={:?}", res.doors);
        println!("decision={:?}", res.decision);
        println!("tools={:?}", res.tools.as_slice());
        if let Some(ref cells) = res.cells {
            for (idx, c) in cells.iter().enumerate() {
                if c.kind != tartarus_scan::ScannedCellKind::Locked {
                    println!(
                        "  cell {:02} (idx {idx}): kind={:?} beast={:?} tier={} conf={:.2} jaws={} ghost={} event={}",
                        idx + 1,
                        c.kind,
                        c.beast,
                        c.tier,
                        c.conf,
                        c.jaws,
                        c.ghost,
                        c.event
                    );
                }
            }
        }
        let mut tracker = RunTracker::new();
        tracker.ingest(&res);
        let snap = tracker.model.evaluate(96_000, 85, 42);
        println!(
            "tracker step={:?} warden={} dest_mask={:#06x} no_swap={} no_devour={} checks={:#06x}",
            tracker.model.step,
            tracker.model.warden,
            tracker.model.dest_mask,
            tracker.model.no_swap,
            tracker.model.no_devour,
            tracker.model.checks
        );
        println!("notes={:?}", tracker.last_notes);
        println!("move_text='{}' elapsed={}ms", snap.move_text, snap.elapsed_ms);
        for opt in &snap.options {
            println!("  opt: '{}' -> {}", opt.label, opt.value_text);
        }
    }

    // Now test sequential live-run tracking across all 3 screenshots via process_frame
    let mut seq_tracker = RunTracker::new();
    seq_tracker.log_days = Some((3, 20));
    let (w1, h1, rgba1) = load_png_rgba8(&fixture_path("board_day3_doors_7_11_12.png"));
    let view1 = ImageView { width: w1, height: h1, data: &rgba1 };
    let (ok1, _, prof1) = seq_tracker.process_frame(
        &view1,
        tartarus_scan::CaptureTrigger::Board { treasure_view: false },
        &mut arena,
    );
    assert!(ok1);
    assert_eq!(seq_tracker.model.tools.as_slice(), &[ToolId::Crown]);
    assert!(prof1.total_ms < 5.0, "expected scan < 5ms, got {:.2}ms", prof1.total_ms);
    let snap1 = seq_tracker.model.evaluate(96_000, 85, 42);
    assert!(snap1.elapsed_ms < 50, "expected solve < 50ms, got {}ms", snap1.elapsed_ms);
    assert_eq!(snap1.options.len(), 3);

    seq_tracker.log_days = Some((4, 20));
    let (w2, h2, rgba2) = load_png_rgba8(&fixture_path("board_day4_doors_15_16_office.png"));
    let view2 = ImageView { width: w2, height: h2, data: &rgba2 };
    let (ok2, _, prof2) = seq_tracker.process_frame(
        &view2,
        tartarus_scan::CaptureTrigger::Board { treasure_view: false },
        &mut arena,
    );
    assert!(ok2);
    assert_eq!(seq_tracker.model.tools.as_slice(), &[ToolId::Crown]);
    assert!(prof2.total_ms < 5.0, "expected scan < 5ms, got {:.2}ms", prof2.total_ms);
    let snap2 = seq_tracker.model.evaluate(96_000, 85, 42);
    assert_eq!(snap2.options.len(), 3);
    assert!(
        snap2.options.iter().any(|o| o.label.contains("Interrogation Room")),
        "expected Interrogation Room in options: {:?}",
        snap2.options
    );
    assert!(snap2.move_text.contains("Interrogation Room"));

    seq_tracker.log_days = Some((11, 20));
    let (w3, h3, rgba3) = load_png_rgba8(&fixture_path("board_day11_cell11_lion_hunt_replace.png"));
    let view3 = ImageView { width: w3, height: h3, data: &rgba3 };
    let (ok3, _, prof3) = seq_tracker.process_frame(
        &view3,
        tartarus_scan::CaptureTrigger::Board { treasure_view: false },
        &mut arena,
    );
    assert!(ok3);
    assert_eq!(
        seq_tracker.model.tools.as_slice(),
        &[ToolId::Crown, ToolId::Trap],
        "expected mid-run Trap instrument to be recognized alongside Crown"
    );
    assert!(!seq_tracker.model.no_swap, "expected Replace to be available");
    assert!(!seq_tracker.model.no_devour, "expected Consume to be available");
    assert!(prof3.total_ms < 5.0, "expected scan < 5ms, got {:.2}ms", prof3.total_ms);
    let snap3 = seq_tracker.model.evaluate(96_000, 85, 42);
    assert_eq!(snap3.options.len(), 4, "expected Hunt, Replace, Consume, and End options");
    assert!(snap3.options.iter().any(|o| o.label.starts_with("Hunt:")));
    assert!(snap3.options.iter().any(|o| o.label.starts_with("Replace ")));
    assert!(snap3.options.iter().any(|o| o.label.starts_with("Consume ")));
    assert!(snap3.options.iter().any(|o| o.label == "End the day"));
}

