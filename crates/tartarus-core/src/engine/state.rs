use serde::{Deserialize, Serialize};

use super::rng::{hash_seed, next_rand, pick_index, pick_one, roll_event_from_pool, shuffle_slice};
use super::types::{
    ALL_TOOLS, BEASTS, Beast, CELL_COUNT, CHALLENGE_DAYS, CellState, ChallengeState, ChoiceId,
    Contract, ContractId, DeliveryRecord, DevourOption, EventKind, GameMode, GameParams, GiftId,
    INITIAL_EVENT_POOL, Inmate, OFFICE, ObjectiveId, PendingChoice, PendingKind, Phase, PreyLink,
    RAINBOW, START_VOUCHERS, SUPREME, SimStats, StartEventId, ToolId, ToolMask, UsedToday,
    VaultEntry, steps_between,
};

const _: () = {
    assert!(CELL_COUNT == 16);
    assert!(INITIAL_EVENT_POOL.len() <= 17);
};

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SimState {
    pub mode: GameMode,
    pub params: GameParams,
    pub rng: u32,
    pub day: u8,
    pub days_max: u8,
    pub phase: Phase,
    pub position: u8,
    pub next_id: u16,
    pub dest_base: u8,
    pub destinations: [u8; 16],
    pub dest_len: u8,
    pub cells: [CellState; CELL_COUNT],
    pub tools_mask: ToolMask,
    pub tools_order: [ToolId; 16],
    pub tools_len: u8,
    pub vault: [VaultEntry; 32],
    pub vault_len: u8,
    pub cache: u8,
    pub prey: [PreyLink; 8],
    pub prey_len: u8,
    pub event_pool: [EventKind; 17],
    pub event_pool_len: u8,
    pub active_event: Option<(EventKind, u8)>,
    pub chain_armed: bool,
    pub ratway: bool,
    pub chaining_eater: Option<u16>,
    pub objective_id: ObjectiveId,
    pub objective_progress: u8,
    pub start_event: StartEventId,
    pub milestone: bool,
    pub used: UsedToday,
    pub stats: SimStats,
    pub pending: [PendingChoice; 4],
    pub pending_len: u8,
    pub challenge: Option<ChallengeState>,
}

fn blank_cells() -> [CellState; CELL_COUNT] {
    let mut cells = [CellState::default(); CELL_COUNT];
    for (n, cell) in cells.iter_mut().enumerate() {
        *cell = CellState {
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
            seen: true,
        };
    }
    assert!(!cells[OFFICE as usize].sealed);
    assert!(cells[1].sealed);
    cells
}

fn initial_event_pool() -> [EventKind; 17] {
    let mut event_pool = [EventKind::Chain; 17];
    for (i, &ev) in INITIAL_EVENT_POOL.iter().enumerate() {
        event_pool[i] = ev;
    }
    assert_eq!(event_pool[0], INITIAL_EVENT_POOL[0]);
    assert!(event_pool.len() >= INITIAL_EVENT_POOL.len());
    event_pool
}

fn default_challenge_state() -> ChallengeState {
    let state = ChallengeState {
        vouchers: START_VOUCHERS,
        contracts: [
            Contract {
                slot: 0,
                id: ContractId::Mixed,
                deadline: 4,
                heavy: false,
                bounty: None,
            },
            Contract {
                slot: 1,
                id: ContractId::Elite,
                deadline: 8,
                heavy: false,
                bounty: None,
            },
            Contract {
                slot: 2,
                id: ContractId::Heavy,
                deadline: 12,
                heavy: true,
                bounty: None,
            },
        ],
        deliveries: [DeliveryRecord::default(); 3],
        deliveries_len: 0,
        score: 0,
        window_open: false,
        window_early: false,
        settled: false,
        passed: false,
    };
    assert_eq!(state.vouchers, START_VOUCHERS);
    assert!(state.contracts[2].heavy);
    state
}

impl SimState {
    pub fn new_blank(mode: GameMode, rewards: bool, days_max: u8) -> Self {
        let mut params = match mode {
            GameMode::Challenge => GameParams::challenge(),
            GameMode::Drill => GameParams {
                rewards,
                drill_days: days_max.clamp(5, 60),
                ..Default::default()
            },
        };
        let d_max = match mode {
            GameMode::Challenge => CHALLENGE_DAYS,
            GameMode::Drill => params.drill_days,
        };
        params.drill_days = d_max;
        let challenge = if mode == GameMode::Challenge {
            Some(default_challenge_state())
        } else {
            None
        };
        let st = Self {
            mode,
            params,
            rng: hash_seed("ANALYZER"),
            day: 1,
            days_max: d_max,
            phase: Phase::Manage,
            position: OFFICE,
            next_id: 1,
            dest_base: 3,
            destinations: [0; 16],
            dest_len: 0,
            cells: blank_cells(),
            tools_mask: ToolMask(0),
            tools_order: [ToolId::Revenge; 16],
            tools_len: 0,
            vault: [VaultEntry::default(); 32],
            vault_len: 0,
            cache: 0,
            prey: [PreyLink::default(); 8],
            prey_len: 0,
            event_pool: initial_event_pool(),
            event_pool_len: INITIAL_EVENT_POOL.len() as u8,
            active_event: None,
            chain_armed: false,
            ratway: false,
            chaining_eater: None,
            objective_id: ObjectiveId::Other,
            objective_progress: 0,
            start_event: StartEventId::None,
            milestone: false,
            used: UsedToday::default(),
            stats: SimStats::default(),
            pending: [PendingChoice::default(); 4],
            pending_len: 0,
            challenge,
        };
        assert_eq!(st.day, 1);
        assert!(st.days_max >= 5);
        st
    }

    pub fn new_drill_run(seed: u32, tools: &[ToolId]) -> Self {
        assert!(tools.len() <= ALL_TOOLS.len());
        let mut st = Self::new_blank(GameMode::Drill, false, 20);
        st.rng = seed;
        let mut doors = [0usize; 15];
        for (i, d) in doors.iter_mut().enumerate() {
            *d = i + 1;
        }
        shuffle_slice(&mut st.rng, &mut doors);
        for &cn in &doors[..4] {
            st.open_cell(cn, false, None);
        }
        st.stats.opened = 0;
        st.objective_progress = 0;
        st.milestone = false;
        let ev = st.roll_event();
        st.cells[doors[4]].event = Some(ev);
        st.cells[doors[4]].event_known = true;
        st.cells[doors[4]].seen = true;
        for &t in tools {
            st.add_tool(t);
        }
        st.begin_day();
        assert_eq!(st.phase, Phase::Patrol);
        st
    }

    pub fn new_random_run(seed: u32) -> Self {
        let mut st = Self::new_blank(GameMode::Drill, true, 20);
        st.rng = seed;
        let mut doors = [0usize; 15];
        for (i, d) in doors.iter_mut().enumerate() {
            *d = i + 1;
        }
        shuffle_slice(&mut st.rng, &mut doors);
        for &cn in &doors[..4] {
            st.open_cell(cn, false, None);
        }
        st.stats.opened = 0;
        let objectives = [
            ObjectiveId::Doors,
            ObjectiveId::Consume,
            ObjectiveId::Walk,
            ObjectiveId::King,
            ObjectiveId::Incident,
        ];
        st.objective_id = pick_one(&mut st.rng, &objectives);
        st.objective_progress = 0;
        st.milestone = false;
        let ev = st.roll_event();
        st.cells[doors[4]].event = Some(ev);
        st.cells[doors[4]].event_known = true;
        st.cells[doors[4]].seen = true;
        let (mut pool, plen) = st.unheld_tools(ToolMask(0));
        shuffle_slice(&mut st.rng, &mut pool[..plen]);
        let take = plen.min(3);
        if take > 0 {
            let mut opts = [ChoiceId::Tool(ToolId::Revenge); 4];
            for i in 0..take {
                opts[i] = ChoiceId::Tool(pool[i]);
            }
            st.push_pending(PendingChoice {
                kind: PendingKind::Tool,
                options: opts,
                options_len: take as u8,
                slot: 0,
                refreshed: false,
                rerolls: 0,
            });
        }
        st.begin_day();
        assert_eq!(st.day, 1);
        assert_eq!(st.phase, Phase::Patrol);
        st
    }

    #[inline]
    pub const fn has_tool(&self, tool: ToolId) -> bool {
        assert!((tool as usize) < ALL_TOOLS.len());
        assert!((self.tools_len as usize) <= self.tools_order.len());
        self.tools_mask.has(tool)
    }

    #[inline]
    pub fn add_tool(&mut self, tool: ToolId) {
        assert!((tool as usize) < ALL_TOOLS.len());
        if !self.has_tool(tool) {
            self.tools_mask.insert(tool);
            if (self.tools_len as usize) < self.tools_order.len() {
                self.tools_order[self.tools_len as usize] = tool;
                self.tools_len += 1;
            }
        }
        assert!(self.has_tool(tool));
    }

    #[inline]
    pub fn tools_slice(&self) -> &[ToolId] {
        let len = self.tools_len as usize;
        assert!(len <= self.tools_order.len());
        let s = &self.tools_order[..len];
        assert_eq!(s.len(), len);
        s
    }

    #[inline]
    pub fn destinations_slice(&self) -> &[u8] {
        let len = self.dest_len as usize;
        assert!(len <= self.destinations.len());
        let s = &self.destinations[..len];
        assert_eq!(s.len(), len);
        s
    }

    #[inline]
    pub fn event_pool_slice(&self) -> &[EventKind] {
        let len = self.event_pool_len as usize;
        assert!(len <= self.event_pool.len());
        let s = &self.event_pool[..len];
        assert_eq!(s.len(), len);
        s
    }

    #[inline]
    pub fn prey_slice(&self) -> &[PreyLink] {
        let len = self.prey_len as usize;
        assert!(len <= self.prey.len());
        let s = &self.prey[..len];
        assert_eq!(s.len(), len);
        s
    }

    #[inline]
    pub fn vault_slice(&self) -> &[VaultEntry] {
        let len = self.vault_len as usize;
        assert!(len <= self.vault.len());
        let s = &self.vault[..len];
        assert_eq!(s.len(), len);
        s
    }

    #[inline]
    pub const fn cage_fight(&self) -> bool {
        assert!((self.position as usize) < CELL_COUNT);
        assert!((self.pending_len as usize) <= self.pending.len());
        matches!(self.start_event, StartEventId::CageFight)
    }

    #[inline]
    pub const fn can_stop_chain(&self) -> bool {
        assert!((self.position as usize) < CELL_COUNT);
        assert!((self.pending_len as usize) <= self.pending.len());
        self.params.chain_stop
    }

    #[inline]
    pub const fn can_devour_now(&self) -> bool {
        assert!((self.position as usize) < CELL_COUNT);
        assert!((self.pending_len as usize) <= self.pending.len());
        self.used.devour < 1
            && !self.used.hunt
            && !self.used.no_devour
            && (self.used.swap < 1 || self.used.devour_after_swap)
    }

    #[inline]
    pub const fn can_swap_now(&self) -> bool {
        assert!((self.position as usize) < CELL_COUNT);
        assert!((self.pending_len as usize) <= self.pending.len());
        self.used.devour < 1
            && !self.used.hunt
            && !self.used.no_swap
            && self.used.swap < self.params.swaps_per_day
    }

    #[inline]
    pub const fn can_reroll(&self) -> bool {
        assert!((self.pending_len as usize) <= self.pending.len());
        assert!((self.position as usize) < CELL_COUNT);
        if self.pending_len == 0 {
            return false;
        }
        let first = &self.pending[0];
        matches!(first.kind, PendingKind::Tool) && first.rerolls > 0
    }

    pub fn push_pending(&mut self, p: PendingChoice) {
        assert!((p.options_len as usize) <= p.options.len());
        assert!((self.pending_len as usize) <= self.pending.len());
        if (self.pending_len as usize) < self.pending.len() {
            self.pending[self.pending_len as usize] = p;
            self.pending_len += 1;
        }
    }

    pub fn pop_pending_front(&mut self) -> Option<PendingChoice> {
        assert!((self.pending_len as usize) <= self.pending.len());
        if self.pending_len == 0 {
            return None;
        }
        let first = self.pending[0];
        for i in 1..(self.pending_len as usize) {
            self.pending[i - 1] = self.pending[i];
        }
        self.pending_len -= 1;
        assert!((self.pending_len as usize) < self.pending.len());
        Some(first)
    }

    #[inline]
    pub fn eats_of(&self, beast: Beast, tier: Option<u8>) -> ([Beast; 2], usize) {
        assert!(beast.idx() < BEASTS.len());
        let first = beast.primary_prey();
        let from = if self.cage_fight() {
            0
        } else {
            self.params.double_eat_tier
        };
        let (out, len) = if tier.unwrap_or(0) >= from
            && let Some(sec) = beast.second_meal()
        {
            ([first, sec], 2)
        } else {
            ([first, first], 1)
        };
        assert!(len >= 1);
        assert!(len <= 2);
        (out, len)
    }

    #[inline]
    pub fn can_eat(&self, eater: Inmate, meal: Inmate) -> bool {
        assert!(eater.beast.idx() < BEASTS.len());
        assert!(meal.beast.idx() < BEASTS.len());
        if eater.id == meal.id || eater.tier >= RAINBOW {
            return false;
        }
        let (eats, eats_len) = self.eats_of(eater.beast, Some(eater.tier));
        let in_eats = (0..eats_len).any(|i| eats[i] == meal.beast);
        let counters = in_eats
            || (eater.beast == Beast::Rat
                && meal.beast == Beast::Rat
                && self.has_tool(ToolId::Skewer));
        if eater.tier != meal.tier {
            if meal.tier < eater.tier && counters && self.cage_fight() {
                return true;
            }
            if meal.tier > eater.tier
                && counters
                && eater.beast == Beast::Mammoth
                && self.has_tool(ToolId::Maul)
            {
                return true;
            }
            return eater.tier > meal.tier
                && eater.beast == Beast::Wolf
                && self.has_tool(ToolId::Guillotine)
                && meal.beast == Beast::Wolf.primary_prey();
        }
        counters || (eater.beast == Beast::Lion && self.has_tool(ToolId::Crown))
    }

    pub fn devour_options(&self, n: u8) -> ([DevourOption; 16], usize) {
        assert!((n as usize) < CELL_COUNT);
        let mut out = [DevourOption {
            cell: 0,
            scratch: false,
        }; 16];
        let mut len = 0;
        let cell = &self.cells[n as usize];
        if cell.sealed {
            return (out, 0);
        }
        let Some(eater) = cell.inmate else {
            return (out, 0);
        };
        for c in &self.cells {
            if c.n == n {
                continue;
            }
            if !c.sealed {
                if let Some(meal) = c.inmate
                    && self.can_eat(eater, meal)
                {
                    out[len] = DevourOption {
                        cell: c.n,
                        scratch: false,
                    };
                    len += 1;
                }
            } else if c.scratch
                && eater.beast == Beast::Wolf
                && eater.tier < RAINBOW
                && let Some(hid) = c.hidden
                && hid.tier == eater.tier
            {
                out[len] = DevourOption {
                    cell: c.n,
                    scratch: true,
                };
                len += 1;
            }
        }
        assert!(len <= CELL_COUNT);
        (out, len)
    }

    pub fn start_options(&self) -> ([DevourOption; 16], usize) {
        assert!((self.position as usize) < CELL_COUNT);
        assert!((self.pending_len as usize) <= self.pending.len());
        if self.phase != Phase::Manage
            || self.pending_len > 0
            || self.chaining_eater.is_some()
            || !self.can_devour_now()
            || self.challenge.as_ref().is_some_and(|c| c.window_open)
        {
            return (
                [DevourOption {
                    cell: 0,
                    scratch: false,
                }; 16],
                0,
            );
        }
        self.devour_options(self.position)
    }

    pub fn chain_options(&self) -> ([DevourOption; 16], usize) {
        assert!((self.position as usize) < CELL_COUNT);
        assert!((self.pending_len as usize) <= self.pending.len());
        let Some(eater_id) = self.chaining_eater else {
            return (
                [DevourOption {
                    cell: 0,
                    scratch: false,
                }; 16],
                0,
            );
        };
        let Some(cell_n) = self.board_cell_of(eater_id) else {
            return (
                [DevourOption {
                    cell: 0,
                    scratch: false,
                }; 16],
                0,
            );
        };
        self.devour_options(cell_n)
    }

    pub fn hunt_targets(&self) -> ([u8; 16], usize) {
        assert!((self.position as usize) < CELL_COUNT);
        let mut out = [0u8; 16];
        let mut len = 0;
        if !self.has_tool(ToolId::Trap) {
            return (out, 0);
        }
        let cell = &self.cells[self.position as usize];
        if cell.sealed {
            return (out, 0);
        }
        let Some(lion) = cell.inmate else {
            return (out, 0);
        };
        if lion.beast != Beast::Lion || self.link_of(lion.id).is_some() {
            return (out, 0);
        }
        for c in &self.cells {
            if c.n == self.position || c.sealed {
                continue;
            }
            if let Some(meal) = c.inmate
                && self.can_eat(lion, meal)
                && self.link_of(meal.id).is_none()
            {
                out[len] = c.n;
                len += 1;
            }
        }
        assert!(len <= CELL_COUNT);
        (out, len)
    }

    pub fn can_hunt_now(&self) -> bool {
        assert!((self.position as usize) < CELL_COUNT);
        assert!((self.pending_len as usize) <= self.pending.len());
        self.phase == Phase::Manage
            && self.pending_len == 0
            && self.chaining_eater.is_none()
            && self.used.devour < 1
            && self.used.swap < 1
            && !self.used.hunt
            && !self.challenge.as_ref().is_some_and(|c| c.window_open)
            && self.hunt_targets().1 > 0
    }

    #[inline]
    pub fn cell_of(&self, inmate_id: u16) -> Option<u8> {
        assert!((self.position as usize) < CELL_COUNT);
        for c in &self.cells {
            if let Some(i) = c.inmate
                && i.id == inmate_id
            {
                assert!((c.n as usize) < CELL_COUNT);
                return Some(c.n);
            }
        }
        None
    }

    #[inline]
    pub fn board_cell_of(&self, inmate_id: u16) -> Option<u8> {
        assert!((self.position as usize) < CELL_COUNT);
        for c in &self.cells {
            if !c.sealed
                && let Some(i) = c.inmate
                && i.id == inmate_id
            {
                assert!((c.n as usize) < CELL_COUNT);
                return Some(c.n);
            }
        }
        None
    }

    #[inline]
    pub fn link_of(&self, inmate_id: u16) -> Option<PreyLink> {
        assert!((self.prey_len as usize) <= self.prey.len());
        let out = self
            .prey_slice()
            .iter()
            .copied()
            .find(|l| l.lion == inmate_id || l.prey == inmate_id);
        assert!(out.is_none() || self.prey_len > 0);
        out
    }

    pub fn check_prey(&mut self) {
        let len_before = self.prey_len as usize;
        assert!(len_before <= self.prey.len());
        let mut write = 0;
        for read in 0..len_before {
            let l = self.prey[read];
            let lion_ok = self.board_cell_of(l.lion).is_some_and(|cn| {
                self.cells[cn as usize]
                    .inmate
                    .is_some_and(|i| i.beast == Beast::Lion)
            });
            let prey_ok = self.board_cell_of(l.prey).is_some_and(|cn| {
                self.cells[cn as usize]
                    .inmate
                    .is_some_and(|i| i.beast == l.prey_beast)
            });
            if lion_ok && prey_ok {
                self.prey[write] = l;
                write += 1;
            }
        }
        assert!(write <= len_before);
        self.prey_len = write as u8;
    }

    #[inline]
    pub fn door_weights(&self) -> [f64; 5] {
        let mut w = self.params.tier_weights;
        assert!(w.iter().all(|x| x.is_finite()));
        if self.has_tool(ToolId::Hook) && w[1..].iter().any(|&x| x > 0.0) {
            w[0] = 0.0;
        }
        assert!(w.iter().all(|x| x.is_finite()));
        w
    }

    #[inline]
    pub fn cell_weights(&self, cell: &CellState) -> [f64; 5] {
        assert!((cell.n as usize) < CELL_COUNT);
        let w = self.door_weights();
        let floor = cell.at_least as usize;
        if floor == 0 {
            return w;
        }
        let mut kept = [0.0; 5];
        let mut any = false;
        for t in floor.min(5)..5 {
            kept[t] = w[t];
            if kept[t] > 0.0 {
                any = true;
            }
        }
        let out = if any { kept } else { w };
        assert!(out.iter().all(|x| x.is_finite()));
        out
    }

    #[inline]
    pub fn make_inmate(&mut self, beast: Beast, tier: u8) -> Inmate {
        assert!(beast.idx() < BEASTS.len());
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        let inmate = Inmate {
            id,
            beast,
            tier: tier.min(RAINBOW),
        };
        assert!(inmate.tier <= RAINBOW);
        inmate
    }

    #[inline]
    pub fn roll_inmate(&mut self, weights: &[f64; 5]) -> Inmate {
        assert!(weights.iter().all(|w| w.is_finite()));
        let tier = pick_index(&mut self.rng, weights) as u8;
        let beast_idx = pick_index(&mut self.rng, &self.params.beast_weights);
        assert!(beast_idx < BEASTS.len());
        let beast = BEASTS[beast_idx];
        self.make_inmate(beast, tier)
    }

    pub fn hidden_of(&mut self, cell_idx: usize) -> Inmate {
        assert!(cell_idx < CELL_COUNT);
        if let Some(h) = self.cells[cell_idx].hidden {
            assert!(h.tier <= RAINBOW);
            return h;
        }
        let w = self.cell_weights(&self.cells[cell_idx]);
        let h = self.roll_inmate(&w);
        self.cells[cell_idx].hidden = Some(h);
        assert!(self.cells[cell_idx].hidden.is_some());
        h
    }

    pub fn door_inmate(&mut self, cell_idx: usize) -> Inmate {
        assert!(cell_idx < CELL_COUNT);
        let c = &self.cells[cell_idx];
        if let Some(h) = c.hidden
            && (h.tier > 0 || c.known > 0 || !self.has_tool(ToolId::Hook))
        {
            assert!(h.tier <= RAINBOW);
            return h;
        }
        let w = self.cell_weights(&self.cells[cell_idx]);
        let rolled = self.roll_inmate(&w);
        assert!(rolled.tier <= RAINBOW);
        rolled
    }

    pub fn roll_event(&mut self) -> EventKind {
        let len = self.event_pool_len as usize;
        assert!(len <= self.event_pool.len());
        let ev = roll_event_from_pool(&mut self.rng, &self.event_pool[..len]);
        assert!(ev.weight() > 0);
        ev
    }

    pub fn empty_cell(&mut self, cell_idx: usize) {
        assert!(cell_idx < CELL_COUNT);
        let sealed = self.params.reusable_cells;
        let ev = if sealed && next_rand(&mut self.rng) * 100.0 < self.params.event_chance {
            Some(self.roll_event())
        } else {
            None
        };
        let n = self.cells[cell_idx].n;
        self.cells[cell_idx] = CellState {
            n,
            sealed,
            inmate: None,
            hidden: None,
            known: 0,
            at_least: 0,
            scratch: false,
            nest_due: None,
            event: ev,
            event_known: true,
            ivory: false,
            seen: false,
        };
        assert!(self.cells[cell_idx].inmate.is_none());
    }

    #[inline]
    pub fn sealed_cells(&self) -> ([usize; 16], usize) {
        assert!((self.position as usize) < CELL_COUNT);
        let mut out = [0usize; 16];
        let mut len = 0;
        for c in &self.cells {
            if c.sealed {
                out[len] = c.n as usize;
                len += 1;
            }
        }
        assert!(len <= CELL_COUNT);
        (out, len)
    }

    #[inline]
    pub fn promotable_inmates(&self) -> ([u16; 16], usize) {
        assert!((self.position as usize) < CELL_COUNT);
        let mut pool = [0u16; 16];
        let mut len = 0;
        for c in &self.cells {
            if !c.sealed
                && let Some(i) = c.inmate
                && i.tier < RAINBOW
            {
                pool[len] = i.id;
                len += 1;
            }
        }
        assert!(len <= CELL_COUNT);
        (pool, len)
    }

    #[inline]
    pub fn unheld_tools(&self, exclude: ToolMask) -> ([ToolId; 31], usize) {
        assert!((self.tools_len as usize) <= self.tools_order.len());
        let mut pool = [ToolId::Revenge; 31];
        let mut len = 0;
        let blocked = self.tools_mask.0 | exclude.0;
        for t in ALL_TOOLS {
            if (blocked & (1u32 << (t as u32))) == 0 {
                pool[len] = t;
                len += 1;
            }
        }
        assert!(len <= ALL_TOOLS.len());
        (pool, len)
    }

    pub fn meet_objective(&mut self) {
        assert!((self.pending_len as usize) <= self.pending.len());
        self.milestone = true;
        if self.params.rewards {
            let (mut pool, plen) = self.unheld_tools(ToolMask(0));
            shuffle_slice(&mut self.rng, &mut pool[..plen]);
            let take = plen.min(3);
            if take > 0 {
                let mut opts = [ChoiceId::Tool(ToolId::Revenge); 4];
                for i in 0..take {
                    opts[i] = ChoiceId::Tool(pool[i]);
                }
                self.push_pending(PendingChoice {
                    kind: PendingKind::Tool,
                    options: opts,
                    options_len: take as u8,
                    slot: 0,
                    refreshed: false,
                    rerolls: 0,
                });
            }
        }
        assert!(self.milestone);
    }

    pub fn count_objective(&mut self, id: ObjectiveId) {
        assert!((id as u8) <= (ObjectiveId::Other as u8));
        if self.objective_id != id || self.milestone {
            return;
        }
        let need = id.need();
        if need == 0 {
            return;
        }
        self.objective_progress = (self.objective_progress + 1).min(need);
        assert!(self.objective_progress <= need);
        if self.objective_progress >= need {
            self.meet_objective();
        }
    }

    pub fn after_growth(&mut self, inmate_id: u16) {
        assert!((self.vault_len as usize) <= self.vault.len());
        let Some(cn) = self.cell_of(inmate_id) else {
            return;
        };
        let Some(inmate) = self.cells[cn as usize].inmate else {
            return;
        };
        assert!(inmate.tier <= RAINBOW);
        if inmate.tier > self.stats.best {
            self.stats.best = inmate.tier;
        }
        if self.objective_id == ObjectiveId::King && !self.milestone && inmate.tier >= SUPREME {
            self.meet_objective();
        }
        if inmate.tier < RAINBOW {
            return;
        }
        let Some(cn2) = self.cell_of(inmate_id) else {
            return;
        };
        self.empty_cell(cn2 as usize);
        if (self.vault_len as usize) < self.vault.len() {
            self.vault[self.vault_len as usize] = VaultEntry {
                id: inmate.id,
                beast: inmate.beast,
                day: self.day,
            };
            self.vault_len += 1;
        }
        self.cache = self.cache.saturating_add(1);
        if self.params.rewards {
            let gift_picks = if self.start_event == StartEventId::Opulence {
                3
            } else {
                1
            };
            for _ in 0..gift_picks {
                self.push_pending(PendingChoice {
                    kind: PendingKind::Gift,
                    options: [
                        ChoiceId::Gift(GiftId::Ring),
                        ChoiceId::Gift(GiftId::Horn),
                        ChoiceId::Gift(GiftId::Wine),
                        ChoiceId::Gift(GiftId::Brand),
                    ],
                    options_len: 4,
                    slot: 0,
                    refreshed: false,
                    rerolls: 0,
                });
            }
        }
    }

    pub fn promote(&mut self, inmate_id: u16, by: u8, reroll_beast: bool) {
        assert!((self.position as usize) < CELL_COUNT);
        let Some(cn) = self.cell_of(inmate_id) else {
            return;
        };
        let Some(mut inmate) = self.cells[cn as usize].inmate else {
            return;
        };
        inmate.tier = (inmate.tier + by).min(RAINBOW);
        assert!(inmate.tier <= RAINBOW);
        if reroll_beast && inmate.tier < RAINBOW {
            let b_idx = pick_index(&mut self.rng, &self.params.beast_weights);
            inmate.beast = BEASTS[b_idx];
        }
        self.cells[cn as usize].inmate = Some(inmate);
        self.after_growth(inmate_id);
    }

    fn trigger_patrol_open_tools(&mut self, cell_idx: usize, inmate_id: u16) {
        assert!(cell_idx < CELL_COUNT);
        if self.has_tool(ToolId::Dogtag)
            && self.cells[cell_idx]
                .inmate
                .is_some_and(|i| i.id == inmate_id && i.tier < RAINBOW)
            && next_rand(&mut self.rng) < 0.25
        {
            self.promote(inmate_id, 1, false);
        }
        if self.has_tool(ToolId::Blacklist) {
            let sealed_cnt = self.cells.iter().filter(|c| c.sealed).count();
            if sealed_cnt <= 5 {
                let (mut pool, plen) = self.promotable_inmates();
                shuffle_slice(&mut self.rng, &mut pool[..plen]);
                for &who_id in &pool[..plen.min(2)] {
                    if self.board_cell_of(who_id).is_some() {
                        self.promote(who_id, 1, false);
                    }
                }
            }
        }
        if self.has_tool(ToolId::Fire)
            && next_rand(&mut self.rng) < 0.35
            && self.cells[cell_idx]
                .inmate
                .is_some_and(|i| i.id == inmate_id)
        {
            let cur_inmate = self.cells[cell_idx].inmate.unwrap();
            let (sealed, slen) = self.sealed_cells();
            if slen > 0 {
                let target = pick_one(&mut self.rng, &sealed[..slen]);
                self.open_cell(target, false, Some((cur_inmate.beast, cur_inmate.tier)));
            }
        }
        if self.has_tool(ToolId::Keys) && next_rand(&mut self.rng) < 0.8 {
            let left = (cell_idx + 15) % 16;
            let right = (cell_idx + 1) % 16;
            self.open_cell(left, false, None);
            self.open_cell(right, false, None);
        }
        if self.has_tool(ToolId::Forged) && next_rand(&mut self.rng) < 0.5 {
            let (sealed, slen) = self.sealed_cells();
            if slen > 0 {
                let extra = pick_one(&mut self.rng, &sealed[..slen]);
                self.open_cell(extra, false, None);
            }
        }
        assert!((self.position as usize) < CELL_COUNT);
    }

    pub fn open_cell(
        &mut self,
        cell_idx: usize,
        is_patrol: bool,
        forced: Option<(Beast, u8)>,
    ) -> Option<EventKind> {
        assert!(cell_idx < CELL_COUNT);
        if !self.cells[cell_idx].sealed {
            return None;
        }
        let event = self.cells[cell_idx].event;
        let inmate = if let Some((b, t)) = forced {
            self.make_inmate(b, t)
        } else {
            self.door_inmate(cell_idx)
        };
        assert!(inmate.tier <= RAINBOW);
        self.cells[cell_idx].sealed = false;
        self.cells[cell_idx].inmate = Some(inmate);
        self.cells[cell_idx].hidden = None;
        self.cells[cell_idx].known = 0;
        self.cells[cell_idx].at_least = 0;
        self.cells[cell_idx].scratch = false;
        self.cells[cell_idx].nest_due = None;
        self.cells[cell_idx].event = None;
        self.stats.opened += 1;
        self.count_objective(ObjectiveId::Doors);

        self.after_growth(inmate.id);

        if is_patrol {
            self.trigger_patrol_open_tools(cell_idx, inmate.id);
        }
        if is_patrol { event } else { None }
    }

    #[inline]
    pub fn destination_limit(&self) -> usize {
        assert!((self.tools_len as usize) <= self.tools_order.len());
        let base = if self.dest_base > 0 {
            self.dest_base as i32
        } else {
            self.params.destinations as i32
        };
        let penalty = (self.has_tool(ToolId::Forged) as i32)
            + (self.has_tool(ToolId::Whip) as i32)
            + (self.has_tool(ToolId::Stake) as i32);
        let lim = (base - penalty).max(1) as usize;
        assert!(lim >= 1);
        lim
    }

    pub fn fit_destinations(&mut self) {
        assert!((self.dest_len as usize) <= self.destinations.len());
        if self.phase == Phase::Patrol {
            let lim = self.destination_limit();
            if (self.dest_len as usize) > lim {
                self.dest_len = lim as u8;
            }
        }
        assert!((self.dest_len as usize) <= self.destinations.len());
    }

    pub fn roll_destinations(&mut self) {
        assert!((self.position as usize) < CELL_COUNT);
        let fewer = next_rand(&mut self.rng) * 100.0 < self.params.fewer_doors_chance;
        self.dest_base = (self.params.destinations as i32 - (fewer as i32)).max(1) as u8;
        let range = self.params.patrol_range.min(15) as usize;
        let mut ahead = [0u8; 15];
        for i in 1..=range {
            ahead[i - 1] = (self.position + (i as u8)) % 16;
        }
        shuffle_slice(&mut self.rng, &mut ahead[..range]);
        let lim = self.destination_limit().min(range);
        let mut picks = [0u8; 16];
        let mut plen = lim;
        picks[..lim].copy_from_slice(&ahead[..lim]);

        if self.params.event_two_doors {
            let has_ev = (0..plen).any(|i| {
                let c = &self.cells[picks[i] as usize];
                c.sealed && c.event.is_some() && c.seen
            });
            if has_ev {
                let mut reordered = [0u8; 16];
                let mut rlen = 0;
                for i in 0..plen {
                    let c = &self.cells[picks[i] as usize];
                    if c.sealed && c.event.is_some() && c.seen {
                        reordered[rlen] = picks[i];
                        rlen += 1;
                    }
                }
                for i in 0..plen {
                    let c = &self.cells[picks[i] as usize];
                    if !(c.sealed && c.event.is_some() && c.seen) {
                        reordered[rlen] = picks[i];
                        rlen += 1;
                    }
                }
                plen = rlen.min(2);
                picks[..plen].copy_from_slice(&reordered[..plen]);
            }
        }

        let pos = self.position;
        picks[..plen].sort_by_key(|&n| steps_between(pos, n));
        self.destinations[..plen].copy_from_slice(&picks[..plen]);
        self.dest_len = plen as u8;
        assert!((self.dest_len as usize) <= self.destinations.len());
    }
}
