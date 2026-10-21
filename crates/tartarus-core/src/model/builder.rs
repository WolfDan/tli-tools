use crate::engine::{
    BEASTS, Beast, CELL_COUNT, CellState, ChoiceId, Contract, ContractId, DeliveryRecord,
    EscortKind, EventKind, GameMode, GiftId, Inmate, OFFICE, PendingChoice, PendingKind, Phase,
    PreyLink, RAINBOW, START_VOUCHERS, SUPREME, SimState, ToolId, ToolMask, UsedToday, VaultEntry,
    cell_label, next_rand, steps_between,
};

use super::board::{
    BoardModel, BoardRunStage, BoardStep, CellEvent, CellKind, CellModel, ChoiceModel,
    ContractStatus, FixedVec, PreyMark,
};

impl BoardModel {
    pub fn build_state(&self, seed: u32) -> (SimState, Vec<String>) {
        let mut problems = Vec::new();
        let challenge = self.mode == GameMode::Challenge;
        let mut st = SimState::new_blank(self.mode, self.rewards, self.max_days);
        let mut rng = if seed == 0 { 0x1234_5678 } else { seed };
        st.rng = (next_rand(&mut rng) * 4294967296.0) as u32;
        st.day = self.day.clamp(1, st.max_days);
        st.next_id = 1;

        for n in 0..CELL_COUNT {
            let c = &self.cells[n];
            let mut cell = CellState {
                n: n as u8,
                sealed: n != (OFFICE as usize),
                inmate: None,
                hidden: None,
                known: 0,
                at_least: 0,
                scratch: false,
                nest_due: None,
                event: None,
                event_known: true,
                ivory: false,
                seen: false,
            };
            if n == (OFFICE as usize) {
                st.cells[n] = cell;
                continue;
            }
            let apply_ev = |cell: &mut CellState, ev: CellEvent| {
                cell.seen = ev.is_some();
                match ev {
                    CellEvent::None => {}
                    CellEvent::Unknown => {
                        cell.event = Some(EventKind::Chain);
                        cell.event_known = false;
                    }
                    CellEvent::Known(k) => {
                        cell.event = Some(k);
                        cell.event_known = true;
                    }
                }
            };

            match c.kind {
                CellKind::Office => {
                    cell.sealed = false;
                }
                CellKind::Empty => {
                    cell.sealed = false;
                }
                CellKind::Inmate => {
                    cell.sealed = false;
                    let id = st.next_id;
                    st.next_id += 1;
                    cell.inmate = Some(Inmate {
                        id,
                        beast: c.beast,
                        tier: c.tier.min(SUPREME),
                    });
                    cell.ivory = c.ivory;
                    apply_ev(&mut cell, c.event);
                }
                CellKind::Nest => {
                    cell.nest_due = Some(c.due.max(st.day + 1));
                }
                CellKind::Scouted => {
                    cell.known = 2;
                    let id = st.next_id;
                    st.next_id += 1;
                    cell.hidden = Some(Inmate {
                        id,
                        beast: c.beast,
                        tier: c.tier.min(SUPREME),
                    });
                    apply_ev(&mut cell, c.event);
                }
                CellKind::Scratch | CellKind::Revealed => {
                    cell.scratch = c.kind == CellKind::Scratch;
                    cell.known = 1;
                    let id = st.next_id;
                    st.next_id += 1;
                    let b_idx = (next_rand(&mut rng) * 4.0).floor() as usize;
                    cell.hidden = Some(Inmate {
                        id,
                        beast: BEASTS[b_idx.min(3)],
                        tier: c.tier.min(SUPREME),
                    });
                    apply_ev(&mut cell, c.event);
                }
                CellKind::Sealed => {
                    cell.at_least = c.at_least.min(SUPREME);
                    apply_ev(&mut cell, c.event);
                }
            }
            st.cells[n] = cell;
        }

        // Prey marks
        st.prey_len = 0;
        for (n, c) in self.cells.iter().enumerate() {
            if c.kind != CellKind::Inmate || c.beast != Beast::Lion {
                continue;
            }
            let Some(prey) = c.prey else {
                continue;
            };
            let Some(lion) = st.cells[n].inmate else {
                continue;
            };
            let target_inmate = st.cells.get(prey.cell as usize).and_then(|tc| tc.inmate);
            let Some(target) = target_inmate else {
                problems.push(format!(
                    "The prey mark from cell {} needs a same-rarity meal for the Lion in cell {}.",
                    cell_label(n as u8),
                    cell_label(prey.cell)
                ));
                continue;
            };
            if !st.can_eat(lion, target) {
                problems.push(format!(
                    "The prey mark from cell {} needs a same-rarity meal for the Lion in cell {}.",
                    cell_label(n as u8),
                    cell_label(prey.cell)
                ));
                continue;
            }
            let already = st.prey_slice().iter().any(|l| {
                l.lion == target.id || l.prey == target.id || l.lion == lion.id || l.prey == lion.id
            });
            if already {
                problems.push(format!(
                    "Cell {} already has a prey mark.",
                    cell_label(prey.cell)
                ));
                continue;
            }
            if (st.prey_len as usize) < st.prey.len() {
                st.prey[st.prey_len as usize] = PreyLink {
                    lion: lion.id,
                    prey: target.id,
                    prey_beast: target.beast,
                    rounds: prey.rounds.clamp(1, 9),
                    next_day: st.day + 1,
                };
                st.prey_len += 1;
            }
        }

        // Vault
        st.vault_len = 0;
        for &beast in &self.vault {
            if (st.vault_len as usize) < st.vault.len() {
                let id = st.next_id;
                st.next_id += 1;
                st.vault[st.vault_len as usize] = VaultEntry { id, beast, day: 1 };
                st.vault_len += 1;
            }
        }
        st.cache = st.vault_len;

        // Tools
        st.tools_mask = ToolMask(0);
        st.tools_len = 0;
        for &t in &self.tools {
            st.add_tool(t);
        }

        st.position = self.warden.min(15);
        st.chain_armed = self.chain_armed;
        let need = self.objective.need();
        st.objective_id = self.objective;
        st.objective_progress = if need > 0 {
            self.walk_tiles.min(need)
        } else {
            0
        };
        st.start_event = self.start_event;
        st.milestone = self.milestone || (need > 0 && st.objective_progress >= need);

        let mut max_tier = if st.vault_len > 0 { RAINBOW } else { 0 };
        for c in &st.cells {
            if let Some(inm) = c.inmate
                && inm.tier > max_tier
            {
                max_tier = inm.tier;
            }
        }
        st.stats.best = max_tier;
        st.used = UsedToday::default();
        st.dest_len = 0;
        st.active_event = None;
        st.pending_len = 0;
        st.phase = Phase::Manage;

        if challenge && let Some(ch) = st.challenge.as_mut() {
            ch.vouchers = self.ch.vouchers.min(START_VOUCHERS);
            let deadlines = [4u8, 8u8, 12u8];
            for slot in 0..3 {
                let cid = self.ch.contracts[slot];
                ch.contracts[slot] = Contract {
                    slot: slot as u8,
                    id: cid,
                    deadline: deadlines[slot],
                    heavy: slot == 2,
                    bounty: if cid == ContractId::Bounty {
                        Some(self.ch.bounty)
                    } else {
                        None
                    },
                };
            }
            ch.deliveries_len = 0;
            ch.score = 0;
            for slot in 0..2 {
                let st_s = self.ch.status[slot];
                if st_s == ContractStatus::Pending {
                    break;
                }
                let success = st_s == ContractStatus::Delivered;
                let pts = if success {
                    self.ch.points[slot].min(999)
                } else {
                    0
                };
                ch.deliveries[ch.deliveries_len as usize] = DeliveryRecord {
                    slot: slot as u8,
                    day: deadlines[slot],
                    success,
                    points: pts,
                    cost: 0,
                    pressure: 0,
                    escort: if success {
                        Some(EscortKind::Light)
                    } else {
                        None
                    },
                    early: false,
                };
                ch.deliveries_len += 1;
                ch.score += pts;
            }
            if self.ch.status[0] == ContractStatus::Pending
                && self.ch.status[1] != ContractStatus::Pending
            {
                problems.push("Contract 2 can only be settled after contract 1.".to_string());
            }
            if (ch.deliveries_len as usize) < 2 {
                let next = ch.contracts[ch.deliveries_len as usize];
                if st.day > next.deadline {
                    problems.push(format!(
                        "\"{}\" was due at the end of day {}. Mark it delivered or skipped.",
                        next.id.name(),
                        next.deadline
                    ));
                }
            }
            ch.window_open = false;
            ch.window_early = false;
        }

        let set_used = |st: &mut SimState| {
            st.used = UsedToday {
                swap: u8::from(self.swap_used),
                devour: u8::from(self.devour_used),
                hunt: self.hunt_used,
                devour_after_swap: self.consume_offered,
                no_swap: self.no_swap,
                no_devour: self.no_devour,
            };
        };

        match self.step {
            BoardStep::Patrol => {
                st.phase = Phase::Patrol;
                let mut dlen = 0usize;
                for step in 1..16u8 {
                    let n = (st.position + step) & 15;
                    if self.has_dest(n) {
                        st.destinations[dlen] = n;
                        dlen += 1;
                    }
                }
                st.dest_len = dlen as u8;
                if st.dest_len == 0 {
                    problems.push("Mark today's doors: tick them under \u{201c}Which door to patrol\u{201d}, or select a cell and press Today's door.".to_string());
                }
                let range = st.params.patrol_range;
                let reach = |w: u8, n: u8| {
                    let s = steps_between(w, n);
                    s >= 1 && s <= range
                };
                let mut outside: FixedVec<&'static str, 16> = FixedVec::new();
                for n in 0..16u8 {
                    if self.has_dest(n) && !reach(st.position, n) {
                        outside.push(cell_label(n));
                    }
                }
                const TUTORIAL_MASK: u16 = (1u16 << 5) | (1u16 << 7) | (1u16 << 13);
                let tutorial = st.position == 4 && self.dest_mask == TUTORIAL_MASK;
                if !outside.is_empty() && !tutorial {
                    let mut fits: FixedVec<&'static str, 16> = FixedVec::new();
                    for w in 0..16u8 {
                        if (0..16u8).all(|n| !self.has_dest(n) || reach(w, n)) {
                            fits.push(cell_label(w));
                        }
                    }
                    let where_str = if fits.len() > 1 {
                        format!(
                            "{} or {}",
                            fits[..fits.len() - 1].join(", "),
                            fits[fits.len() - 1]
                        )
                    } else if fits.len() == 1 {
                        fits[0].to_string()
                    } else {
                        String::new()
                    };
                    let out_str = if outside.len() == 1 {
                        format!("cell {} isn't", outside[0])
                    } else {
                        format!("cells {} aren't", outside.join(" and "))
                    };
                    let suffix = if !fits.is_empty() {
                        format!(
                            "Check the warden's cell: with these doors it's in cell {where_str}."
                        )
                    } else {
                        "No single cell has all these doors in reach, so check the doors too."
                            .to_string()
                    };
                    problems.push(format!(
                        "Doors are always among the {range} cells clockwise ahead of the warden, and {out_str}: the warden is set in cell {}. {suffix}",
                        cell_label(st.position)
                    ));
                }
            }
            BoardStep::Ratway => {
                st.phase = Phase::Patrol;
                st.ratway = true;
                for i in 1..16u8 {
                    st.destinations[(i - 1) as usize] = i;
                }
                st.dest_len = 15;
            }
            BoardStep::Event => {
                let here = &st.cells[st.position as usize];
                if here.sealed || here.inmate.is_none() {
                    problems.push("An event needs the inmate that came out of the door in the warden's cell: put the warden there and set its inmate.".to_string());
                }
                st.phase = Phase::Event;
                st.active_event = Some((self.event_kind, st.position));
            }
            BoardStep::Choice => {
                set_used(&mut st);
                let kind = self.choice.kind;
                let mut opts = [ChoiceId::Tool(ToolId::Revenge); 4];
                let mut olen = 0usize;
                if kind == PendingKind::Gift {
                    opts = [
                        ChoiceId::Gift(GiftId::Ring),
                        ChoiceId::Gift(GiftId::Horn),
                        ChoiceId::Gift(GiftId::Wine),
                        ChoiceId::Gift(GiftId::Brand),
                    ];
                    olen = 4;
                } else {
                    for &t in self.choice.slice() {
                        if !st.has_tool(t) && !opts[..olen].contains(&ChoiceId::Tool(t)) && olen < 4
                        {
                            opts[olen] = ChoiceId::Tool(t);
                            olen += 1;
                        }
                    }
                }
                if olen == 0 {
                    problems.push("Pick the tools on offer.".to_string());
                }
                if kind == PendingKind::Refit && !challenge {
                    problems.push("Refits only follow contract deliveries. Pick the contract challenge, or another kind of offer.".to_string());
                }
                let slot = if challenge {
                    st.challenge
                        .as_ref()
                        .map(|c| c.deliveries_len.saturating_sub(1))
                        .unwrap_or(0)
                } else {
                    0
                };
                st.pending[0] = PendingChoice {
                    kind,
                    options: opts,
                    options_len: olen as u8,
                    slot,
                    refreshed: false,
                    rerolls: if kind == PendingKind::Tool {
                        self.choice.rerolls
                    } else {
                        0
                    },
                };
                st.pending_len = 1;
            }
            BoardStep::Window => {
                if !challenge {
                    problems.push("Deliveries are part of the contract challenge. Switch the goal to Contract challenge.".to_string());
                } else if let Some(ch) = st.challenge.as_mut() {
                    let early = self.window_early;
                    if (ch.deliveries_len as usize) >= 3 {
                        problems.push("Every contract is already settled.".to_string());
                    } else {
                        let next = ch.contracts[ch.deliveries_len as usize];
                        if early && !next.heavy {
                            problems.push("The early window is only for the heavy contract, once the first two are settled.".to_string());
                        } else if early && st.day < 9 {
                            problems.push(
                                "The heavy contract can be delivered early from day 9.".to_string(),
                            );
                        }
                    }
                    ch.window_open = true;
                    ch.window_early = early;
                }
            }
            BoardStep::Chain => {
                set_used(&mut st);
                st.used.devour = 1;
                let here = &st.cells[st.position as usize];
                if let Some(inmate) = here.inmate.filter(|_| !here.sealed) {
                    st.chaining_eater = Some(inmate.id);
                    let (_, clen) = st.chain_options();
                    if clen == 0 {
                        problems.push("Your game asks for a meal only when the chaining inmate has one to eat. Check the board: it can't find a meal for it.".to_string());
                    }
                } else {
                    problems.push("A chain needs its eater in the warden's cell: put the warden there and set the inmate as it is now.".to_string());
                }
            }
            BoardStep::Office => {
                set_used(&mut st);
                if st.position != OFFICE {
                    problems.push("Instigate and Bestow come with the warden in the office: put the warden in cell 01.".to_string());
                }
                st.phase = Phase::Office;
            }
            BoardStep::Manage => {
                set_used(&mut st);
            }
        }

        (st, problems)
    }

    pub fn from_state(st: &SimState, prev: Option<&BoardModel>) -> Self {
        let mut m = Self::blank();
        m.mode = st.mode;
        m.rewards = st.params.rewards;
        m.day = st.day;
        m.max_days = if st.mode == GameMode::Challenge {
            prev.map(|p| p.max_days).unwrap_or(20)
        } else {
            st.max_days
        };
        m.warden = st.position;
        m.chain_armed = st.chain_armed;
        m.objective = st.objective_id;
        m.start_event = st.start_event;
        m.walk_tiles = st.objective_progress;
        m.milestone = st.milestone;
        m.tools = st.tools_slice().iter().copied().take(32).collect();
        m.vault = st.vault_slice().iter().map(|v| v.beast).take(16).collect();

        for n in 0..CELL_COUNT {
            if n == (OFFICE as usize) {
                m.cells[n] = CellModel::office();
                continue;
            }
            let c = &st.cells[n];
            let before = prev.map(|p| &p.cells[n]);
            let event = if c.event.is_some() && c.seen {
                if !c.event_known {
                    CellEvent::Unknown
                } else if let Some(b) = before {
                    if b.event.is_some() {
                        b.event
                    } else {
                        CellEvent::Unknown
                    }
                } else {
                    CellEvent::Unknown
                }
            } else {
                CellEvent::None
            };

            if c.sealed {
                if let Some(due) = c.nest_due {
                    m.cells[n] = CellModel {
                        kind: CellKind::Nest,
                        due,
                        ..CellModel::sealed()
                    };
                } else if let Some(h) = c.hidden
                    && c.scratch
                    && c.known > 0
                {
                    m.cells[n] = CellModel {
                        kind: CellKind::Scratch,
                        tier: h.tier.min(SUPREME),
                        event,
                        ..CellModel::sealed()
                    };
                } else if let Some(h) = c.hidden
                    && c.known == 1
                {
                    m.cells[n] = CellModel {
                        kind: CellKind::Revealed,
                        tier: h.tier.min(SUPREME),
                        event,
                        ..CellModel::sealed()
                    };
                } else if let Some(h) = c.hidden
                    && c.known >= 2
                {
                    m.cells[n] = CellModel {
                        kind: CellKind::Scouted,
                        beast: h.beast,
                        tier: h.tier.min(SUPREME),
                        event,
                        ..CellModel::sealed()
                    };
                } else {
                    m.cells[n] = CellModel {
                        kind: CellKind::Sealed,
                        event,
                        at_least: c.at_least,
                        ..CellModel::sealed()
                    };
                }
            } else if let Some(inm) = c.inmate {
                let link = st.link_of(inm.id);
                let prey = link.and_then(|l| {
                    if l.lion == inm.id {
                        st.cell_of(l.prey).map(|pc| PreyMark {
                            cell: pc,
                            rounds: l.rounds,
                        })
                    } else {
                        None
                    }
                });
                m.cells[n] = CellModel {
                    kind: CellKind::Inmate,
                    beast: inm.beast,
                    tier: inm.tier.min(SUPREME),
                    ivory: c.ivory,
                    prey,
                    event,
                    due: 0,
                    at_least: 0,
                };
            } else {
                m.cells[n] = CellModel {
                    kind: CellKind::Empty,
                    ..CellModel::sealed()
                };
            }
        }

        if st.pending_len > 0 {
            m.step = BoardStep::Choice;
            let p0 = &st.pending[0];
            let mut opts = [ToolId::Revenge; 4];
            let mut olen = 0usize;
            for &c in p0.slice() {
                if let ChoiceId::Tool(t) = c
                    && olen < 4
                {
                    opts[olen] = t;
                    olen += 1;
                }
            }
            m.choice = ChoiceModel::from_tools(p0.kind, &opts[..olen], p0.rerolls);
        } else if let Some(ch) = st.challenge.as_ref().filter(|c| c.window_open) {
            m.step = BoardStep::Window;
            m.window_early = ch.window_early;
        } else if let (Phase::Event, Some((kind, _))) = (st.phase, st.active_event) {
            m.step = BoardStep::Event;
            m.event_kind = kind;
        } else if st.chaining_eater.is_some() {
            m.step = BoardStep::Chain;
        } else if st.phase == Phase::Patrol {
            m.step = if st.ratway {
                BoardStep::Ratway
            } else {
                BoardStep::Patrol
            };
        } else {
            m.step = BoardStep::Manage;
        }

        m.dest_mask = 0;
        if st.phase == Phase::Patrol && !st.ratway {
            for &d in st.destinations_slice() {
                m.set_dest(d, true);
            }
        }
        m.devour_used = st.used.devour > 0;
        m.swap_used = st.used.swap > 0;
        m.hunt_used = st.used.hunt;
        m.consume_offered = st.used.devour_after_swap;
        m.no_swap = st.used.no_swap;
        m.no_devour = st.used.no_devour;

        if let Some(ch) = st.challenge.as_ref() {
            m.ch.vouchers = ch.vouchers;
            m.ch.contracts = [ch.contracts[0].id, ch.contracts[1].id, ch.contracts[2].id];
            if let Some(b) = ch.contracts.iter().find_map(|c| c.bounty) {
                m.ch.bounty = b;
            }
            for slot in 0..2 {
                if (slot as u8) < ch.deliveries_len {
                    let d = ch.deliveries[slot];
                    m.ch.status[slot] = if d.success {
                        ContractStatus::Delivered
                    } else {
                        ContractStatus::Skipped
                    };
                    m.ch.points[slot] = if d.success { d.points } else { 0 };
                } else {
                    m.ch.status[slot] = ContractStatus::Pending;
                    m.ch.points[slot] = 0;
                }
            }
        }
        m.stage = if st.phase == Phase::Ended {
            BoardRunStage::Ended
        } else {
            BoardRunStage::InProgress
        };
        m
    }
}
