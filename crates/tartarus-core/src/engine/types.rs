use serde::{Deserialize, Serialize};

pub const VERSION: u32 = 3;
pub const CELL_COUNT: usize = 16;
pub const OFFICE: u8 = 0;
pub const SUPREME: u8 = 4;
pub const RAINBOW: u8 = 5;
pub const CHALLENGE_DAYS: u8 = 12;
pub const START_VOUCHERS: u8 = 6;

pub const FREEDOM_CELLS: usize = 3;
pub const CROW_CELLS: usize = 4;
pub const HUNT_BELOW: u8 = 2;
pub const INSTIGATE_EVENTS: usize = 2;
pub const APPETITE_TRIES: u32 = 5;
pub const APPETITE_ODDS: f64 = 0.5;

pub const PRESSURE: [u16; 6] = [2, 4, 8, 16, 32, 65];
pub const WORTH: [u32; 6] = [1, 3, 9, 27, 81, 400];
pub const START_CELLS: [u8; 6] = [1, 2, 5, 6, 10, 11];

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum Beast {
    #[default]
    Mammoth = 0,
    Lion = 1,
    Wolf = 2,
    Rat = 3,
}

pub const BEASTS: [Beast; 4] = [Beast::Mammoth, Beast::Lion, Beast::Wolf, Beast::Rat];

impl Beast {
    #[inline]
    pub const fn idx(self) -> usize {
        self as usize
    }

    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Beast::Mammoth => "mammoth",
            Beast::Lion => "lion",
            Beast::Wolf => "wolf",
            Beast::Rat => "rat",
        }
    }

    #[inline]
    pub const fn name(self) -> &'static str {
        match self {
            Beast::Mammoth => "Mammoth",
            Beast::Lion => "Lion",
            Beast::Wolf => "Wolf",
            Beast::Rat => "Rat",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "mammoth" => Some(Beast::Mammoth),
            "lion" => Some(Beast::Lion),
            "wolf" => Some(Beast::Wolf),
            "rat" => Some(Beast::Rat),
            _ => None,
        }
    }

    #[inline]
    pub const fn primary_prey(self) -> Beast {
        match self {
            Beast::Mammoth => Beast::Lion,
            Beast::Lion => Beast::Wolf,
            Beast::Wolf => Beast::Rat,
            Beast::Rat => Beast::Mammoth,
        }
    }

    #[inline]
    pub const fn second_meal(self) -> Option<Beast> {
        match self {
            Beast::Mammoth => Some(Beast::Wolf),
            Beast::Lion => Some(Beast::Rat),
            Beast::Wolf | Beast::Rat => None,
        }
    }
}

#[inline]
pub const fn tier_name(tier: u8) -> &'static str {
    match tier {
        0 => "Normal",
        1 => "Magic",
        2 => "Rare",
        3 => "Legendary",
        4 => "Supreme",
        _ => "Rainbow",
    }
}

pub const CELL_LABELS: [&str; 16] = [
    "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12", "13", "14", "15", "16",
];

#[inline]
pub const fn cell_label(n: u8) -> &'static str {
    CELL_LABELS[(n as usize) & 15]
}

const INMATE_DESCRIPTIONS: [[&str; 4]; 6] = [
    ["Normal Mammoth", "Normal Lion", "Normal Wolf", "Normal Rat"],
    ["Magic Mammoth", "Magic Lion", "Magic Wolf", "Magic Rat"],
    ["Rare Mammoth", "Rare Lion", "Rare Wolf", "Rare Rat"],
    [
        "Legendary Mammoth",
        "Legendary Lion",
        "Legendary Wolf",
        "Legendary Rat",
    ],
    [
        "Supreme Mammoth",
        "Supreme Lion",
        "Supreme Wolf",
        "Supreme Rat",
    ],
    [
        "Rainbow Mammoth",
        "Rainbow Lion",
        "Rainbow Wolf",
        "Rainbow Rat",
    ],
];

#[inline]
pub const fn describe_inmate(inmate: Inmate) -> &'static str {
    let t = if (inmate.tier as usize) < 5 {
        inmate.tier as usize
    } else {
        5
    };
    INMATE_DESCRIPTIONS[t][inmate.beast.idx()]
}

#[inline]
pub fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        format!("1 {word}")
    } else {
        format!("{n} {word}s")
    }
}

#[inline]
pub const fn steps_between(a: u8, b: u8) -> u8 {
    (b + 16 - a) % 16
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum ToolId {
    #[default]
    Revenge = 0,
    Muzzle = 1,
    Saw = 2,
    Keys = 3,
    Rope = 4,
    Fire = 5,
    Ironbox = 6,
    Skewer = 7,
    Hook = 8,
    Pestle = 9,
    Scales = 10,
    Tongs = 11,
    Guillotine = 12,
    Axe = 13,
    Shackles = 14,
    Collar = 15,
    Forged = 16,
    Dogtag = 17,
    Cage = 18,
    Blacklist = 19,
    Shortchain = 20,
    Maiden = 21,
    Whip = 22,
    Ravenous = 23,
    Scapegoat = 24,
    Stake = 25,
    Coin = 26,
    Crown = 27,
    Maul = 28,
    Claws = 29,
    Trap = 30,
}

pub const ALL_TOOLS: [ToolId; 31] = [
    ToolId::Revenge,
    ToolId::Muzzle,
    ToolId::Saw,
    ToolId::Keys,
    ToolId::Rope,
    ToolId::Fire,
    ToolId::Ironbox,
    ToolId::Skewer,
    ToolId::Hook,
    ToolId::Pestle,
    ToolId::Scales,
    ToolId::Tongs,
    ToolId::Guillotine,
    ToolId::Axe,
    ToolId::Shackles,
    ToolId::Collar,
    ToolId::Forged,
    ToolId::Dogtag,
    ToolId::Cage,
    ToolId::Blacklist,
    ToolId::Shortchain,
    ToolId::Maiden,
    ToolId::Whip,
    ToolId::Ravenous,
    ToolId::Scapegoat,
    ToolId::Stake,
    ToolId::Coin,
    ToolId::Crown,
    ToolId::Maul,
    ToolId::Claws,
    ToolId::Trap,
];

const fn count_ascii_alnum(s: &str) -> u8 {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut cnt = 0u8;
    while i < bytes.len() {
        if bytes[i].is_ascii_alphanumeric() {
            cnt += 1;
        }
        i += 1;
    }
    cnt
}

const TOOL_NAME_ALNUM_LEN: [u8; 31] = {
    let mut arr = [0u8; 31];
    let mut i = 0;
    while i < 31 {
        arr[i] = count_ascii_alnum(ALL_TOOLS[i].name());
        i += 1;
    }
    arr
};

impl ToolId {
    #[inline]
    pub const fn idx(self) -> usize {
        self as usize
    }

    #[inline]
    pub const fn name_alnum_len(self) -> usize {
        TOOL_NAME_ALNUM_LEN[self as usize] as usize
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            ToolId::Revenge => "revenge",
            ToolId::Muzzle => "muzzle",
            ToolId::Saw => "saw",
            ToolId::Keys => "keys",
            ToolId::Rope => "rope",
            ToolId::Fire => "fire",
            ToolId::Ironbox => "ironbox",
            ToolId::Skewer => "skewer",
            ToolId::Hook => "hook",
            ToolId::Pestle => "pestle",
            ToolId::Scales => "scales",
            ToolId::Tongs => "tongs",
            ToolId::Guillotine => "guillotine",
            ToolId::Axe => "axe",
            ToolId::Shackles => "shackles",
            ToolId::Collar => "collar",
            ToolId::Forged => "forged",
            ToolId::Dogtag => "dogtag",
            ToolId::Cage => "cage",
            ToolId::Blacklist => "blacklist",
            ToolId::Shortchain => "shortchain",
            ToolId::Maiden => "maiden",
            ToolId::Whip => "whip",
            ToolId::Ravenous => "ravenous",
            ToolId::Scapegoat => "scapegoat",
            ToolId::Stake => "stake",
            ToolId::Coin => "coin",
            ToolId::Crown => "crown",
            ToolId::Maul => "maul",
            ToolId::Claws => "claws",
            ToolId::Trap => "trap",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            ToolId::Revenge => "Avenger's Casket",
            ToolId::Muzzle => "Raging Beast's Agonizing Muzzle",
            ToolId::Saw => "Rusty Saw",
            ToolId::Keys => "Lost Ring of Keys",
            ToolId::Rope => "Hangman's Noose",
            ToolId::Fire => "Searing Branding Iron",
            ToolId::Ironbox => "Squirming Iron Box: Colossal Rat",
            ToolId::Skewer => "Skewered Rat Heads: Colossal Rat",
            ToolId::Hook => "Sorting Hook",
            ToolId::Pestle => "Iron Pestle",
            ToolId::Scales => "Warden's Scales",
            ToolId::Tongs => "Iron Tongs of Lust: Mammoth",
            ToolId::Guillotine => "Fanged Guillotine: Alpha Wolf",
            ToolId::Axe => "Gaoler's Execution Axe",
            ToolId::Shackles => "Long-Chained Shackles",
            ToolId::Collar => "Maddened Beast's Barbed Collar",
            ToolId::Forged => "Forged Key",
            ToolId::Dogtag => "Felon's Dog Tag",
            ToolId::Cage => "Crowded Beastcage",
            ToolId::Blacklist => "Executioner's Blacklist",
            ToolId::Shortchain => "Short-Chained Shackles",
            ToolId::Maiden => "Severed Iron Maiden",
            ToolId::Whip => "Barbed Whip",
            ToolId::Ravenous => "Ferocious Beast's Ravenous Chains",
            ToolId::Scapegoat => "Scapegoat's Guillotine",
            ToolId::Stake => "Burning Stake",
            ToolId::Coin => "Kin's Coin of Fate",
            ToolId::Crown => "Iron Crown of the Prison King: Berserk Lion",
            ToolId::Maul => "Elephant-Foot Maul: Mammoth",
            ToolId::Claws => "Filthy Claws: Alpha Wolf",
            ToolId::Trap => "Blood-Stained Trap: Berserk Lion",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        ALL_TOOLS.into_iter().find(|t| t.as_str() == s)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct ToolMask(pub u32);

impl ToolMask {
    #[inline]
    pub const fn has(self, tool: ToolId) -> bool {
        (self.0 & (1u32 << (tool as u32))) != 0
    }

    #[inline]
    pub fn insert(&mut self, tool: ToolId) {
        self.0 |= 1u32 << (tool as u32);
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum GiftId {
    Ring = 0,
    Horn = 1,
    Wine = 2,
    Brand = 3,
}

pub const ALL_GIFTS: [GiftId; 4] = [GiftId::Ring, GiftId::Horn, GiftId::Wine, GiftId::Brand];

impl GiftId {
    pub const fn as_str(self) -> &'static str {
        match self {
            GiftId::Ring => "ring",
            GiftId::Horn => "horn",
            GiftId::Wine => "wine",
            GiftId::Brand => "brand",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            GiftId::Ring => "Three-Headed Thumb Ring",
            GiftId::Horn => "Brass Horn of Strife",
            GiftId::Wine => "Crimson Vintage",
            GiftId::Brand => "Brand of Original Sin",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        ALL_GIFTS.into_iter().find(|g| g.as_str() == s)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum ChoiceId {
    Tool(ToolId),
    Gift(GiftId),
}

impl ChoiceId {
    pub const fn as_str(self) -> &'static str {
        match self {
            ChoiceId::Tool(t) => t.as_str(),
            ChoiceId::Gift(g) => g.as_str(),
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            ChoiceId::Tool(t) => t.name(),
            ChoiceId::Gift(g) => g.name(),
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        if let Some(t) = ToolId::parse(s) {
            Some(ChoiceId::Tool(t))
        } else {
            GiftId::parse(s).map(ChoiceId::Gift)
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum EventKind {
    Chain = 0,
    Execution = 1,
    Trickster = 2,
    Appetite = 3,
    Convict = 4,
    Crow = 5,
    Freedom = 6,
    Usurp1 = 7,
    Usurp2 = 8,
    Usurp3 = 9,
    Usurp4 = 10,
    Ratway = 11,
    Hunt = 12,
    Portion = 13,
    Fortune = 14,
    Archenemy = 15,
    Transfer = 16,
}

pub const INITIAL_EVENT_POOL: [EventKind; 14] = [
    EventKind::Chain,
    EventKind::Execution,
    EventKind::Trickster,
    EventKind::Appetite,
    EventKind::Convict,
    EventKind::Crow,
    EventKind::Freedom,
    EventKind::Usurp1,
    EventKind::Ratway,
    EventKind::Hunt,
    EventKind::Portion,
    EventKind::Fortune,
    EventKind::Archenemy,
    EventKind::Transfer,
];

impl EventKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            EventKind::Chain => "chain",
            EventKind::Execution => "execution",
            EventKind::Trickster => "trickster",
            EventKind::Appetite => "appetite",
            EventKind::Convict => "convict",
            EventKind::Crow => "crow",
            EventKind::Freedom => "freedom",
            EventKind::Usurp1 => "usurp1",
            EventKind::Usurp2 => "usurp2",
            EventKind::Usurp3 => "usurp3",
            EventKind::Usurp4 => "usurp4",
            EventKind::Ratway => "ratway",
            EventKind::Hunt => "hunt",
            EventKind::Portion => "portion",
            EventKind::Fortune => "fortune",
            EventKind::Archenemy => "archenemy",
            EventKind::Transfer => "transfer",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            EventKind::Chain => "Chain reaction",
            EventKind::Execution => "Execution",
            EventKind::Trickster => "Behind Closed Doors",
            EventKind::Appetite => "Ravenous Appetite",
            EventKind::Convict => "High-Risk Convict",
            EventKind::Crow => "Crow Whispers",
            EventKind::Freedom => "Taste of Freedom",
            EventKind::Usurp1 => "Usurpation I",
            EventKind::Usurp2 => "Usurpation II",
            EventKind::Usurp3 => "Usurpation III",
            EventKind::Usurp4 => "Usurpation IV",
            EventKind::Ratway => "Sewer Ratway",
            EventKind::Hunt => "\"Treasure\" Hunt",
            EventKind::Portion => "Extra Portion",
            EventKind::Fortune => "Shared Fortune",
            EventKind::Archenemy => "Archenemy",
            EventKind::Transfer => "Transfer of Convicts",
        }
    }

    pub const fn weight(self) -> u32 {
        match self {
            EventKind::Convict | EventKind::Freedom | EventKind::Trickster => 40,
            EventKind::Appetite | EventKind::Portion | EventKind::Archenemy => 30,
            EventKind::Fortune
            | EventKind::Execution
            | EventKind::Ratway
            | EventKind::Chain
            | EventKind::Transfer => 20,
            EventKind::Crow
            | EventKind::Hunt
            | EventKind::Usurp1
            | EventKind::Usurp2
            | EventKind::Usurp3
            | EventKind::Usurp4 => 10,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "chain" => Some(EventKind::Chain),
            "execution" | "efficiency" => Some(EventKind::Execution),
            "trickster" | "closed" => Some(EventKind::Trickster),
            "appetite" => Some(EventKind::Appetite),
            "convict" => Some(EventKind::Convict),
            "crow" | "whispers" => Some(EventKind::Crow),
            "freedom" => Some(EventKind::Freedom),
            "usurp1" => Some(EventKind::Usurp1),
            "usurp2" => Some(EventKind::Usurp2),
            "usurp3" => Some(EventKind::Usurp3),
            "usurp4" => Some(EventKind::Usurp4),
            "ratway" => Some(EventKind::Ratway),
            "hunt" | "treasure" => Some(EventKind::Hunt),
            "portion" => Some(EventKind::Portion),
            "fortune" => Some(EventKind::Fortune),
            "archenemy" => Some(EventKind::Archenemy),
            "transfer" => Some(EventKind::Transfer),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum ObjectiveId {
    King = 0,
    Walk = 1,
    Consume = 2,
    Incident = 3,
    Doors = 4,
    Other = 5,
}

impl ObjectiveId {
    pub const fn as_str(self) -> &'static str {
        match self {
            ObjectiveId::King => "king",
            ObjectiveId::Walk => "walk",
            ObjectiveId::Consume => "consume",
            ObjectiveId::Incident => "incident",
            ObjectiveId::Doors => "doors",
            ObjectiveId::Other => "other",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            ObjectiveId::King => "King of the Caged",
            ObjectiveId::Walk => "Prison Walk",
            ObjectiveId::Consume => "Survival of the Fittest",
            ObjectiveId::Incident => "Undercurrents",
            ObjectiveId::Doors => "Behind Cell Doors",
            ObjectiveId::Other => "Another objective",
        }
    }

    pub const fn need(self) -> u8 {
        match self {
            ObjectiveId::Walk => 18,
            ObjectiveId::Doors => 10,
            ObjectiveId::Consume => 4,
            ObjectiveId::Incident => 2,
            ObjectiveId::King | ObjectiveId::Other => 0,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "king" => Some(ObjectiveId::King),
            "walk" => Some(ObjectiveId::Walk),
            "consume" | "devour" => Some(ObjectiveId::Consume),
            "incident" => Some(ObjectiveId::Incident),
            "doors" | "open" => Some(ObjectiveId::Doors),
            "other" => Some(ObjectiveId::Other),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum StartEventId {
    None = 0,
    CageFight = 1,
    IronLaw = 2,
    Tyranny = 3,
    Opulence = 4,
}

impl StartEventId {
    pub const fn as_str(self) -> &'static str {
        match self {
            StartEventId::None => "none",
            StartEventId::CageFight => "cagefight",
            StartEventId::IronLaw => "ironlaw",
            StartEventId::Tyranny => "tyranny",
            StartEventId::Opulence => "opulence",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            StartEventId::None => "None",
            StartEventId::CageFight => "Cage Fight",
            StartEventId::IronLaw => "Iron Law",
            StartEventId::Tyranny => "Tyranny",
            StartEventId::Opulence => "Opulence",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "none" | "" => Some(StartEventId::None),
            "cagefight" | "foodchain" => Some(StartEventId::CageFight),
            "ironlaw" => Some(StartEventId::IronLaw),
            "tyranny" => Some(StartEventId::Tyranny),
            "opulence" => Some(StartEventId::Opulence),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum Phase {
    Patrol = 0,
    Manage = 1,
    Office = 2,
    Event = 3,
    Ended = 4,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum GameMode {
    Drill = 0,
    Challenge = 1,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum OfficeChoice {
    Bestow = 0,
    Instigate = 1,
}

impl OfficeChoice {
    pub const fn as_str(self) -> &'static str {
        match self {
            OfficeChoice::Bestow => "bestow",
            OfficeChoice::Instigate => "instigate",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum EscortKind {
    Light = 0,
    Heavy = 1,
}

impl EscortKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            EscortKind::Light => "light",
            EscortKind::Heavy => "heavy",
        }
    }

    pub const fn cap(self) -> u16 {
        match self {
            EscortKind::Light => 36,
            EscortKind::Heavy => 72,
        }
    }

    pub const fn cost(self) -> u8 {
        match self {
            EscortKind::Light => 1,
            EscortKind::Heavy => 3,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum ContractId {
    Mixed = 0,
    Cohort = 1,
    Veteran = 2,
    Elite = 3,
    Kin = 4,
    Muster = 5,
    Heavy = 6,
    Bounty = 7,
}

impl ContractId {
    pub const fn as_str(self) -> &'static str {
        match self {
            ContractId::Mixed => "mixed",
            ContractId::Cohort => "cohort",
            ContractId::Veteran => "veteran",
            ContractId::Elite => "elite",
            ContractId::Kin => "kin",
            ContractId::Muster => "muster",
            ContractId::Heavy => "heavy",
            ContractId::Bounty => "bounty",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            ContractId::Mixed => "Mixed convoy",
            ContractId::Cohort => "Work gang",
            ContractId::Veteran => "Veteran-led crew",
            ContractId::Elite => "Elite convoy",
            ContractId::Kin => "Kin escort",
            ContractId::Muster => "Elite muster",
            ContractId::Heavy => "Heavy convoy",
            ContractId::Bounty => "Heavy bounty",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "mixed" => Some(ContractId::Mixed),
            "cohort" => Some(ContractId::Cohort),
            "veteran" => Some(ContractId::Veteran),
            "elite" => Some(ContractId::Elite),
            "kin" => Some(ContractId::Kin),
            "muster" => Some(ContractId::Muster),
            "heavy" => Some(ContractId::Heavy),
            "bounty" => Some(ContractId::Bounty),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Contract {
    pub slot: u8,
    pub id: ContractId,
    pub deadline: u8,
    pub heavy: bool,
    pub bounty: Option<Beast>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct DeliveryRecord {
    pub slot: u8,
    pub day: u8,
    pub success: bool,
    pub points: u16,
    pub cost: u8,
    pub pressure: u16,
    pub escort: Option<EscortKind>,
    pub early: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ChallengeState {
    pub vouchers: u8,
    pub contracts: [Contract; 3],
    pub deliveries: [DeliveryRecord; 3],
    pub deliveries_len: u8,
    pub score: u16,
    pub window_open: bool,
    pub window_early: bool,
    pub settled: bool,
    pub passed: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct Inmate {
    pub id: u16,
    pub beast: Beast,
    pub tier: u8,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct CellState {
    pub n: u8,
    pub sealed: bool,
    pub inmate: Option<Inmate>,
    pub hidden: Option<Inmate>,
    pub known: u8,
    pub at_least: u8,
    pub scratch: bool,
    pub nest_due: Option<u8>,
    pub event: Option<EventKind>,
    pub event_known: bool,
    pub ivory: bool,
    pub seen: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct PreyLink {
    pub lion: u16,
    pub prey: u16,
    pub prey_beast: Beast,
    pub rounds: u8,
    pub next_day: u8,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct VaultEntry {
    pub id: u16,
    pub beast: Beast,
    pub day: u8,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum PendingKind {
    Tool = 0,
    Refit = 1,
    Gift = 2,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PendingChoice {
    pub kind: PendingKind,
    pub options: [ChoiceId; 4],
    pub options_len: u8,
    pub slot: u8,
    pub refreshed: bool,
    pub rerolls: u8,
}

impl Default for PendingChoice {
    fn default() -> Self {
        Self {
            kind: PendingKind::Tool,
            options: [ChoiceId::Tool(ToolId::Revenge); 4],
            options_len: 0,
            slot: 0,
            refreshed: false,
            rerolls: 0,
        }
    }
}

impl PendingChoice {
    #[inline]
    pub fn slice(&self) -> &[ChoiceId] {
        &self.options[..self.options_len as usize]
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct GameParams {
    pub drill_days: u8,
    pub destinations: u8,
    pub patrol_range: u8,
    pub event_two_doors: bool,
    pub fewer_doors_chance: f64,
    pub swaps_per_day: u8,
    pub reusable_cells: bool,
    pub chain_stop: bool,
    pub tier_weights: [f64; 5],
    pub swap_weights: [f64; 5],
    pub beast_weights: [f64; 4],
    pub double_eat_tier: u8,
    pub event_chance: f64,
    pub execution_same_tier: bool,
    pub execution_rerolls: bool,
    pub ivory_count: u8,
    pub ivory_includes_swapped: bool,
    pub prey_rounds: u8,
    pub scratch_stacks_base: bool,
    pub nest_days: u8,
    pub nest_tier: u8,
    pub rewards: bool,
    pub start_tools: u8,
}

impl Default for GameParams {
    fn default() -> Self {
        Self {
            drill_days: 20,
            destinations: 3,
            patrol_range: 6,
            event_two_doors: false,
            fewer_doors_chance: 0.0,
            swaps_per_day: 1,
            reusable_cells: true,
            chain_stop: true,
            tier_weights: [40.0, 38.0, 12.0, 8.0, 2.0],
            swap_weights: [52.0, 32.0, 9.0, 6.0, 1.5],
            beast_weights: [25.0, 25.0, 25.0, 25.0],
            double_eat_tier: 0,
            event_chance: 5.0,
            execution_same_tier: false,
            execution_rerolls: false,
            ivory_count: 4,
            ivory_includes_swapped: true,
            prey_rounds: 3,
            scratch_stacks_base: true,
            nest_days: 2,
            nest_tier: 2,
            rewards: true,
            start_tools: 1,
        }
    }
}

impl GameParams {
    pub fn challenge() -> Self {
        Self {
            execution_same_tier: true,
            execution_rerolls: true,
            rewards: true,
            start_tools: 0,
            ..Self::default()
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct UsedToday {
    pub swap: u8,
    pub devour: u8,
    pub hunt: bool,
    pub devour_after_swap: bool,
    pub no_swap: bool,
    pub no_devour: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct SimStats {
    pub opened: u16,
    pub patrols: u16,
    pub devours: u16,
    pub chains: u16,
    pub swaps: u16,
    pub executions: u16,
    pub best: u8,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct DevourOption {
    pub cell: u8,
    pub scratch: bool,
}
