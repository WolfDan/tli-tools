use std::env;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::time::Instant;
use tartarus_core::TreasureOrigin;

const READ_CHUNK_BYTES: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogEvent {
    ObjectivePopup,
    InstrumentsPopup { fresh: bool },
    BoardTrigger { settle_ms: u64, treasure_view: bool },
    SkipPending,
    TreasurePopup { after: Option<TreasureOrigin> },
    SpeedFlip,
    Riot(bool),
    RunOver,
    RunNew,
    RunBegin { run_id: u64, is_settled: bool },
    DayCheck { cur: u8, max: u8 },
    RelicPopState(bool),
}

fn fnv1a_64(s: &str) -> u64 {
    use std::hash::Hasher;
    let mut hasher = fnv::FnvHasher::default();
    hasher.write(s.as_bytes());
    let hash = hasher.finish();
    if hash == 0 { 1 } else { hash }
}

fn parse_s16_bgm_begin(line: &str) -> Option<(u64, bool)> {
    let rest = line.split("[S16BGM] board Begin").nth(1)?;
    let mut run_id: Option<u64> = None;
    let mut is_settled = false;
    for tok in rest.split_whitespace() {
        if let Some(id_str) = tok.strip_prefix("runId=") {
            if !id_str.is_empty() {
                run_id = Some(fnv1a_64(id_str));
            }
        } else if let Some(settled_str) = tok.strip_prefix("IsSettled=") {
            is_settled = settled_str.eq_ignore_ascii_case("true") || settled_str == "1";
        }
    }
    run_id.map(|id| (id, is_settled))
}

fn parse_s16_last_day(line: &str) -> Option<(u8, u8)> {
    let rest = line.split("[S16LastDay] check").nth(1)?;
    let mut cur: Option<u8> = None;
    let mut max: Option<u8> = None;
    for tok in rest.split_whitespace() {
        if let Some(c_str) = tok.strip_prefix("cur=") {
            cur = c_str.parse::<u8>().ok();
        } else if let Some(m_str) = tok.strip_prefix("max=") {
            max = m_str.parse::<u8>().ok();
        }
    }
    Some((cur?, max?))
}

pub struct LogWatcher {
    pub path: Option<PathBuf>,
    file: Option<File>,
    buf: Vec<u8>,
    offset: u64,
    carry: String,
    events: Vec<LogEvent>,
    initialized: bool,
    last_action_at: Option<Instant>,
    last_eat_at: Option<Instant>,
    last_gift_confirm_at: Option<Instant>,
    saw_cur_zero: bool,
}

impl Default for LogWatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl LogWatcher {
    pub fn new() -> Self {
        let path = Self::discover_log_path();
        let file = path.as_ref().and_then(|p| File::open(p).ok());
        Self {
            path,
            file,
            buf: vec![0u8; READ_CHUNK_BYTES],
            offset: 0,
            carry: String::with_capacity(4096),
            events: Vec::with_capacity(16),
            initialized: false,
            last_action_at: None,
            last_eat_at: None,
            last_gift_confirm_at: None,
            saw_cur_zero: false,
        }
    }

    pub fn discover_log_path() -> Option<PathBuf> {
        const TLI_STEAM_APP_ID: u32 = 1_974_050;

        if let Ok(custom) = env::var("TAC_OVERLAY_LOG") {
            let p = PathBuf::from(custom);
            if p.exists() {
                return Some(p);
            }
        }

        let mut base_dirs: Vec<PathBuf> = Vec::new();
        let steam_dir = env::var("STEAM_DIR")
            .ok()
            .and_then(|d| steamlocate::SteamDir::from_dir(&PathBuf::from(d)).ok())
            .or_else(|| steamlocate::SteamDir::locate().ok());
        if let Some(steam) = steam_dir
            && let Ok(Some((app, lib))) = steam.find_app(TLI_STEAM_APP_ID)
        {
            base_dirs.push(lib.resolve_app_dir(&app));
        }

        #[cfg(target_os = "windows")]
        for base in [
            "C:\\Program Files (x86)\\Torchlight Infinite\\Game",
            "C:\\Program Files\\Torchlight Infinite\\Game",
        ] {
            base_dirs.push(PathBuf::from(base));
        }

        let mut best: Option<(PathBuf, std::time::SystemTime)> = None;
        for base in base_dirs {
            for tl_dir in ["TorchLight", "Torchlight"] {
                let candidate = base
                    .join("UE_game")
                    .join(tl_dir)
                    .join("Saved")
                    .join("Logs")
                    .join("UE_game.log");
                if let Ok(meta) = fs::metadata(&candidate)
                    && meta.is_file()
                {
                    let mtime = meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    if best.as_ref().is_none_or(|(_, bt)| mtime > *bt) {
                        best = Some((candidate, mtime));
                    }
                }
            }
        }
        best.map(|(p, _)| p)
    }

    pub fn is_connected(&self) -> bool {
        self.path.as_ref().is_some_and(|p| p.exists())
    }

    fn drain_complete_lines(&mut self, state_only: bool) {
        let mut carry = std::mem::take(&mut self.carry);
        let mut events = std::mem::take(&mut self.events);
        let mut cursor = 0usize;
        while let Some(rel_nl) = carry[cursor..].find('\n') {
            let line_end = cursor + rel_nl;
            let line = carry[cursor..line_end].trim_end_matches('\r');
            self.process_line_internal(line, &mut events, state_only);
            cursor = line_end + 1;
        }
        if cursor > 0 {
            carry.drain(..cursor);
        }
        self.carry = carry;
        self.events = events;
    }

    pub fn poll(&mut self) -> &[LogEvent] {
        self.events.clear();
        if self.path.is_none() {
            self.path = Self::discover_log_path();
        }
        let Some(ref path) = self.path else {
            return &self.events;
        };

        let Ok(meta) = fs::metadata(path) else {
            self.file = None;
            return &self.events;
        };
        let len = meta.len();

        if self.file.is_none() {
            self.file = File::open(path).ok();
        }

        if !self.initialized {
            self.initialized = true;
            self.offset = len;
            let tail_start = len.saturating_sub(4 * 1024 * 1024);
            if let Some(mut file) = self.file.take() {
                if file.seek(SeekFrom::Start(tail_start)).is_ok() {
                    let mut remaining = (len - tail_start) as usize;
                    while remaining > 0 {
                        let chunk_len = remaining.min(self.buf.len());
                        let Ok(n) = file.read(&mut self.buf[..chunk_len]) else {
                            break;
                        };
                        if n == 0 {
                            break;
                        }
                        remaining = remaining.saturating_sub(n);
                        self.carry
                            .push_str(&String::from_utf8_lossy(&self.buf[..n]));
                        self.drain_complete_lines(true);
                    }
                    self.carry.clear();
                    let _ = file.seek(SeekFrom::Start(len));
                }
                self.file = Some(file);
            }
            return &self.events;
        } else if len < self.offset {
            self.offset = 0;
            self.carry.clear();
            self.file = File::open(path).ok();
        }

        if len == self.offset {
            return &self.events;
        }

        let Some(mut file) = self.file.take() else {
            return &self.events;
        };
        if file.seek(SeekFrom::Start(self.offset)).is_err() {
            return &self.events;
        }

        let mut remaining = (len - self.offset).min(512 * 1024) as usize;
        while remaining > 0 {
            let chunk_len = remaining.min(self.buf.len());
            let Ok(n) = file.read(&mut self.buf[..chunk_len]) else {
                break;
            };
            if n == 0 {
                break;
            }
            self.offset += n as u64;
            remaining = remaining.saturating_sub(n);
            self.carry
                .push_str(&String::from_utf8_lossy(&self.buf[..n]));
            self.drain_complete_lines(false);
        }
        self.file = Some(file);
        &self.events
    }

    #[cfg(test)]
    fn process_line(&mut self, line: &str, out: &mut Vec<LogEvent>) {
        self.process_line_internal(line, out, false);
    }

    fn process_line_internal(&mut self, line: &str, out: &mut Vec<LogEvent>, state_only: bool) {
        if !line.contains("S16")
            && !line.contains("Eating_red_screen")
            && !line.contains("ColoredPop")
        {
            return;
        }
        if line.contains("LogDataTable")
            || line.contains("AssetLoad@")
            || line.contains("CardMotion")
            || line.contains("CardPose")
            || line.contains("BGM_")
        {
            return;
        }

        let now = Instant::now();
        if line.contains("Btn_S16Board_ActionCell") {
            self.last_action_at = Some(now);
        }
        if line.contains("Eating_red_screen") {
            self.last_eat_at = Some(now);
        }
        if line.contains("ColoredPop_Confirm") {
            self.last_gift_confirm_at = Some(now);
        }

        if let Some((run_id, is_settled)) = parse_s16_bgm_begin(line) {
            out.push(LogEvent::RunBegin { run_id, is_settled });
        }

        if let Some((cur, max)) = parse_s16_last_day(line) {
            if max > 0 {
                self.saw_cur_zero = cur == 0;
            }
            out.push(LogEvent::DayCheck { cur, max });
        }

        if line.contains("S16BoardRelicPop Run") {
            out.push(LogEvent::RelicPopState(true));
        } else if line.contains("S16BoardRelicPop Destory")
            || line.contains("S16GamePlayEntry Run")
            || line.contains("S16GamePlayCheckOut Run")
            || line.contains("S16BoardMain Destory")
        {
            out.push(LogEvent::RelicPopState(false));
        }

        // Run over vs Run new (from desktop/lib/follow-adapter.js)
        // S16GamePlayEntry Run is the pre-run "Start Warden's Duty" screen between runs,
        // and "[S16LastDay] check cur=... max=0" is the end-of-run results screen.
        if line.contains("Btn_S16GamePlay_CheckOut_AbandonChallenge")
            || line.contains("S16GamePlayEntry Run")
            || (line.contains("[S16LastDay] check cur=") && line.contains("max=0"))
        {
            out.push(LogEvent::RunOver);
        } else if line.contains("Btn_S16GamePlay_Start")
            || line.contains("S16BoardTaskPop Run")
            || line.contains("S16BoardRelicPop Run")
            || line.contains("S16BoardMain Run")
        {
            out.push(LogEvent::RunNew);
        }

        // Riot auto-fold / unfold (from desktop/lib/follow-adapter.js)
        if line.contains("S16GamePlayCheckOut Run") || line.contains("UI_S16_Riot_Interface") {
            out.push(LogEvent::Riot(true));
        } else if line.contains("S16GamePlayEntry Run")
            || line.contains("Btn_S16GamePlay_Start")
            || line.contains("S16BoardTaskPop Run")
            || line.contains("S16BoardRelicPop Run")
            || line.contains("S16BoardMain Run")
        {
            out.push(LogEvent::Riot(false));
        }

        if state_only {
            return;
        }

        // Exact triggers from js/follow.js DEFAULT_RULES + S16BoardNewDayPop Destory
        if line.contains("S16BoardTaskPop Run") {
            out.push(LogEvent::ObjectivePopup);
        } else if line.contains("S16BoardRelicPop Run")
            || line.contains("Btn_S16Board_RelicPop_Reroll")
        {
            let fresh = self.saw_cur_zero;
            self.saw_cur_zero = false;
            out.push(LogEvent::InstrumentsPopup { fresh });
        } else if line.contains("S16BoardNewDayPop Run") {
            out.push(LogEvent::SkipPending);
        } else if line.contains("S16BoardNewDayPop Destory") {
            out.push(LogEvent::BoardTrigger {
                settle_ms: 450,
                treasure_view: false,
            });
        } else if line.contains("Btn_S16Board_Top_Speed") {
            out.push(LogEvent::SpeedFlip);
        } else if line.contains("S16BoardColoredPop Run") {
            let after = self.last_action_at.and_then(|act| {
                if now.duration_since(act).as_millis() <= 6000 {
                    let is_consume = self
                        .last_eat_at
                        .is_some_and(|eat| eat + std::time::Duration::from_millis(500) >= act);
                    Some(if is_consume {
                        TreasureOrigin::Consume
                    } else {
                        TreasureOrigin::Swap
                    })
                } else {
                    None
                }
            });
            out.push(LogEvent::TreasurePopup { after });
        } else if line.contains("S16BoardColoredPop Destory") {
            let confirmed_recently = self
                .last_gift_confirm_at
                .is_some_and(|g| now.duration_since(g).as_millis() < 3000);
            out.push(LogEvent::BoardTrigger {
                settle_ms: 600,
                treasure_view: !confirmed_recently,
            });
        } else if (line.contains("[S16LastDay] check cur=") && !line.contains("max=0"))
            || line.contains("UI_S16_Card_Up")
            || line.contains("Btn_S16Board_ActionCell")
            || line.contains("UI_S16_Click_Eat")
            || line.contains("UI_S16_Eating_red_screen")
            || line.contains("UI_S16_Prison_Relock")
            || line.contains("UI_S16_Click_Cancel")
            || line.contains("Btn_S16Board_EventPop_Confirm")
        {
            out.push(LogEvent::BoardTrigger {
                settle_ms: 1000,
                treasure_view: false,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s16_log_triggers() {
        let mut lw = LogWatcher {
            path: None,
            file: None,
            buf: vec![0u8; 1024],
            offset: 0,
            carry: String::new(),
            events: Vec::new(),
            initialized: true,
            last_action_at: None,
            last_eat_at: None,
            last_gift_confirm_at: None,
            saw_cur_zero: false,
        };
        let mut evs = Vec::new();
        lw.process_line("[2026.10.17] S16BoardTaskPop Run", &mut evs);
        assert!(evs.contains(&LogEvent::ObjectivePopup));
        assert!(evs.contains(&LogEvent::RunNew));

        evs.clear();
        lw.process_line("[2026.10.17] [S16LastDay] check cur=0 max=12", &mut evs);
        assert!(evs.contains(&LogEvent::DayCheck { cur: 0, max: 12 }));
        lw.process_line("[2026.10.17] S16BoardRelicPop Run", &mut evs);
        assert!(evs.contains(&LogEvent::InstrumentsPopup { fresh: true }));
        assert!(evs.contains(&LogEvent::RelicPopState(true)));

        evs.clear();
        lw.process_line(
            "[2026.10.17] [S16BGM] board Begin runId=425a86a4-c923-11f1-8fcd-00000000005b IsSettled=false Result=-1",
            &mut evs,
        );
        assert!(evs.iter().any(|e| matches!(
            e,
            LogEvent::RunBegin {
                is_settled: false,
                ..
            }
        )));

        evs.clear();
        lw.process_line("[2026.10.17] S16BoardNewDayPop Destory", &mut evs);
        assert!(evs.contains(&LogEvent::BoardTrigger {
            settle_ms: 450,
            treasure_view: false
        }));

        evs.clear();
        lw.process_line("[2026.10.17] S16GamePlayCheckOut Run", &mut evs);
        assert!(evs.contains(&LogEvent::Riot(true)));
        assert!(evs.contains(&LogEvent::RelicPopState(false)));

        evs.clear();
        lw.process_line(
            "[2026.10.17] Btn_S16GamePlay_CheckOut_AbandonChallenge",
            &mut evs,
        );
        assert!(evs.contains(&LogEvent::RunOver));
    }
}
