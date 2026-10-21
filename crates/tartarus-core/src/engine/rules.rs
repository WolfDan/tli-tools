use super::rng::{next_rand, pick_index, pick_one, shuffle_slice};
use super::state::SimState;
use super::types::{
    APPETITE_ODDS, APPETITE_TRIES, BEASTS, Beast, CELL_COUNT, CROW_CELLS, EventKind, FREEDOM_CELLS,
    HUNT_BELOW, INSTIGATE_EVENTS, OFFICE, ObjectiveId, OfficeChoice, Phase, PreyLink, SUPREME,
    StartEventId, ToolId, UsedToday, steps_between,
};

impl SimState {
    pub fn begin_day(&mut self) {
        self.phase = Phase::Patrol;
        self.used = UsedToday::default();
        self.check_prey();

        let prey_snapshot_len = self.prey_len as usize;
        let mut prey_snapshot = [PreyLink::default(); 8];
        prey_snapshot[..prey_snapshot_len].copy_from_slice(&self.prey[..prey_snapshot_len]);

        for link in &prey_snapshot[..prey_snapshot_len] {
            if link.next_day > self.day {
                continue;
            }
            if let Some(pos) = (0..self.prey_len as usize)
                .find(|&idx| self.prey[idx].lion == link.lion && self.prey[idx].prey == link.prey)
            {
                if self.prey[pos].rounds <= 1 {
                    for j in (pos + 1)..(self.prey_len as usize) {
                        self.prey[j - 1] = self.prey[j];
                    }
                    self.prey_len -= 1;
                } else {
                    self.prey[pos].rounds -= 1;
                    self.prey[pos].next_day = self.day + 1;
                }
            }
            self.promote(link.lion, 1, false);
            self.promote(link.prey, 1, false);
        }
        self.check_prey();

        for n in 0..CELL_COUNT {
            if let Some(due) = self.cells[n].nest_due
                && due <= self.day
            {
                self.open_cell(n, false, Some((Beast::Rat, self.params.nest_tier)));
            }
        }

        if self.has_tool(ToolId::Tongs) {
            let mut mammoths = [0usize; 16];
            let mut mlen = 0;
            for c in &self.cells {
                if !c.sealed && !c.ivory && c.inmate.is_some_and(|i| i.beast == Beast::Mammoth) {
                    mammoths[mlen] = c.n as usize;
                    mlen += 1;
                }
            }
            if mlen > 0 {
                let chosen = pick_one(&mut self.rng, &mammoths[..mlen]);
                self.cells[chosen].ivory = true;
            }
        }

        if self.has_tool(ToolId::Claws) {
            let mut doors = [0usize; 16];
            let mut dlen = 0;
            for c in &self.cells {
                if c.sealed && !c.scratch && c.nest_due.is_none() {
                    doors[dlen] = c.n as usize;
                    dlen += 1;
                }
            }
            if dlen > 0 {
                let chosen = pick_one(&mut self.rng, &doors[..dlen]);
                self.hidden_of(chosen);
                self.cells[chosen].scratch = true;
                self.cells[chosen].known = self.cells[chosen].known.max(1);
            }
        }

        if self.has_tool(ToolId::Ironbox) {
            let mut free = [0usize; 16];
            let mut flen = 0;
            for c in &self.cells {
                if c.sealed && c.nest_due.is_none() {
                    free[flen] = c.n as usize;
                    flen += 1;
                }
            }
            if flen > 0 {
                let chosen = pick_one(&mut self.rng, &free[..flen]);
                self.cells[chosen].nest_due = Some(self.day + self.params.nest_days);
            }
        }

        if self.has_tool(ToolId::Ravenous) {
            let mut board = [0usize; 16];
            let mut blen = 0;
            let mut top = 0u8;
            for c in &self.cells {
                if !c.sealed
                    && let Some(i) = c.inmate
                {
                    board[blen] = c.n as usize;
                    blen += 1;
                    if i.tier > top {
                        top = i.tier;
                    }
                }
            }
            if blen > 0 {
                let mut tops = [0usize; 16];
                let mut tlen = 0;
                for &cn in &board[..blen] {
                    if self.cells[cn].inmate.unwrap().tier == top {
                        tops[tlen] = cn;
                        tlen += 1;
                    }
                }
                let cell_idx = pick_one(&mut self.rng, &tops[..tlen]);
                let eater_id = self.cells[cell_idx].inmate.unwrap().id;
                let n1 = (cell_idx + 15) % 16;
                let n2 = (cell_idx + 1) % 16;
                let mut sides = [0usize; 2];
                let mut slen = 0;
                for &k in &[n1, n2] {
                    if !self.cells[k].sealed && self.cells[k].inmate.is_some() {
                        sides[slen] = k;
                        slen += 1;
                    }
                }
                if slen > 0 {
                    let mut rises = 0u8;
                    for &s in &sides[..slen] {
                        self.empty_cell(s);
                        if next_rand(&mut self.rng) < 0.4 {
                            rises += 1;
                        }
                    }
                    self.check_prey();
                    if rises > 0 {
                        self.promote(eater_id, rises, false);
                    }
                }
            }
        }

        if self.has_tool(ToolId::Coin) {
            let mut counts = [[0u8; 6]; 4];
            let mut board = [0usize; 16];
            let mut blen = 0;
            for c in &self.cells {
                if !c.sealed
                    && let Some(i) = c.inmate
                {
                    if (i.tier as usize) < 6 {
                        counts[i.beast.idx()][i.tier as usize] += 1;
                    }
                    board[blen] = c.n as usize;
                    blen += 1;
                }
            }
            let mut matching = [0usize; 16];
            let mut mlen = 0;
            for &cn in &board[..blen] {
                let i = self.cells[cn].inmate.unwrap();
                if (i.tier as usize) < 6 && counts[i.beast.idx()][i.tier as usize] >= 2 {
                    matching[mlen] = cn;
                    mlen += 1;
                }
            }
            if mlen > 0 && next_rand(&mut self.rng) < 0.6 {
                let gone_cn = pick_one(&mut self.rng, &matching[..mlen]);
                let gone_inmate = self.cells[gone_cn].inmate.unwrap();
                let mut luckies = [0usize; 16];
                let mut llen = 0;
                for &cn in &matching[..mlen] {
                    if cn != gone_cn {
                        let i = self.cells[cn].inmate.unwrap();
                        if i.beast == gone_inmate.beast && i.tier == gone_inmate.tier {
                            luckies[llen] = cn;
                            llen += 1;
                        }
                    }
                }
                if llen > 0 {
                    let lucky_cn = pick_one(&mut self.rng, &luckies[..llen]);
                    let lucky_id = self.cells[lucky_cn].inmate.unwrap().id;
                    self.empty_cell(gone_cn);
                    self.check_prey();
                    self.promote(lucky_id, 2, false);
                }
            }
        }

        self.roll_destinations();
    }

    #[inline]
    pub fn guard(&self, need_patrol: bool) -> bool {
        if self.phase == Phase::Ended
            || self.pending_len > 0
            || self.chaining_eater.is_some()
            || self.phase == Phase::Office
            || self.phase == Phase::Event
            || self.challenge.as_ref().is_some_and(|c| c.window_open)
        {
            return false;
        }
        if need_patrol {
            self.phase == Phase::Patrol
        } else {
            self.phase == Phase::Manage
        }
    }

    pub fn clockwise_path(&self, to: u8) -> ([u8; 16], usize) {
        let mut path = [0u8; 16];
        let mut len = 0;
        let mut n = self.position;
        while n != to {
            n = (n + 1) % 16;
            path[len] = n;
            len += 1;
        }
        if len == 0 {
            path[0] = to;
            len = 1;
        }
        (path, len)
    }

    pub fn patrol(&mut self, to: u8) -> bool {
        if !self.guard(true) {
            return false;
        }
        if !self.destinations_slice().contains(&to) {
            return false;
        }
        let ratway = self.ratway;
        self.ratway = false;
        let steps = if ratway {
            0
        } else {
            steps_between(self.position, to)
        };
        if !ratway {
            self.stats.patrols += 1;
            let (path, plen) = self.clockwise_path(to);
            for &k in &path[..plen] {
                if k != to {
                    self.cells[k as usize].seen = true;
                }
            }
        }
        let mut event = None;
        if self.chain_armed && !ratway {
            let (path, plen) = self.clockwise_path(to);
            for &k in &path[..plen] {
                self.open_cell(k as usize, false, None);
            }
            self.chain_armed = false;
        } else if self.cells[to as usize].sealed {
            event = self.open_cell(to as usize, true, None);
        }
        if event.is_none()
            && !self.cells[to as usize].sealed
            && self.cells[to as usize].inmate.is_some()
            && self.cells[to as usize].event.is_some()
        {
            event = self.cells[to as usize].event.take();
        }
        self.position = to;
        self.check_prey();

        if !ratway {
            if self.has_tool(ToolId::Shortchain) && steps <= 3 && next_rand(&mut self.rng) < 0.5 {
                let (sealed, slen) = self.sealed_cells();
                if slen > 0 {
                    let target = pick_one(&mut self.rng, &sealed[..slen]);
                    self.open_cell(target, false, None);
                }
            }
            if self.has_tool(ToolId::Shackles) && steps >= 4 && next_rand(&mut self.rng) < 0.4 {
                let (pool, plen) = self.promotable_inmates();
                if plen > 0 {
                    let who = pick_one(&mut self.rng, &pool[..plen]);
                    self.promote(who, 1, false);
                }
            }
            if self.objective_id == ObjectiveId::Walk && !self.milestone {
                let need = ObjectiveId::Walk.need();
                self.objective_progress = (self.objective_progress + steps).min(need);
                if self.objective_progress >= need {
                    self.meet_objective();
                }
            }
        }

        if let Some(ev) = event
            && self.cells[to as usize].inmate.is_some()
        {
            self.phase = Phase::Event;
            self.active_event = Some((ev, to));
            self.count_objective(ObjectiveId::Incident);
            return true;
        }

        self.phase = if to == OFFICE {
            Phase::Office
        } else {
            Phase::Manage
        };
        if to == OFFICE && self.start_event == StartEventId::Tyranny {
            for _ in 0..3 {
                self.instigate();
                self.bestow();
            }
        }
        true
    }

    pub fn instigate(&mut self) {
        let mut free = [0usize; 16];
        let mut flen = 0;
        for c in &self.cells {
            if c.sealed && c.event.is_none() && c.nest_due.is_none() {
                free[flen] = c.n as usize;
                flen += 1;
            }
        }
        shuffle_slice(&mut self.rng, &mut free[..flen]);
        for &cn in &free[..flen.min(INSTIGATE_EVENTS)] {
            let ev = self.roll_event();
            self.cells[cn].event = Some(ev);
            self.cells[cn].event_known = true;
            self.cells[cn].seen = true;
        }
    }

    pub fn bestow(&mut self) {
        let (pool, plen) = self.promotable_inmates();
        if plen > 0 {
            let who = pick_one(&mut self.rng, &pool[..plen]);
            self.promote(who, 1, false);
        }
    }

    pub fn office_act(&mut self, choice: OfficeChoice) -> bool {
        if self.phase != Phase::Office || self.pending_len > 0 {
            return false;
        }
        match choice {
            OfficeChoice::Instigate => self.instigate(),
            OfficeChoice::Bestow => self.bestow(),
        }
        self.phase = Phase::Manage;
        true
    }

    pub fn execution_victims(&self, n: u8) -> ([u8; 16], usize) {
        let mut out = [0u8; 16];
        let mut len = 0;
        let Some(exec) = self.cells[n as usize].inmate else {
            return (out, 0);
        };
        let (eats, elen) = self.eats_of(exec.beast, None);
        for c in &self.cells {
            if c.n == n || c.sealed {
                continue;
            }
            if let Some(i) = c.inmate {
                let is_food = (0..elen).any(|k| eats[k] == i.beast);
                let tier_ok = if self.params.execution_same_tier {
                    i.tier == exec.tier
                } else {
                    i.tier < exec.tier
                };
                if is_food && tier_ok {
                    out[len] = c.n;
                    len += 1;
                }
            }
        }
        (out, len)
    }

    pub fn resolve_event(&mut self, accept: bool) -> bool {
        if self.phase != Phase::Event || self.pending_len > 0 {
            return false;
        }
        let Some((kind, n)) = self.active_event else {
            return false;
        };
        if !accept {
            self.active_event = None;
            self.phase = Phase::Manage;
            return true;
        }
        match kind {
            EventKind::Ratway => {
                self.active_event = None;
                self.phase = Phase::Patrol;
                self.ratway = true;
                for i in 1..16u8 {
                    self.destinations[(i - 1) as usize] = i;
                }
                self.dest_len = 15;
                return true;
            }
            EventKind::Chain => {
                self.chain_armed = true;
            }
            EventKind::Trickster => {
                let Some(mut who) = self.cells[n as usize].inmate else {
                    return false;
                };
                let mut counts = [0u8; 4];
                for c in &self.cells {
                    if !c.sealed
                        && let Some(i) = c.inmate
                    {
                        counts[i.beast.idx()] += 1;
                    }
                }
                let min_cnt = *counts.iter().min().unwrap_or(&0);
                let mut scarce = [Beast::Mammoth; 4];
                let mut slen = 0;
                for b in BEASTS {
                    if counts[b.idx()] == min_cnt {
                        scarce[slen] = b;
                        slen += 1;
                    }
                }
                who.beast = pick_one(&mut self.rng, &scarce[..slen]);
                who.tier = SUPREME;
                self.cells[n as usize].inmate = Some(who);
                self.check_prey();
                self.after_growth(who.id);
            }
            EventKind::Crow => {
                let mut seen_cnt = 0;
                for i in 1..16u8 {
                    if seen_cnt >= CROW_CELLS {
                        break;
                    }
                    let idx = ((n + i) % 16) as usize;
                    if !self.cells[idx].sealed || self.cells[idx].nest_due.is_some() {
                        continue;
                    }
                    self.hidden_of(idx);
                    self.cells[idx].known = self.cells[idx].known.max(1);
                    seen_cnt += 1;
                }
            }
            EventKind::Hunt => {
                for idx in 0..CELL_COUNT {
                    if !self.cells[idx].sealed || self.cells[idx].nest_due.is_some() {
                        continue;
                    }
                    let h = self.hidden_of(idx);
                    if h.tier < HUNT_BELOW {
                        self.cells[idx].known = self.cells[idx].known.max(1);
                    } else {
                        self.cells[idx].at_least = self.cells[idx].at_least.max(HUNT_BELOW);
                    }
                }
            }
            EventKind::Usurp1 | EventKind::Usurp2 | EventKind::Usurp3 | EventKind::Usurp4 => {
                let (count_opt, by, next_ev) = match kind {
                    EventKind::Usurp1 => (Some(1usize), 1u8, Some(EventKind::Usurp2)),
                    EventKind::Usurp2 => (Some(1usize), 2u8, Some(EventKind::Usurp3)),
                    EventKind::Usurp3 => (Some(2usize), 2u8, Some(EventKind::Usurp4)),
                    EventKind::Usurp4 => (None, 2u8, None),
                    _ => unreachable!(),
                };
                let (mut pool, plen) = self.promotable_inmates();
                let take = if let Some(cnt) = count_opt {
                    shuffle_slice(&mut self.rng, &mut pool[..plen]);
                    plen.min(cnt)
                } else {
                    plen
                };
                for &who_id in &pool[..take] {
                    if self.board_cell_of(who_id).is_some() {
                        self.promote(who_id, by, false);
                    }
                }
                // Update event_pool
                let mut write = 0;
                for read in 0..(self.event_pool_len as usize) {
                    if self.event_pool[read] != kind {
                        self.event_pool[write] = self.event_pool[read];
                        write += 1;
                    }
                }
                self.event_pool_len = write as u8;
                if let Some(nxt) = next_ev
                    && !self.event_pool_slice().contains(&nxt)
                    && (self.event_pool_len as usize) < self.event_pool.len()
                {
                    self.event_pool[self.event_pool_len as usize] = nxt;
                    self.event_pool_len += 1;
                }
                if kind == EventKind::Usurp1 {
                    let mut free = [0usize; 16];
                    let mut flen = 0;
                    for c in &self.cells {
                        if c.sealed && c.event.is_none() && c.nest_due.is_none() {
                            free[flen] = c.n as usize;
                            flen += 1;
                        }
                    }
                    if flen > 0 {
                        let door = pick_one(&mut self.rng, &free[..flen]);
                        self.cells[door].event = Some(EventKind::Usurp2);
                        self.cells[door].event_known = true;
                        self.cells[door].seen = true;
                    }
                }
            }
            EventKind::Freedom => {
                let (mut doors, dlen) = self.sealed_cells();
                shuffle_slice(&mut self.rng, &mut doors[..dlen]);
                for &cn in &doors[..dlen.min(FREEDOM_CELLS)] {
                    self.open_cell(cn, false, None);
                }
            }
            EventKind::Convict => {
                let (sealed, slen) = self.sealed_cells();
                if slen > 0 {
                    let target = pick_one(&mut self.rng, &sealed[..slen]);
                    let mut w = self.params.tier_weights;
                    w[0] = 0.0;
                    w[1] = 0.0;
                    w[2] = 0.0;
                    let tier = if w.iter().any(|&x| x > 0.0) {
                        pick_index(&mut self.rng, &w) as u8
                    } else {
                        3
                    };
                    let beast = BEASTS[pick_index(&mut self.rng, &self.params.beast_weights)];
                    self.open_cell(target, false, Some((beast, tier)));
                }
            }
            EventKind::Appetite => {
                let mut low_tier = u8::MAX;
                for c in &self.cells {
                    if !c.sealed
                        && let Some(i) = c.inmate
                        && i.tier < low_tier
                    {
                        low_tier = i.tier;
                    }
                }
                let mut lowest = [0u16; 16];
                let mut llen = 0;
                for c in &self.cells {
                    if !c.sealed
                        && let Some(i) = c.inmate
                        && i.tier == low_tier
                    {
                        lowest[llen] = i.id;
                        llen += 1;
                    }
                }
                if llen > 0 {
                    let who_id = pick_one(&mut self.rng, &lowest[..llen]);
                    let mut gain = 0u8;
                    for _ in 0..APPETITE_TRIES {
                        if next_rand(&mut self.rng) < APPETITE_ODDS {
                            gain += 1;
                        }
                    }
                    if gain > 0 {
                        self.promote(who_id, gain, false);
                    }
                }
            }
            EventKind::Portion => {
                let (mut pool, plen) = self.promotable_inmates();
                shuffle_slice(&mut self.rng, &mut pool[..plen]);
                for &who_id in &pool[..plen.min(3)] {
                    if self.board_cell_of(who_id).is_some() {
                        self.promote(who_id, 1, false);
                    }
                }
            }
            EventKind::Fortune => {
                let Some(cur) = self.cells[n as usize].inmate else {
                    return false;
                };
                let mut same = 0u8;
                for c in &self.cells {
                    if c.n != n
                        && !c.sealed
                        && let Some(i) = c.inmate
                        && i.beast == cur.beast
                    {
                        same += 1;
                    }
                }
                if same > 0 {
                    self.promote(cur.id, same, false);
                }
            }
            EventKind::Archenemy => {
                let Some(cur) = self.cells[n as usize].inmate else {
                    return false;
                };
                let (sealed, slen) = self.sealed_cells();
                if slen > 0 {
                    let target = pick_one(&mut self.rng, &sealed[..slen]);
                    let (foods, flen) = self.eats_of(cur.beast, Some(cur.tier));
                    let food = pick_one(&mut self.rng, &foods[..flen]);
                    self.open_cell(target, false, Some((food, cur.tier.min(SUPREME))));
                }
            }
            EventKind::Transfer => {
                let Some(cur) = self.cells[n as usize].inmate else {
                    return false;
                };
                let (sealed, slen) = self.sealed_cells();
                if slen > 0 {
                    let target = pick_one(&mut self.rng, &sealed[..slen]);
                    self.open_cell(target, false, Some((cur.beast, cur.tier)));
                }
            }
            EventKind::Execution => {
                let Some(exec) = self.cells[n as usize].inmate else {
                    return false;
                };
                let (victims, vlen) = self.execution_victims(n);
                for &vcn in &victims[..vlen] {
                    self.empty_cell(vcn as usize);
                }
                self.stats.executions += 1;
                self.check_prey();
                if vlen > 0 {
                    self.promote(exec.id, vlen as u8, self.params.execution_rerolls);
                }
            }
        }
        self.active_event = None;
        self.phase = Phase::Manage;
        true
    }
}
