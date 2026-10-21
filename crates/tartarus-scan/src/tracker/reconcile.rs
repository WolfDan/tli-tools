//! Frame-to-board reconciliation, tool panel self-healing, jaws consistency checking, and ghost cell search.

use super::{RunTracker, ScanReading};
use crate::cv::{ScanResult, ScannedCellKind, ScannedTool};
use tartarus_core::{
    BEASTS, Beast, BoardModel, BoardRunStage, BoardStep, CHALLENGE_DAYS, CellEvent, CellKind,
    CellModel, FixedVec, GameMode, OFFICE, ObjectiveId, PendingKind, SUPREME, TOOL_SURE, ToolId,
    ToolsPrompt, TreasurePhase,
};

impl RunTracker {
    #[inline]
    fn scale_step(scale: Option<f64>) -> u16 {
        scale
            .map(|s| (s * 20.0).round().clamp(0.0, u16::MAX as f64) as u16)
            .unwrap_or(0)
    }

    fn kept_tusk(base: &BoardModel, n: usize, beast: Beast, tier: u8) -> bool {
        let was = &base.cells[n];
        was.kind == CellKind::Inmate && was.ivory && was.beast == beast && tier >= was.tier
    }

    fn run_tools(
        &self,
        read: &[ScannedTool],
        base: &BoardModel,
        m: &mut BoardModel,
        notes: &mut Vec<String>,
        scale: Option<f64>,
    ) -> FixedVec<ScannedTool, 6> {
        let same_run = m.day >= base.day;
        let known: &[ToolId] = if same_run { &base.tools } else { &[] };
        let offer: &[ToolId] = if same_run
            && base.step == BoardStep::Choice
            && base.choice.kind == PendingKind::Tool
        {
            base.choice.slice()
        } else {
            &[]
        };

        let mut out: FixedVec<ScannedTool, 6> = known
            .iter()
            .take(6)
            .map(|&sim| ScannedTool {
                id: sim.as_str(),
                sim,
                name: sim.name(),
                score: 1.0,
                conf: 1.0,
            })
            .collect();

        let has = |list: &[ScannedTool], id: ToolId| list.iter().any(|t| t.sim == id);
        let slots = read.len();

        m.tools_prompt = ToolsPrompt::None;
        m.tools_slots = 0;
        m.tools_scale_step = Self::scale_step(scale);
        let phantom = self
            .phantom_slots
            .get(&m.tools_scale_step)
            .copied()
            .unwrap_or(0);
        let extra = m.extra_slots.max(phantom);

        if same_run
            && (!known.is_empty() || !offer.is_empty() || base.stage == BoardRunStage::Opening)
        {
            // Self-heal any existing slot if a much higher confidence read disagrees
            // (e.g. when a transient overlay occlusion caused an initial misread).
            for i in 0..out.len().min(read.len()) {
                if read[i].sim != out[i].sim
                    && read[i].conf >= 0.95
                    && read[i].score >= 0.94
                    && !has(&out, read[i].sim)
                {
                    out[i] = read[i];
                }
            }

            if !offer.is_empty() && slots > known.len() && !offer.iter().any(|&id| has(&out, id)) {
                let mut seen: FixedVec<ScannedTool, 6> = read
                    .iter()
                    .copied()
                    .filter(|t| offer.contains(&t.sim))
                    .take(6)
                    .collect();
                seen.as_mut_slice()
                    .sort_by(|a, b| b.conf.partial_cmp(&a.conf).unwrap());
                let advised = self.last_advised_tool.filter(|id| offer.contains(id));
                let take = seen.first().map(|t| t.sim).or(advised);
                if let Some(t_id) = take {
                    let _ = out.try_push(ScannedTool {
                        id: t_id.as_str(),
                        sim: t_id,
                        name: t_id.name(),
                        score: 1.0,
                        conf: 1.0,
                    });
                    if seen.is_empty() {
                        notes.push(format!(
                            "Took it you chose {}, as advised. If you took another, change it under Torture Instrument.",
                            t_id.name()
                        ));
                    }
                } else {
                    let offer_names: Vec<&str> = offer.iter().map(|id| id.name()).collect();
                    notes.push(format!(
                        "Your instruments show one more than before, from the offer of {}: set which you took under Torture Instrument.",
                        offer_names.join(", ")
                    ));
                }
            } else if offer.is_empty() && slots > out.len() {
                // Mid-run milestone / treasure instrument acquired without a prior Pick screen capture:
                // accept any newly scanned high-confidence tool slots directly.
                for t in &read[out.len()..] {
                    if !has(&out, t.sim) && (t.conf as f64) >= TOOL_SURE && t.score >= 0.88 {
                        let _ = out.try_push(*t);
                    }
                }
            }

            let off_tools: FixedVec<ToolId, 8> = read
                .iter()
                .filter(|t| (t.conf as f64) >= TOOL_SURE && !has(&out, t.sim))
                .map(|t| t.sim)
                .take(8)
                .collect();
            if slots > out.len() + extra {
                m.tools_slots = slots;
                m.tools_prompt = ToolsPrompt::CountMismatch {
                    panel_slots: slots as u8,
                };
            } else if !off_tools.is_empty() {
                m.tools_prompt = ToolsPrompt::Unexpected { off: off_tools };
            }
            return out;
        }

        for t in read {
            if !has(&out, t.sim) && (t.conf as f64) >= TOOL_SURE {
                let _ = out.try_push(*t);
            }
        }
        if !read.is_empty() {
            m.tools_slots = read.len();
            m.tools_prompt = ToolsPrompt::UnknownRun {
                panel_slots: read.len() as u8,
            };
        }
        out
    }

    fn chain_to_model(&self, r: &ScanResult, base: &BoardModel) -> ScanReading {
        let mut m = *base;
        let mut checks = 0u16;
        let mut notes = Vec::new();
        let cells = r.cells.as_ref().unwrap();
        let chain = r.chain.as_ref().unwrap();

        let warden = r.warden.map_or(base.warden, |w| w.cell);
        let after = base.warden == warden
            && (base.step == BoardStep::Manage || base.step == BoardStep::Chain);
        let walked = base.step == BoardStep::Patrol && base.has_dest(warden);

        if !after && !walked {
            for (n, c) in cells.iter().enumerate() {
                if n == (OFFICE as usize) || n == (warden as usize) {
                    continue;
                }
                match c.kind {
                    ScannedCellKind::Locked => {
                        m.cells[n] = CellModel::sealed();
                    }
                    ScannedCellKind::Inmate => {
                        let tier = c.tier.min(4);
                        let mut cm = CellModel::sealed();
                        cm.kind = CellKind::Inmate;
                        cm.beast = c.beast;
                        cm.tier = tier;
                        cm.ivory = Self::kept_tusk(base, n, c.beast, tier);
                        m.cells[n] = cm;
                    }
                    _ => {
                        let mut cm = CellModel::sealed();
                        cm.kind = CellKind::Empty;
                        m.cells[n] = cm;
                    }
                }
                if c.kind != ScannedCellKind::Locked {
                    checks |= 1u16 << n;
                }
            }
            notes.push("A consume's meal pick darkens the board, so its rarities were guessed: check the marked cells.".to_string());
        }

        m.warden = warden;
        let w_idx = warden as usize;
        let was = base.cells[w_idx];
        let eater = &cells[w_idx];
        if eater.kind == ScannedCellKind::Inmate {
            let mut cm = CellModel::sealed();
            cm.kind = CellKind::Inmate;
            cm.beast = eater.beast;
            cm.tier = eater.tier.min(4);
            cm.ivory = was.ivory;
            cm.prey = if was.kind == CellKind::Inmate {
                was.prey
            } else {
                None
            };
            m.cells[w_idx] = cm;
        } else {
            checks |= 1u16 << w_idx;
            notes.push(
                "The eater in the warden's cell wasn't read: set it as it is now.".to_string(),
            );
        }

        let tier = if m.cells[w_idx].kind == CellKind::Inmate {
            m.cells[w_idx].tier
        } else {
            0
        };

        let mut eaten = 0usize;
        if after || walked {
            for (n, c) in cells.iter().enumerate() {
                if n == (OFFICE as usize)
                    || n == w_idx
                    || c.kind != ScannedCellKind::Locked
                    || m.cells[n].kind != CellKind::Inmate
                {
                    continue;
                }
                m.cells[n] = CellModel::sealed();
                eaten += 1;
            }
        }

        for &target_n in &chain.targets {
            let n = target_n as usize;
            let c = m.cells[n];
            if c.kind == CellKind::Scratch {
                continue;
            }
            if c.kind == CellKind::Inmate && c.tier <= tier && c.beast == cells[n].beast {
                continue;
            }
            let mut cm = CellModel::sealed();
            cm.kind = CellKind::Inmate;
            cm.beast = cells[n].beast;
            cm.tier = tier;
            cm.ivory = c.kind == CellKind::Inmate && c.ivory;
            m.cells[n] = cm;
            checks |= 1u16 << n;
        }

        let _ = (eaten, walked);
        m.step = BoardStep::Chain;
        m.devour_used = true;
        m.no_devour = false;
        m.no_swap = false;
        m.treasure = TreasurePhase::Closed;
        m.clear_dest();
        m.chain_armed = false;
        m.stage = BoardRunStage::InProgress;

        let tools: FixedVec<ScannedTool, 6> = m
            .tools
            .iter()
            .take(6)
            .map(|&id| ScannedTool {
                id: id.as_str(),
                sim: id,
                name: id.name(),
                score: 1.0,
                conf: 1.0,
            })
            .collect();

        ScanReading {
            chromatic: m.vault.len(),
            model: m,
            checks,
            notes,
            tools,
            chain_targets: Some(chain.targets),
        }
    }

    pub fn scan_to_model(&self, r: &ScanResult) -> ScanReading {
        let base = &self.model;
        if r.chain.is_some() {
            return self.chain_to_model(r, base);
        }

        let mut m = *base;
        m.stage = BoardRunStage::InProgress;
        let mut checks = 0u16;
        let mut notes = Vec::new();
        let cells = r.cells.as_ref().unwrap();

        if let Some((cur, max)) = self.log_days.filter(|&(_, max)| max > 0) {
            let gt_left = max.saturating_sub(cur.max(1));
            if m.mode == GameMode::Challenge {
                m.day = CHALLENGE_DAYS
                    .saturating_sub(gt_left)
                    .clamp(1, CHALLENGE_DAYS);
            } else {
                m.max_days = max.clamp(1, 60);
                m.day = m.max_days.saturating_sub(gt_left).max(1);
            }
        } else if let Some((left, _conf)) = r.days {
            if m.mode == GameMode::Challenge {
                m.day = CHALLENGE_DAYS.saturating_sub(left).clamp(1, CHALLENGE_DAYS);
            } else {
                if left >= m.max_days {
                    m.max_days = (left + 1).min(60);
                }
                m.day = m.max_days.saturating_sub(left).max(1);
            }
        } else {
            notes.push("The days until the riot weren't read: check the day.".to_string());
        }

        for (n, c) in cells.iter().enumerate() {
            if n == (OFFICE as usize) {
                continue;
            }
            match c.kind {
                ScannedCellKind::Locked => {
                    if c.nest {
                        let mut cm = CellModel::sealed();
                        cm.kind = CellKind::Nest;
                        cm.due = m.day + 2;
                        m.cells[n] = cm;
                        checks |= 1u16 << n;
                    } else {
                        let was = base.cells[n];
                        let event = if c.event {
                            CellEvent::Unknown
                        } else {
                            CellEvent::None
                        };
                        if was.kind == CellKind::Revealed {
                            let mut cm = CellModel::sealed();
                            cm.kind = CellKind::Revealed;
                            cm.tier = was.tier;
                            cm.event = event;
                            m.cells[n] = cm;
                        } else if was.kind == CellKind::Sealed && was.at_least > 0 {
                            let mut cm = CellModel::sealed();
                            cm.event = event;
                            cm.at_least = was.at_least;
                            m.cells[n] = cm;
                        } else {
                            let mut cm = CellModel::sealed();
                            cm.event = event;
                            m.cells[n] = cm;
                        }
                    }
                    if c.conf < 0.60 {
                        checks |= 1u16 << n;
                    }
                }
                ScannedCellKind::Inmate => {
                    let tier = c.tier.min(4);
                    let mut cm = CellModel::sealed();
                    cm.kind = CellKind::Inmate;
                    cm.beast = c.beast;
                    cm.tier = tier;
                    cm.ivory = Self::kept_tusk(base, n, c.beast, tier);
                    cm.event = if c.event {
                        CellEvent::Unknown
                    } else {
                        CellEvent::None
                    };
                    m.cells[n] = cm;
                    if c.conf < 0.60 || c.ghost {
                        checks |= 1u16 << n;
                    }
                }
                _ => {
                    let mut cm = CellModel::sealed();
                    cm.kind = CellKind::Empty;
                    m.cells[n] = cm;
                    checks |= 1u16 << n;
                }
            }
        }

        if let Some(w) = r.warden {
            m.warden = w.cell;
            if w.conf < 0.60 {
                checks |= 1u16 << (w.cell as usize);
            }
        } else {
            notes.push("The warden's arrow wasn't found: set the warden.".to_string());
        }

        if let Some(ref doors) = r.doors {
            if !doors.cells.is_empty() {
                m.step = BoardStep::Patrol;
                m.clear_dest();
                for &n in &doors.cells {
                    if n != m.warden {
                        m.set_dest(n, true);
                    }
                }
                let two_door_tool =
                    |id: ToolId| matches!(id, ToolId::Forged | ToolId::Whip | ToolId::Stake);
                let has_two_door_limit = base.tools.iter().copied().any(two_door_tool)
                    || r.tools
                        .iter()
                        .any(|t| (t.conf as f64) >= TOOL_SURE && two_door_tool(t.sim));
                let expected_doors = if has_two_door_limit { 2 } else { 3 };
                if doors.conf < 0.60 || doors.cells.len() < expected_doors {
                    notes.push("Today's doors were hard to read: check them.".to_string());
                }
            } else {
                m.step = BoardStep::Manage;
                m.clear_dest();
                m.devour_used = false;
                m.swap_used = false;
                m.hunt_used = false;
            }
        } else {
            m.step = BoardStep::Manage;
            m.clear_dest();
            m.devour_used = false;
            m.swap_used = false;
            m.hunt_used = false;
        }

        m.chain_armed = false;
        m.consume_offered = false;
        m.treasure = TreasurePhase::Closed;
        m.no_swap = false;
        m.no_devour = false;

        let same_visit = m.day == base.day
            && m.warden == base.warden
            && matches!(
                base.step,
                BoardStep::Manage | BoardStep::Choice | BoardStep::Event
            );
        if r.decision.is_none() && same_visit && m.step == BoardStep::Manage {
            m.swap_used = base.swap_used;
            m.devour_used = base.devour_used;
            m.hunt_used = base.hunt_used;
            m.no_swap = base.no_swap;
            m.no_devour = base.no_devour;
            m.consume_offered = base.consume_offered;
            let w_idx = m.warden as usize;
            let was = base.cells[w_idx];
            let now = m.cells[w_idx];
            if !m.devour_used
                && was.kind == CellKind::Inmate
                && now.kind == CellKind::Inmate
                && (was.beast != now.beast || was.tier != now.tier)
            {
                m.swap_used = true;
                m.consume_offered = true;
                notes.push(
                    "The warden's inmate changed since the board before, so today's Replace is taken as done."
                        .to_string(),
                );
                m.cells[w_idx].ivory = false;
            }
        }

        if let Some(dec) = r.decision {
            if dec.office {
                m.step = BoardStep::Office;
                m.clear_dest();
            } else if dec.ratway {
                m.step = BoardStep::Ratway;
                m.clear_dest();
            } else if m.warden == OFFICE && !dec.end {
                m.step = BoardStep::Office;
                m.clear_dest();
            } else {
                m.swap_used = false;
                m.devour_used = false;
                m.hunt_used = false;
                if !same_visit && m.cells[m.warden as usize].kind == CellKind::Inmate {
                    // On the first decision upon entering an Inmate cell, Replace is always offered by game rules.
                    m.no_swap = false;
                    if !dec.replace || !dec.end {
                        notes.push(
                            "Today's decision cards were still appearing: check them.".to_string(),
                        );
                    }
                } else if !dec.end {
                    // Mid-animation frame where "End" hasn't faded in yet: preserve prior no_swap state and re-probe.
                    m.no_swap = base.no_swap;
                    notes.push(
                        "Today's decision cards were still appearing: check them.".to_string(),
                    );
                } else {
                    m.no_swap = !dec.replace;
                }
                m.no_devour = !dec.consume;
                m.consume_offered = dec.consume;
            }
        } else if m.warden == OFFICE && m.step == BoardStep::Manage {
            let already_acted_in_office =
                m.day == base.day && base.warden == OFFICE && base.step == BoardStep::Manage;
            if !already_acted_in_office {
                m.step = BoardStep::Office;
                m.clear_dest();
            }
        }

        let mut tools = self.run_tools(&r.tools, base, &mut m, &mut notes, r.loc.map(|l| l.s));
        if r.decision.is_some_and(|d| d.hunt)
            && !tools.iter().any(|t| t.sim == ToolId::Trap)
            && tools.len() < 6
        {
            tools.push(ScannedTool {
                id: ToolId::Trap.as_str(),
                sim: ToolId::Trap,
                name: ToolId::Trap.name(),
                score: 1.0,
                conf: 1.0,
            });
        }
        m.tools = tools.iter().map(|t| t.sim).take(32).collect();
        if tools.iter().any(|t| (t.conf as f64) < TOOL_SURE) {
            notes.push(
                "Some instruments were hard to tell apart: check them under Torture Instrument."
                    .to_string(),
            );
        }

        let same_run = m.day >= base.day;
        let chromatic = r
            .chromatic
            .unwrap_or_else(|| if same_run { base.vault.len() } else { 0 });
        m.vault = std::iter::repeat_n(Beast::Wolf, chromatic.min(16)).collect();
        let highest = m
            .cells
            .iter()
            .filter(|c| c.kind == CellKind::Inmate)
            .map(|c| c.tier)
            .max()
            .unwrap_or(0);
        if m.objective == ObjectiveId::King {
            m.milestone = highest >= SUPREME || chromatic > 0;
        }

        let has_target = |state: &tartarus_core::SimState, n: usize| -> bool {
            !state.cells[n].sealed
                && state.cells[n].inmate.is_some()
                && state.devour_options(n as u8).1 > 0
        };

        let mut trial_final = m;
        trial_final.step = BoardStep::Manage;
        let (mut final_st, _) = trial_final.build_state(1);

        let mut ghosts = [0usize; 16];
        let mut glen = 0usize;
        for (n, c) in cells.iter().enumerate() {
            if c.ghost && m.cells[n].kind == CellKind::Inmate {
                ghosts[glen] = n;
                glen += 1;
            }
        }

        if (1..=3).contains(&glen) {
            let combos = 1usize << (2 * glen);
            let mut best_combo: Option<([Beast; 3], usize, usize)> = None;
            for mask in 0..combos {
                let mut assign = [Beast::Mammoth; 3];
                let mut kept_orig = 0usize;
                for i in 0..glen {
                    let g = ghosts[i];
                    let b = BEASTS[(mask >> (2 * i)) & 3];
                    assign[i] = b;
                    if b == cells[g].beast {
                        kept_orig += 1;
                    }
                    if let Some(inm) = final_st.cells[g].inmate.as_mut() {
                        inm.beast = b;
                    }
                }
                let fit = cells
                    .iter()
                    .enumerate()
                    .filter(|&(k, d)| {
                        d.kind == ScannedCellKind::Inmate && has_target(&final_st, k) == d.jaws
                    })
                    .count();
                if best_combo.is_none_or(|(_, b_fit, b_orig)| {
                    fit > b_fit || (fit == b_fit && kept_orig > b_orig)
                }) {
                    best_combo = Some((assign, fit, kept_orig));
                }
            }
            if let Some((assign, _, _)) = best_combo {
                for i in 0..glen {
                    let g = ghosts[i];
                    m.cells[g].beast = assign[i];
                    if let Some(inm) = final_st.cells[g].inmate.as_mut() {
                        inm.beast = assign[i];
                    }
                }
            }
        } else if glen > 3 {
            for &n in &ghosts[..glen] {
                let mut best: Option<(Beast, usize)> = None;
                for beast in BEASTS {
                    if let Some(inm) = final_st.cells[n].inmate.as_mut() {
                        inm.beast = beast;
                    }
                    let fit = cells
                        .iter()
                        .enumerate()
                        .filter(|&(k, d)| {
                            d.kind == ScannedCellKind::Inmate && has_target(&final_st, k) == d.jaws
                        })
                        .count();
                    if best.is_none_or(|b| fit > b.1) {
                        best = Some((beast, fit));
                    }
                }
                if let Some((best_beast, _)) = best {
                    m.cells[n].beast = best_beast;
                    if let Some(inm) = final_st.cells[n].inmate.as_mut() {
                        inm.beast = best_beast;
                    }
                }
            }
        }

        let mut odd = 0usize;
        for (n, c) in cells.iter().enumerate() {
            if c.kind != ScannedCellKind::Inmate || m.cells[n].kind != CellKind::Inmate {
                continue;
            }
            if c.tier < SUPREME && has_target(&final_st, n) != c.jaws {
                checks |= 1u16 << n;
                odd += 1;
            }
        }
        if odd > 0 {
            notes.push(if odd == 1 {
                "1 inmate's jaws marker doesn't fit what was read: check it and the inmates around."
                    .to_string()
            } else {
                format!("{odd} inmates' jaws markers don't fit what was read: check them and the inmates around.")
            });
        }

        ScanReading {
            model: m,
            checks,
            notes,
            chromatic,
            tools,
            chain_targets: None,
        }
    }
}
