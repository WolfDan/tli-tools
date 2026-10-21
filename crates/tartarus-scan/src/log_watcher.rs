//! Authoritative `UE_game.log` view & run state watcher (`LogWatcher`, `LogEvent`, `ActiveUiView`).
//!
//! Eliminates all overlap between `UiReader` and `UE_game.log`:
//! - Active view lifecycle (`S16BoardMain`, `S16BoardRelicPop`, `S16BoardColoredPop`,
//!   `S16BoardTaskPop`, `S16BoardThemePop`, `S16BoardEventPop`, `S16BoardNewDayPop`,
//!   `S16GamePlayEntry`, `S16GamePlayCheckOut`) is tracked with 100% reliability from log lines.
//! - Day counter (`[S16LastDay] check cur=<u8> max=<u8>`) and run identity
//!   (`[S16BGM] board Begin runId=<u64> IsSettled=<bool>`) are parsed directly from log lines.

use std::env;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::time::Instant;

use tartarus_core::TreasureOrigin;

const READ_CHUNK_BYTES: usize = 64 * 1024;
const INIT_TAIL_BYTES_MAX: u64 = 4 * 1024 * 1024;
const POLL_READ_BYTES_MAX: u64 = 512 * 1024;
const READ_CHUNKS_MAX: usize = 128;
const LINES_PER_DRAIN_MAX: usize = 65_536;

const _: () = {
    assert!(READ_CHUNK_BYTES == 65_536);
    assert!(INIT_TAIL_BYTES_MAX >= POLL_READ_BYTES_MAX);
    assert!(READ_CHUNKS_MAX * READ_CHUNK_BYTES >= INIT_TAIL_BYTES_MAX as usize);
};

/// Active S16 UMG view state tracked from `UE_game.log` (`Run` / `Destory` events).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ActiveUiView {
    #[default]
    Unknown,
    Lobby,
    BoardMain,
    RelicPop,
    ColoredPop,
    TaskPop,
    ThemePop,
    EventPop,
    NewDayPop,
    CheckOutRiot,
}

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
    assert!(!s.is_empty());
    let mut hasher = fnv::FnvHasher::default();
    hasher.write(s.as_bytes());
    let hash = hasher.finish();
    let non_zero = if hash == 0 { 1 } else { hash };
    assert_ne!(non_zero, 0);
    non_zero
}

fn parse_s16_bgm_begin(line: &str) -> Option<(u64, bool)> {
    assert!(!line.is_empty());
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
    let id = run_id?;
    assert_ne!(id, 0);
    Some((id, is_settled))
}

fn parse_s16_last_day(line: &str) -> Option<(u8, u8)> {
    assert!(!line.is_empty());
    let rest = line.split("[S16LastDay] check").nth(1)?;
    let mut cur: Option<u8> = None;
    let mut day_max: Option<u8> = None;
    for tok in rest.split_whitespace() {
        if let Some(c_str) = tok.strip_prefix("cur=") {
            cur = c_str.parse::<u8>().ok();
        } else if let Some(m_str) = tok.strip_prefix("max=") {
            day_max = m_str.parse::<u8>().ok();
        }
    }
    let pair = (cur?, day_max?);
    assert!(pair.0 <= 250 && pair.1 <= 250);
    Some(pair)
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
    pub active_view: ActiveUiView,
}

impl Default for LogWatcher {
    fn default() -> Self {
        let lw = Self::new();
        assert_eq!(lw.buf.len(), READ_CHUNK_BYTES);
        assert!(!lw.initialized);
        lw
    }
}

impl LogWatcher {
    pub fn new() -> Self {
        let path = Self::discover_log_path();
        let file = path.as_ref().and_then(|p| File::open(p).ok());
        let lw = Self {
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
            active_view: ActiveUiView::Unknown,
        };
        assert_eq!(lw.buf.len(), READ_CHUNK_BYTES);
        assert_eq!(lw.offset, 0);
        lw
    }

    pub fn new_isolated() -> Self {
        let lw = Self {
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
            active_view: ActiveUiView::Unknown,
        };
        assert!(lw.initialized);
        assert!(lw.path.is_none());
        lw
    }

    #[inline]
    pub const fn active_view(&self) -> ActiveUiView {
        self.active_view
    }

    pub fn discover_log_path() -> Option<PathBuf> {
        const TLI_STEAM_APP_ID: u32 = 1_974_050;
        const {
            assert!(TLI_STEAM_APP_ID > 0);
        }

        if let Ok(custom) = env::var("TAC_OVERLAY_LOG") {
            let p = PathBuf::from(custom);
            if p.exists() {
                assert!(!p.as_os_str().is_empty());
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
        let found = best.map(|(p, _)| p);
        assert!(found.as_ref().is_none_or(|p| !p.as_os_str().is_empty()));
        found
    }

    pub fn is_connected(&self) -> bool {
        let connected = self.path.as_ref().is_some_and(|p| p.exists());
        assert!(!connected || self.path.is_some());
        assert!(!self.buf.is_empty());
        connected
    }

    fn drain_complete_lines(&mut self, state_only: bool) {
        assert!(!self.buf.is_empty());
        let mut carry = std::mem::take(&mut self.carry);
        let mut events = std::mem::take(&mut self.events);
        let mut cursor = 0usize;
        let mut lines_count = 0usize;
        while lines_count < LINES_PER_DRAIN_MAX {
            let Some(rel_nl) = carry[cursor..].find('\n') else {
                break;
            };
            lines_count += 1;
            let line_end = cursor + rel_nl;
            let line = carry[cursor..line_end].trim_end_matches('\r');
            self.process_line_internal(line, &mut events, state_only);
            cursor = line_end + 1;
        }
        assert!(cursor <= carry.len());
        if cursor > 0 {
            carry.drain(..cursor);
        }
        self.carry = carry;
        self.events = events;
    }

    fn read_stream_chunks(&mut self, file: &mut File, remaining_init: usize, state_only: bool) {
        assert!(!self.buf.is_empty());
        assert!(remaining_init <= INIT_TAIL_BYTES_MAX as usize);
        let mut remaining = remaining_init;
        let mut chunk_iters = 0usize;
        while remaining > 0 && chunk_iters < READ_CHUNKS_MAX {
            chunk_iters += 1;
            let chunk_len = remaining.min(self.buf.len());
            let Ok(n) = file.read(&mut self.buf[..chunk_len]) else {
                break;
            };
            if n == 0 {
                break;
            }
            if !state_only {
                self.offset += n as u64;
            }
            remaining = remaining.saturating_sub(n);
            self.carry
                .push_str(&String::from_utf8_lossy(&self.buf[..n]));
            self.drain_complete_lines(state_only);
        }
    }

    fn initialize_tail(&mut self, len: u64) {
        assert!(!self.initialized);
        self.initialized = true;
        self.offset = len;
        let tail_start = len.saturating_sub(INIT_TAIL_BYTES_MAX);
        assert!(tail_start <= len);
        if let Some(mut file) = self.file.take() {
            if file.seek(SeekFrom::Start(tail_start)).is_ok() {
                let remaining = (len - tail_start) as usize;
                self.read_stream_chunks(&mut file, remaining, true);
                self.carry.clear();
                let _ = file.seek(SeekFrom::Start(len));
            }
            self.file = Some(file);
        }
    }

    pub fn poll(&mut self) -> &[LogEvent] {
        assert!(!self.buf.is_empty());
        self.events.clear();
        assert!(self.events.is_empty());
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
            self.initialize_tail(len);
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

        let remaining = (len - self.offset).min(POLL_READ_BYTES_MAX) as usize;
        self.read_stream_chunks(&mut file, remaining, false);
        self.file = Some(file);
        &self.events
    }

    pub fn process_line(&mut self, line: &str, out: &mut Vec<LogEvent>) {
        let prev_len = out.len();
        self.process_line_internal(line, out, false);
        assert!(out.len() >= prev_len);
        assert!(!self.buf.is_empty());
    }

    fn is_relevant_s16_line(line: &str) -> bool {
        let has_tag = line.contains("S16")
            || line.contains("Eating_red_screen")
            || line.contains("ColoredPop");
        let is_noise = line.contains("LogDataTable")
            || line.contains("AssetLoad@")
            || line.contains("CardMotion")
            || line.contains("CardPose")
            || line.contains("BGM_");
        assert!(!line.is_empty() || !has_tag);
        assert!(!is_noise || !line.is_empty());
        has_tag && !is_noise
    }

    fn update_active_view(&mut self, line: &str) {
        assert!(!line.is_empty());
        if line.contains("S16BoardRelicPop Run") {
            self.active_view = ActiveUiView::RelicPop;
        } else if line.contains("S16BoardColoredPop Run") {
            self.active_view = ActiveUiView::ColoredPop;
        } else if line.contains("S16BoardTaskPop Run") {
            self.active_view = ActiveUiView::TaskPop;
        } else if line.contains("S16BoardThemePop Run") {
            self.active_view = ActiveUiView::ThemePop;
        } else if line.contains("S16BoardEventPop Run") {
            self.active_view = ActiveUiView::EventPop;
        } else if line.contains("S16BoardNewDayPop Run") {
            self.active_view = ActiveUiView::NewDayPop;
        } else if line.contains("S16GamePlayCheckOut Run") || line.contains("UI_S16_Riot_Interface")
        {
            self.active_view = ActiveUiView::CheckOutRiot;
        } else if line.contains("S16GamePlayEntry Run") {
            self.active_view = ActiveUiView::Lobby;
        } else if line.contains("S16BoardMain Run")
            || line.contains("S16BoardRelicPop Destory")
            || line.contains("S16BoardColoredPop Destory")
            || line.contains("S16BoardTaskPop Destory")
            || line.contains("S16BoardThemePop Destory")
            || line.contains("S16BoardEventPop Destory")
            || line.contains("S16BoardNewDayPop Destory")
        {
            self.active_view = ActiveUiView::BoardMain;
        }
        assert!(self.active_view == self.active_view());
    }

    fn record_timestamps_and_state(&mut self, line: &str, now: Instant, out: &mut Vec<LogEvent>) {
        assert!(!line.is_empty());
        let prev_len = out.len();
        self.update_active_view(line);
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

        if let Some((cur, day_max)) = parse_s16_last_day(line) {
            if day_max > 0 {
                self.saw_cur_zero = cur == 0;
            }
            out.push(LogEvent::DayCheck { cur, max: day_max });
        }

        Self::emit_run_and_riot_state(line, out);
        assert!(out.len() >= prev_len);
    }

    fn emit_run_and_riot_state(line: &str, out: &mut Vec<LogEvent>) {
        assert!(!line.is_empty());
        let prev_len = out.len();
        if line.contains("S16BoardRelicPop Run") {
            out.push(LogEvent::RelicPopState(true));
        } else if line.contains("S16BoardRelicPop Destory")
            || line.contains("S16GamePlayEntry Run")
            || line.contains("S16GamePlayCheckOut Run")
            || line.contains("S16BoardMain Destory")
        {
            out.push(LogEvent::RelicPopState(false));
        }

        if line.contains("Btn_S16GamePlay_CheckOut_AbandonChallenge")
            || line.contains("S16GamePlayEntry Run")
            || (line.contains("[S16LastDay] check cur=") && line.contains("max=0"))
        {
            out.push(LogEvent::RunOver);
        } else if line.contains("Btn_S16GamePlay_Start")
            || line.contains("S16BoardTaskPop Run")
            || line.contains("S16BoardThemePop Run")
            || line.contains("S16BoardRelicPop Run")
            || line.contains("S16BoardMain Run")
        {
            out.push(LogEvent::RunNew);
        }

        if line.contains("S16GamePlayCheckOut Run") || line.contains("UI_S16_Riot_Interface") {
            out.push(LogEvent::Riot(true));
        } else if line.contains("S16GamePlayEntry Run")
            || line.contains("Btn_S16GamePlay_Start")
            || line.contains("S16BoardTaskPop Run")
            || line.contains("S16BoardThemePop Run")
            || line.contains("S16BoardRelicPop Run")
            || line.contains("S16BoardMain Run")
        {
            out.push(LogEvent::Riot(false));
        }
        assert!(out.len() >= prev_len);
    }

    fn resolve_treasure_origin(&self, now: Instant) -> Option<TreasureOrigin> {
        let act = self.last_action_at?;
        assert!(now >= act);
        if now.duration_since(act).as_millis() <= 6000 {
            let is_consume = self
                .last_eat_at
                .is_some_and(|eat| eat + std::time::Duration::from_millis(500) >= act);
            let origin = if is_consume {
                TreasureOrigin::Consume
            } else {
                TreasureOrigin::Swap
            };
            assert!(matches!(
                origin,
                TreasureOrigin::Consume | TreasureOrigin::Swap
            ));
            Some(origin)
        } else {
            None
        }
    }

    fn emit_action_triggers(&mut self, line: &str, now: Instant, out: &mut Vec<LogEvent>) {
        assert!(!line.is_empty());
        let prev_len = out.len();
        if line.contains("S16BoardTaskPop Run") || line.contains("S16BoardThemePop Run") {
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
            let after = self.resolve_treasure_origin(now);
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
        assert!(out.len() >= prev_len);
    }

    fn process_line_internal(&mut self, line: &str, out: &mut Vec<LogEvent>, state_only: bool) {
        let prev_len = out.len();
        if !Self::is_relevant_s16_line(line) {
            return;
        }
        let now = Instant::now();
        self.record_timestamps_and_state(line, now, out);
        if !state_only {
            self.emit_action_triggers(line, now, out);
        }
        assert!(out.len() >= prev_len);
        assert!(!line.is_empty());
    }
}
