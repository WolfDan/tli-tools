use super::rng::{next_rand, pick_one, refit_seed, shuffle_slice};
use super::state::SimState;
use super::types::{
    BEASTS, Beast, CELL_COUNT, CHALLENGE_DAYS, ChoiceId, Contract, ContractId, DeliveryRecord,
    DevourOption, EscortKind, GiftId, Inmate, ObjectiveId, PRESSURE, PendingChoice, PendingKind,
    Phase, PreyLink, RAINBOW, SUPREME, ToolId, ToolMask,
};

impl SimState {
    pub fn gain_for(&self, from: u8, option: DevourOption) -> u8 {
        let eater = self.cells[from as usize].inmate.unwrap();
        let target = &self.cells[option.cell as usize];
        let meal = if target.sealed {
            target.hidden
        } else {
            target.inmate
        };
        let mut gain = if option.scratch {
            2 + (self.params.scratch_stacks_base as u8)
        } else {
            1
        };
        if !option.scratch
            && eater.beast == Beast::Mammoth
            && self.has_tool(ToolId::Maul)
            && let Some(m) = meal
        {
            gain = (m.tier as i32 + 2 - eater.tier as i32).max(1) as u8;
        }
        let open_cnt = self
            .cells
            .iter()
            .filter(|c| !c.sealed && c.inmate.is_some())
            .count();
        let crowd_after = open_cnt.saturating_sub(if option.scratch { 0 } else { 1 });
        if self.has_tool(ToolId::Muzzle) && crowd_after <= 4 {
            gain += 2;
        }
        if self.has_tool(ToolId::Skewer) && eater.beast == Beast::Rat {
            gain += 1;
        }
        if self.has_tool(ToolId::Crown) && eater.beast == Beast::Lion {
            gain += 2;
        }
        gain
    }

    pub fn do_devour(&mut self, from: u8, option: DevourOption, chained: bool) {
        let eater = self.cells[from as usize].inmate.unwrap();
        let mut gain = self.gain_for(from, option);
        let meal_cell_idx = option.cell as usize;
        self.count_objective(ObjectiveId::Consume);
        self.empty_cell(meal_cell_idx);

        if self.has_tool(ToolId::Saw) && next_rand(&mut self.rng) < 0.25 {
            gain += 1;
        }
        if self.has_tool(ToolId::Whip) && next_rand(&mut self.rng) < 0.35 {
            gain += 1;
        }
        if chained && self.has_tool(ToolId::Collar) && next_rand(&mut self.rng) < 0.8 {
            gain += 1;
        }
        let revenge = self.has_tool(ToolId::Revenge) && next_rand(&mut self.rng) < 0.3;

        self.promote(eater.id, gain, !self.cage_fight());
        self.stats.devours += 1;
        self.check_prey();

        if revenge && let Some(ecn) = self.board_cell_of(eater.id) {
            let cur_eater = self.cells[ecn as usize].inmate.unwrap();
            let (sealed, slen) = self.sealed_cells();
            if slen > 0 {
                let target = pick_one(&mut self.rng, &sealed[..slen]);
                let (foods, flen) = self.eats_of(cur_eater.beast, Some(cur_eater.tier));
                let food = pick_one(&mut self.rng, &foods[..flen]);
                self.open_cell(target, false, Some((food, cur_eater.tier)));
            }
        }

        if self.has_tool(ToolId::Cage)
            && self.cells[meal_cell_idx].sealed
            && next_rand(&mut self.rng) < 0.4
        {
            self.open_cell(meal_cell_idx, false, None);
        }
    }

    pub fn go_on_chain(&mut self) {
        let stoppable = self.can_stop_chain();
        while let Some(eater_id) = self.chaining_eater {
            let Some(cn) = self.board_cell_of(eater_id) else {
                break;
            };
            let (opts, olen) = self.devour_options(cn);
            if olen == 0 {
                break;
            }
            if olen > 1 || stoppable {
                return;
            }
            self.do_devour(cn, opts[0], true);
            self.stats.chains += 1;
        }
        self.chaining_eater = None;
    }

    pub fn devour(&mut self, from: u8, to: u8) -> bool {
        if !self.guard(false) || !self.can_devour_now() || from != self.position {
            return false;
        }
        let (opts, olen) = self.devour_options(from);
        let Some(&opt) = opts[..olen].iter().find(|o| o.cell == to) else {
            return false;
        };
        let eater_id = self.cells[from as usize].inmate.unwrap().id;
        self.used.devour += 1;
        self.chaining_eater = Some(eater_id);
        self.do_devour(from, opt, false);
        self.go_on_chain();
        true
    }

    pub fn chain_bite(&mut self, to: u8) -> bool {
        if self.pending_len > 0 {
            return false;
        }
        let Some(eater_id) = self.chaining_eater else {
            return false;
        };
        let Some(cn) = self.board_cell_of(eater_id) else {
            return false;
        };
        let (opts, olen) = self.devour_options(cn);
        let Some(&opt) = opts[..olen].iter().find(|o| o.cell == to) else {
            return false;
        };
        self.do_devour(cn, opt, true);
        self.stats.chains += 1;
        self.go_on_chain();
        true
    }

    pub fn stop_chain(&mut self) -> bool {
        if self.chaining_eater.is_none() || !self.can_stop_chain() || self.pending_len > 0 {
            return false;
        }
        self.chaining_eater = None;
        true
    }

    pub fn hunt(&mut self, to: u8) -> bool {
        if !self.guard(false) || !self.can_hunt_now() {
            return false;
        }
        let (targets, tlen) = self.hunt_targets();
        if !targets[..tlen].contains(&to) {
            return false;
        }
        let Some(lion) = self.cells[self.position as usize].inmate else {
            return false;
        };
        let Some(meal) = self.cells[to as usize].inmate else {
            return false;
        };
        if (self.prey_len as usize) < self.prey.len() {
            self.prey[self.prey_len as usize] = PreyLink {
                lion: lion.id,
                prey: meal.id,
                prey_beast: meal.beast,
                rounds: self.params.prey_rounds,
                next_day: self.day + 1,
            };
            self.prey_len += 1;
        }
        self.used.hunt = true;
        true
    }

    pub fn scout(&mut self) {
        let mut doors = [0usize; 16];
        let mut dlen = 0;
        for idx in 0..CELL_COUNT {
            if self.cells[idx].sealed && self.cells[idx].nest_due.is_none() {
                self.hidden_of(idx);
                doors[dlen] = idx;
                dlen += 1;
            }
        }
        if dlen == 0 {
            return;
        }
        let mut top = 0u8;
        for &idx in &doors[..dlen] {
            let t = self.cells[idx].hidden.unwrap().tier;
            if t > top {
                top = t;
            }
        }
        let mut tops = [0usize; 16];
        let mut tlen = 0;
        for &idx in &doors[..dlen] {
            if self.cells[idx].hidden.unwrap().tier == top {
                tops[tlen] = idx;
                tlen += 1;
            }
        }
        let target = pick_one(&mut self.rng, &tops[..tlen]);
        self.cells[target].known = 2;
    }

    pub fn swap(&mut self, n: u8) -> bool {
        if !self.guard(false) || n != self.position || !self.can_swap_now() {
            return false;
        }
        let cell_idx = n as usize;
        if self.cells[cell_idx].sealed || self.cells[cell_idx].inmate.is_none() {
            return false;
        }
        let mut weights = self.params.swap_weights;
        if self.has_tool(ToolId::Rope) {
            weights[0] = 0.0;
            weights[1] = 0.0;
        }
        if !weights.iter().any(|&w| w > 0.0) {
            return false;
        }
        let old = self.cells[cell_idx].inmate.take().unwrap();
        self.check_prey();

        let mut above = false;
        if self.has_tool(ToolId::Scapegoat) {
            let mut others = [0usize; 16];
            let mut olen = 0;
            for c in &self.cells {
                if !c.sealed && c.inmate.is_some() {
                    others[olen] = c.n as usize;
                    olen += 1;
                }
            }
            if olen >= 2 {
                shuffle_slice(&mut self.rng, &mut others[..olen]);
                for &v in &others[..2] {
                    self.empty_cell(v);
                }
                self.check_prey();
                above = true;
            }
        }

        let mut higher = [0.0; 5];
        let mut has_higher = false;
        for (t, &w) in weights.iter().enumerate() {
            if (t as u8) > old.tier {
                higher[t] = w;
                if w > 0.0 {
                    has_higher = true;
                }
            }
        }
        let mut fresh = if above && has_higher {
            self.roll_inmate(&higher)
        } else {
            self.roll_inmate(&weights)
        };
        if above && fresh.tier <= old.tier {
            fresh.tier = (old.tier + 1).min(RAINBOW);
        }
        let staked = self.has_tool(ToolId::Stake)
            && old.tier < 3
            && next_rand(&mut self.rng) < 0.4
            && fresh.tier != 3;
        if staked {
            fresh.tier = 3;
        }
        if self.has_tool(ToolId::Pestle) {
            fresh.beast = old.beast;
        }

        self.cells[cell_idx].inmate = Some(fresh);
        self.used.swap += 1;
        self.stats.swaps += 1;

        if self.has_tool(ToolId::Maiden) {
            let mut above_cnt = 0;
            let mut below_cnt = 0;
            for c in &self.cells {
                if !c.sealed
                    && let Some(i) = c.inmate
                    && i.id != fresh.id
                {
                    if i.tier > fresh.tier {
                        above_cnt += 1;
                    } else if i.tier < fresh.tier {
                        below_cnt += 1;
                    }
                }
            }
            if above_cnt == below_cnt {
                self.promote(fresh.id, 3, false);
            }
        }

        self.after_growth(fresh.id);

        if self.has_tool(ToolId::Axe) && fresh.tier > old.tier {
            let (mut doors, dlen) = self.sealed_cells();
            shuffle_slice(&mut self.rng, &mut doors[..dlen]);
            for &cn in &doors[..dlen.min(2)] {
                self.open_cell(cn, false, None);
            }
        }

        if self.has_tool(ToolId::Scales) && next_rand(&mut self.rng) < 0.8 {
            self.scout();
        }

        if self.cells[cell_idx].ivory && self.board_cell_of(fresh.id).is_some() {
            let mut pool = [0u16; 16];
            let mut plen = 0;
            for c in &self.cells {
                if !c.sealed
                    && let Some(i) = c.inmate
                    && (self.params.ivory_includes_swapped || i.id != fresh.id)
                {
                    pool[plen] = i.id;
                    plen += 1;
                }
            }
            shuffle_slice(&mut self.rng, &mut pool[..plen]);
            let take = plen.min(self.params.ivory_count as usize);
            for &id in &pool[..take] {
                if self.board_cell_of(id).is_some() {
                    self.promote(id, 1, false);
                }
            }
        }
        true
    }

    pub fn apply_gift(&mut self, gift: GiftId) {
        match gift {
            GiftId::Ring => {
                let (mut sealed, slen) = self.sealed_cells();
                shuffle_slice(&mut self.rng, &mut sealed[..slen]);
                for &cn in &sealed[..slen.min(3)] {
                    self.open_cell(cn, false, None);
                }
            }
            GiftId::Horn => {
                let mut ids = [0u16; 16];
                let mut ilen = 0;
                for c in &self.cells {
                    if !c.sealed
                        && let Some(i) = c.inmate
                    {
                        let (_, olen) = self.devour_options(c.n);
                        if olen > 0 {
                            ids[ilen] = i.id;
                            ilen += 1;
                        }
                    }
                }
                for &inmate_id in &ids[..ilen] {
                    let Some(cn) = self.board_cell_of(inmate_id) else {
                        continue;
                    };
                    let (opts, olen) = self.devour_options(cn);
                    let mut valid = [DevourOption {
                        cell: 0,
                        scratch: false,
                    }; 16];
                    let mut vlen = 0;
                    for &o in &opts[..olen] {
                        if !o.scratch {
                            valid[vlen] = o;
                            vlen += 1;
                        }
                    }
                    if vlen == 0 {
                        continue;
                    }
                    let chosen = pick_one(&mut self.rng, &valid[..vlen]);
                    self.do_devour(cn, chosen, false);
                }
            }
            GiftId::Wine => {
                let mut low = [0u16; 16];
                let mut llen = 0;
                for c in &self.cells {
                    if !c.sealed
                        && let Some(i) = c.inmate
                        && i.tier < 3
                    {
                        low[llen] = i.id;
                        llen += 1;
                    }
                }
                for &id in &low[..llen] {
                    if self.board_cell_of(id).is_some() {
                        self.promote(id, 1, false);
                    }
                }
            }
            GiftId::Brand => {
                let mut hungry = [0u16; 16];
                let mut hlen = 0;
                for c in &self.cells {
                    if !c.sealed
                        && let Some(i) = c.inmate
                    {
                        let (_, olen) = self.devour_options(c.n);
                        if olen == 0 {
                            hungry[hlen] = i.id;
                            hlen += 1;
                        }
                    }
                }
                shuffle_slice(&mut self.rng, &mut hungry[..hlen]);
                for &id in &hungry[..hlen.min(4)] {
                    let Some(cn) = self.board_cell_of(id) else {
                        continue;
                    };
                    let (_, olen) = self.devour_options(cn);
                    if olen > 0 {
                        continue;
                    }
                    let inmate = self.cells[cn as usize].inmate.unwrap();
                    let mut possible = [Beast::Mammoth; 4];
                    let mut plen = 0;
                    for b in BEASTS {
                        let cand = Inmate {
                            id: inmate.id,
                            beast: b,
                            tier: inmate.tier,
                        };
                        let has_meal = self
                            .cells
                            .iter()
                            .any(|c| !c.sealed && c.inmate.is_some_and(|o| self.can_eat(cand, o)));
                        if has_meal {
                            possible[plen] = b;
                            plen += 1;
                        }
                    }
                    if plen > 0 {
                        let new_b = pick_one(&mut self.rng, &possible[..plen]);
                        self.cells[cn as usize].inmate.as_mut().unwrap().beast = new_b;
                    }
                }
                self.check_prey();
            }
        }
    }

    pub fn choose(&mut self, id: ChoiceId) -> bool {
        if self.pending_len == 0 {
            return false;
        }
        if !self.pending[0].slice().contains(&id) {
            return false;
        }
        let choice = self.pop_pending_front().unwrap();
        match (choice.kind, id) {
            (PendingKind::Gift, ChoiceId::Gift(g)) => {
                self.apply_gift(g);
            }
            (_, ChoiceId::Tool(t)) => {
                self.add_tool(t);
                if self.phase == Phase::Patrol {
                    self.fit_destinations();
                }
            }
            _ => return false,
        }
        if self.phase == Phase::Event
            && let Some((_, ecn)) = self.active_event
            && self.cells[ecn as usize].inmate.is_none()
        {
            self.active_event = None;
            self.phase = Phase::Manage;
        }
        true
    }

    pub fn reroll_offer(&mut self) -> bool {
        if !self.can_reroll() {
            return false;
        }
        let cur = self.pending[0];
        let mut shown = ToolMask(0);
        for &opt in cur.slice() {
            if let ChoiceId::Tool(t) = opt {
                shown.insert(t);
            }
        }
        let (mut pool, mut plen) = self.unheld_tools(shown);
        if plen < 3 {
            let (full_pool, full_len) = self.unheld_tools(ToolMask(0));
            pool = full_pool;
            plen = full_len;
        }
        shuffle_slice(&mut self.rng, &mut pool[..plen]);
        let take = plen.min(3);
        for i in 0..take {
            self.pending[0].options[i] = ChoiceId::Tool(pool[i]);
        }
        self.pending[0].options_len = take as u8;
        self.pending[0].rerolls -= 1;
        true
    }

    #[inline]
    pub fn current_contract(&self) -> Option<Contract> {
        let ch = self.challenge.as_ref()?;
        if (ch.deliveries_len as usize) < 3 {
            Some(ch.contracts[ch.deliveries_len as usize])
        } else {
            None
        }
    }

    #[inline]
    pub fn can_open_early(&self) -> bool {
        let Some(ch) = self.challenge.as_ref() else {
            return false;
        };
        !ch.window_open
            && self.phase != Phase::Ended
            && ch.deliveries_len == 2
            && self.day >= 9
            && self.day <= CHALLENGE_DAYS
            && (self.phase == Phase::Patrol || self.phase == Phase::Manage)
            && self.pending_len == 0
            && self.active_event.is_none()
            && self.chaining_eater.is_none()
    }

    pub fn open_early_delivery(&mut self) -> bool {
        if !self.can_open_early() {
            return false;
        }
        if let Some(ch) = self.challenge.as_mut() {
            ch.window_open = true;
            ch.window_early = true;
        }
        true
    }

    pub fn cancel_early_delivery(&mut self) -> bool {
        let Some(ch) = self.challenge.as_mut() else {
            return false;
        };
        if !ch.window_open || !ch.window_early {
            return false;
        }
        ch.window_open = false;
        ch.window_early = false;
        true
    }

    pub fn finish(&mut self) {
        self.phase = Phase::Ended;
        self.destinations = [0; 16];
        self.dest_len = 0;
        self.active_event = None;
        self.pending_len = 0;
        self.chaining_eater = None;
        if let Some(ch) = self.challenge.as_mut() {
            ch.window_open = false;
            while (ch.deliveries_len as usize) < 3 {
                let slot = ch.deliveries_len;
                let deadline = ch.contracts[slot as usize].deadline;
                ch.deliveries[slot as usize] = DeliveryRecord {
                    slot,
                    day: deadline,
                    success: false,
                    points: 0,
                    cost: 0,
                    pressure: 0,
                    escort: None,
                    early: false,
                };
                ch.deliveries_len += 1;
            }
            let completed = ch.deliveries.iter().filter(|d| d.success).count();
            let heavy = ch.deliveries.iter().any(|d| d.slot == 2 && d.success);
            ch.settled = true;
            ch.passed = completed >= 2 && heavy;
        }
    }

    pub fn advance_day(&mut self) {
        if self.day >= self.max_days {
            self.finish();
            return;
        }
        self.day += 1;
        self.begin_day();
    }

    pub fn end_day(&mut self) -> bool {
        if !self.guard(false) {
            return false;
        }
        if let Some(contract) = self.current_contract()
            && contract.deadline == self.day
        {
            if let Some(ch) = self.challenge.as_mut() {
                ch.window_open = true;
                ch.window_early = false;
            }
            return true;
        }
        self.advance_day();
        true
    }

    pub fn queue_refit(&mut self, slot: u8) {
        let mut s = refit_seed(slot);
        let (mut pool, plen) = self.unheld_tools(ToolMask(0));
        shuffle_slice(&mut s, &mut pool[..plen]);
        let take = plen.min(3);
        if take == 0 {
            return;
        }
        let mut opts = [ChoiceId::Tool(ToolId::Revenge); 4];
        for i in 0..take {
            opts[i] = ChoiceId::Tool(pool[i]);
        }
        let item = PendingChoice {
            kind: PendingKind::Refit,
            options: opts,
            options_len: take as u8,
            slot,
            refreshed: false,
            rerolls: 0,
        };
        if (self.pending_len as usize) < self.pending.len() {
            for i in (0..self.pending_len as usize).rev() {
                self.pending[i + 1] = self.pending[i];
            }
            self.pending[0] = item;
            self.pending_len += 1;
        }
    }

    pub fn after_delivery(&mut self) {
        let early = self.challenge.as_ref().is_some_and(|c| c.window_early);
        if let Some(ch) = self.challenge.as_mut() {
            ch.window_open = false;
            ch.window_early = false;
        }
        if self.day >= CHALLENGE_DAYS || early {
            self.finish();
        } else {
            self.advance_day();
        }
    }

    pub fn skip_delivery(&mut self) -> bool {
        let Some(ch) = self.challenge.as_ref() else {
            return false;
        };
        if !ch.window_open || ch.window_early {
            return false;
        }
        let Some(contract) = self.current_contract() else {
            return false;
        };
        if let Some(ch_mut) = self.challenge.as_mut() {
            let idx = ch_mut.deliveries_len as usize;
            if idx < 3 {
                ch_mut.deliveries[idx] = DeliveryRecord {
                    slot: contract.slot,
                    day: self.day,
                    success: false,
                    points: 0,
                    cost: 0,
                    pressure: 0,
                    escort: None,
                    early: false,
                };
                ch_mut.deliveries_len += 1;
            }
        }
        self.after_delivery();
        true
    }

    pub fn deliver(&mut self, cells: &[u8], vault_ids: &[u16], escort: EscortKind) -> bool {
        let mut cell_mask = 0u16;
        for &cn in cells {
            if (cn as usize) >= CELL_COUNT {
                return false;
            }
            let bit = 1u16 << cn;
            if (cell_mask & bit) != 0 {
                return false;
            }
            cell_mask |= bit;
        }
        self.deliver_mask(cell_mask, vault_ids, escort)
    }

    pub fn deliver_mask(&mut self, cell_mask: u16, vault_ids: &[u16], escort: EscortKind) -> bool {
        let Some(ch) = self.challenge.as_ref() else {
            return false;
        };
        if !ch.window_open || ch.vouchers < escort.cost() {
            return false;
        }
        let Some(contract) = self.current_contract() else {
            return false;
        };
        let mut cargo = [Inmate::default(); 24];
        let mut clen = 0;
        let mut rem = cell_mask;
        while rem != 0 {
            let cn = rem.trailing_zeros() as usize;
            rem &= rem - 1;
            let c = &self.cells[cn];
            if c.sealed || c.inmate.is_none() {
                return false;
            }
            cargo[clen] = c.inmate.unwrap();
            clen += 1;
        }
        if !vault_ids.is_empty() && !contract.heavy {
            return false;
        }
        for &vid in vault_ids {
            let Some(v) = self.vault_slice().iter().find(|v| v.id == vid) else {
                return false;
            };
            cargo[clen] = Inmate {
                id: v.id,
                beast: v.beast,
                tier: RAINBOW,
            };
            clen += 1;
        }
        if clen == 0 || !qualifies(contract, &cargo[..clen]) {
            return false;
        }
        let pressure: u16 = cargo[..clen]
            .iter()
            .map(|p| PRESSURE[p.tier as usize])
            .sum();
        if pressure > escort.cap() {
            return false;
        }
        let base_pts: u16 = cargo[..clen].iter().map(|p| 1u16 << p.tier).sum();
        let bonus_pts: u16 = if let Some(b) = contract.bounty {
            cargo[..clen]
                .iter()
                .filter(|p| p.tier >= SUPREME && p.beast == b)
                .count() as u16
                * 8
        } else {
            0
        };
        let points = base_pts + bonus_pts;

        let mut rem_empty = cell_mask;
        while rem_empty != 0 {
            let cn = rem_empty.trailing_zeros() as usize;
            rem_empty &= rem_empty - 1;
            self.empty_cell(cn);
        }
        if !vault_ids.is_empty() {
            let mut write = 0;
            for read in 0..(self.vault_len as usize) {
                if !vault_ids.contains(&self.vault[read].id) {
                    self.vault[write] = self.vault[read];
                    write += 1;
                }
            }
            self.vault_len = write as u8;
        }
        self.check_prey();

        let early = self.challenge.as_ref().unwrap().window_early;
        let deliveries_after = {
            let ch_mut = self.challenge.as_mut().unwrap();
            ch_mut.vouchers -= escort.cost();
            ch_mut.score += points;
            let idx = ch_mut.deliveries_len as usize;
            if idx < 3 {
                ch_mut.deliveries[idx] = DeliveryRecord {
                    slot: contract.slot,
                    day: self.day,
                    success: true,
                    points,
                    cost: escort.cost(),
                    pressure,
                    escort: Some(escort),
                    early,
                };
                ch_mut.deliveries_len += 1;
            }
            ch_mut.deliveries_len
        };
        self.after_delivery();
        if self.phase != Phase::Ended && deliveries_after <= 2 {
            self.queue_refit(contract.slot);
        }
        true
    }
}

pub fn qualifies(contract: Contract, cargo: &[Inmate]) -> bool {
    let mut mask = 0u8;
    let mut elite = 0usize;
    let mut low = 0usize;
    let mut top = 0usize;
    for p in cargo {
        mask |= 1 << p.beast.idx();
        if p.tier == 2 || p.tier == 3 {
            elite += 1;
        }
        if p.tier <= 1 {
            low += 1;
        }
        if p.tier >= SUPREME {
            top += 1;
        }
    }
    let beasts = mask.count_ones() as usize;
    match contract.id {
        ContractId::Mixed => cargo.len() >= 3 && beasts >= 2,
        ContractId::Cohort => cargo.len() >= 4,
        ContractId::Veteran => elite >= 1 && low >= 2,
        ContractId::Elite => elite >= 2 && beasts >= 2,
        ContractId::Kin => cargo.iter().any(|p| {
            p.tier == 3
                && cargo
                    .iter()
                    .any(|q| q.id != p.id && q.beast == p.beast && q.tier < 3)
        }),
        ContractId::Muster => elite >= 3,
        ContractId::Heavy | ContractId::Bounty => top >= 1,
    }
}
