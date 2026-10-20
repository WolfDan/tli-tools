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

    pub fn from_str(s: &str) -> Option<Self> {
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
    "01", "02", "03", "04", "05", "06", "07", "08",
    "09", "10", "11", "12", "13", "14", "15", "16",
];

#[inline]
pub const fn cell_label(n: u8) -> &'static str {
    CELL_LABELS[(n as usize) & 15]
}

const INMATE_DESCRIPTIONS: [[&str; 4]; 6] = [
    ["Normal Mammoth", "Normal Lion", "Normal Wolf", "Normal Rat"],
    ["Magic Mammoth", "Magic Lion", "Magic Wolf", "Magic Rat"],
    ["Rare Mammoth", "Rare Lion", "Rare Wolf", "Rare Rat"],
    ["Legendary Mammoth", "Legendary Lion", "Legendary Wolf", "Legendary Rat"],
    ["Supreme Mammoth", "Supreme Lion", "Supreme Wolf", "Supreme Rat"],
    ["Rainbow Mammoth", "Rainbow Lion", "Rainbow Wolf", "Rainbow Rat"],
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

    pub fn from_str(s: &str) -> Option<Self> {
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

    pub fn from_str(s: &str) -> Option<Self> {
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

    pub fn from_str(s: &str) -> Option<Self> {
        if let Some(t) = ToolId::from_str(s) {
            Some(ChoiceId::Tool(t))
        } else {
            GiftId::from_str(s).map(ChoiceId::Gift)
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

    pub fn from_str(s: &str) -> Option<Self> {
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

    pub fn from_str(s: &str) -> Option<Self> {
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

    pub fn from_str(s: &str) -> Option<Self> {
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

    pub fn from_str(s: &str) -> Option<Self> {
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

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SimState {
    pub mode: GameMode,
    pub params: GameParams,
    pub rng: u32,
    pub day: u8,
    pub max_days: u8,
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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct DevourOption {
    pub cell: u8,
    pub scratch: bool,
}

#[inline]
pub const fn hash_seed(text: &str) -> u32 {
    let bytes = text.as_bytes();
    let mut h: u32 = 2166136261;
    let mut i = 0;
    while i < bytes.len() {
        h ^= bytes[i] as u32;
        h = h.wrapping_mul(16777619);
        i += 1;
    }
    h
}

#[inline]
pub const fn refit_seed(slot: u8) -> u32 {
    let base = hash_seed("ANALYZER|refit|");
    let d = (b'0' + (slot % 10)) as u32;
    let h1 = (base ^ d).wrapping_mul(16777619);
    let h2 = (h1 ^ (b'|' as u32)).wrapping_mul(16777619);
    (h2 ^ (b'0' as u32)).wrapping_mul(16777619)
}

#[inline]
pub fn next_rand(rng: &mut u32) -> f64 {
    *rng = rng.wrapping_add(0x6d2b79f5);
    let mut t = *rng;
    t = (t ^ (t >> 15)).wrapping_mul(t | 1);
    t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
    (t ^ (t >> 14)) as f64 / 4294967296.0
}

#[inline]
pub fn pick_index(rng: &mut u32, weights: &[f64]) -> usize {
    let total: f64 = weights.iter().map(|w| w.max(0.0)).sum();
    if total <= 0.0 {
        return 0;
    }
    let mut r = next_rand(rng) * total;
    for (i, w) in weights.iter().enumerate() {
        r -= w.max(0.0);
        if r < 0.0 {
            return i;
        }
    }
    weights.len().saturating_sub(1)
}

#[inline]
pub fn pick_one<T: Copy>(rng: &mut u32, list: &[T]) -> T {
    let idx = (next_rand(rng) * (list.len() as f64)).floor() as usize;
    list[idx.min(list.len() - 1)]
}

#[inline]
pub fn roll_event_from_pool(rng: &mut u32, pool: &[EventKind]) -> EventKind {
    if pool.is_empty() {
        return EventKind::Chain;
    }
    let mut total_w = 0u32;
    for &ev in pool {
        total_w += ev.weight();
    }
    if total_w == 0 {
        return pick_one(rng, pool);
    }
    let mut r = (next_rand(rng) * (total_w as f64)).floor() as u32;
    for &ev in pool {
        let w = ev.weight();
        if r < w {
            return ev;
        }
        r -= w;
    }
    pool[pool.len() - 1]
}

#[inline]
pub fn shuffle_slice<T: Copy>(rng: &mut u32, slice: &mut [T]) {
    let len = slice.len();
    if len <= 1 {
        return;
    }
    for i in (1..len).rev() {
        let j = (next_rand(rng) * ((i + 1) as f64)).floor() as usize;
        slice.swap(i, j);
    }
}

#[inline]
pub const fn steps_between(a: u8, b: u8) -> u8 {
    (b + 16 - a) % 16
}

impl SimState {
    pub fn new_blank(mode: GameMode, rewards: bool, max_days: u8) -> Self {
        let mut params = match mode {
            GameMode::Challenge => GameParams::challenge(),
            GameMode::Drill => {
                let mut p = GameParams::default();
                p.rewards = rewards;
                p.drill_days = max_days.clamp(5, 60);
                p
            }
        };
        let max_d = match mode {
            GameMode::Challenge => CHALLENGE_DAYS,
            GameMode::Drill => params.drill_days,
        };
        params.drill_days = max_d;

        let mut cells = [CellState::default(); CELL_COUNT];
        for n in 0..CELL_COUNT {
            cells[n] = CellState {
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

        let mut event_pool = [EventKind::Chain; 17];
        for (i, &ev) in INITIAL_EVENT_POOL.iter().enumerate() {
            event_pool[i] = ev;
        }

        Self {
            mode,
            params,
            rng: hash_seed("ANALYZER"),
            day: 1,
            max_days: max_d,
            phase: Phase::Manage,
            position: OFFICE,
            next_id: 1,
            dest_base: 3,
            destinations: [0; 16],
            dest_len: 0,
            cells,
            tools_mask: ToolMask(0),
            tools_order: [ToolId::Revenge; 16],
            tools_len: 0,
            vault: [VaultEntry::default(); 32],
            vault_len: 0,
            cache: 0,
            prey: [PreyLink::default(); 8],
            prey_len: 0,
            event_pool,
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
            challenge: if mode == GameMode::Challenge {
                Some(ChallengeState {
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
                })
            } else {
                None
            },
        }
    }

    pub fn new_drill_run(seed: u32, tools: &[ToolId]) -> Self {
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
        st
    }

    #[inline]
    pub const fn has_tool(&self, tool: ToolId) -> bool {
        self.tools_mask.has(tool)
    }

    #[inline]
    pub fn add_tool(&mut self, tool: ToolId) {
        if !self.has_tool(tool) {
            self.tools_mask.insert(tool);
            if (self.tools_len as usize) < self.tools_order.len() {
                self.tools_order[self.tools_len as usize] = tool;
                self.tools_len += 1;
            }
        }
    }

    #[inline]
    pub fn tools_slice(&self) -> &[ToolId] {
        &self.tools_order[..self.tools_len as usize]
    }

    #[inline]
    pub fn destinations_slice(&self) -> &[u8] {
        &self.destinations[..self.dest_len as usize]
    }

    #[inline]
    pub fn event_pool_slice(&self) -> &[EventKind] {
        &self.event_pool[..self.event_pool_len as usize]
    }

    #[inline]
    pub fn prey_slice(&self) -> &[PreyLink] {
        &self.prey[..self.prey_len as usize]
    }

    #[inline]
    pub fn vault_slice(&self) -> &[VaultEntry] {
        &self.vault[..self.vault_len as usize]
    }

    #[inline]
    pub const fn cage_fight(&self) -> bool {
        matches!(self.start_event, StartEventId::CageFight)
    }

    #[inline]
    pub const fn can_stop_chain(&self) -> bool {
        self.params.chain_stop
    }

    #[inline]
    pub const fn can_devour_now(&self) -> bool {
        self.used.devour < 1
            && !self.used.hunt
            && !self.used.no_devour
            && (self.used.swap < 1 || self.used.devour_after_swap)
    }

    #[inline]
    pub const fn can_swap_now(&self) -> bool {
        self.used.devour < 1
            && !self.used.hunt
            && !self.used.no_swap
            && self.used.swap < self.params.swaps_per_day
    }

    #[inline]
    pub const fn can_reroll(&self) -> bool {
        if self.pending_len == 0 {
            return false;
        }
        let first = &self.pending[0];
        matches!(first.kind, PendingKind::Tool) && first.rerolls > 0
    }

    pub fn push_pending(&mut self, p: PendingChoice) {
        if (self.pending_len as usize) < self.pending.len() {
            self.pending[self.pending_len as usize] = p;
            self.pending_len += 1;
        }
    }

    pub fn pop_pending_front(&mut self) -> Option<PendingChoice> {
        if self.pending_len == 0 {
            return None;
        }
        let first = self.pending[0];
        for i in 1..(self.pending_len as usize) {
            self.pending[i - 1] = self.pending[i];
        }
        self.pending_len -= 1;
        Some(first)
    }

    #[inline]
    pub fn eats_of(&self, beast: Beast, tier: Option<u8>) -> ([Beast; 2], usize) {
        let first = beast.primary_prey();
        let from = if self.cage_fight() {
            0
        } else {
            self.params.double_eat_tier
        };
        if tier.unwrap_or(0) >= from {
            if let Some(sec) = beast.second_meal() {
                return ([first, sec], 2);
            }
        }
        ([first, first], 1)
    }

    #[inline]
    pub fn can_eat(&self, eater: Inmate, meal: Inmate) -> bool {
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
                if let Some(meal) = c.inmate {
                    if self.can_eat(eater, meal) {
                        out[len] = DevourOption {
                            cell: c.n,
                            scratch: false,
                        };
                        len += 1;
                    }
                }
            } else if c.scratch && eater.beast == Beast::Wolf && eater.tier < RAINBOW {
                if let Some(hid) = c.hidden {
                    if hid.tier == eater.tier {
                        out[len] = DevourOption {
                            cell: c.n,
                            scratch: true,
                        };
                        len += 1;
                    }
                }
            }
        }
        (out, len)
    }

    pub fn start_options(&self) -> ([DevourOption; 16], usize) {
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
            if let Some(meal) = c.inmate {
                if self.can_eat(lion, meal) && self.link_of(meal.id).is_none() {
                    out[len] = c.n;
                    len += 1;
                }
            }
        }
        (out, len)
    }

    pub fn can_hunt_now(&self) -> bool {
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
        for c in &self.cells {
            if let Some(i) = c.inmate {
                if i.id == inmate_id {
                    return Some(c.n);
                }
            }
        }
        None
    }

    #[inline]
    pub fn board_cell_of(&self, inmate_id: u16) -> Option<u8> {
        for c in &self.cells {
            if !c.sealed {
                if let Some(i) = c.inmate {
                    if i.id == inmate_id {
                        return Some(c.n);
                    }
                }
            }
        }
        None
    }

    #[inline]
    pub fn link_of(&self, inmate_id: u16) -> Option<PreyLink> {
        self.prey_slice()
            .iter()
            .copied()
            .find(|l| l.lion == inmate_id || l.prey == inmate_id)
    }

    pub fn check_prey(&mut self) {
        let mut write = 0;
        for read in 0..(self.prey_len as usize) {
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
        self.prey_len = write as u8;
    }

    #[inline]
    pub fn door_weights(&self) -> [f64; 5] {
        let mut w = self.params.tier_weights;
        if self.has_tool(ToolId::Hook) && w[1..].iter().any(|&x| x > 0.0) {
            w[0] = 0.0;
        }
        w
    }

    #[inline]
    pub fn cell_weights(&self, cell: &CellState) -> [f64; 5] {
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
        if any { kept } else { w }
    }

    #[inline]
    pub fn make_inmate(&mut self, beast: Beast, tier: u8) -> Inmate {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        Inmate {
            id,
            beast,
            tier: tier.min(RAINBOW),
        }
    }

    #[inline]
    pub fn roll_inmate(&mut self, weights: &[f64; 5]) -> Inmate {
        let tier = pick_index(&mut self.rng, weights) as u8;
        let beast_idx = pick_index(&mut self.rng, &self.params.beast_weights);
        let beast = BEASTS[beast_idx];
        self.make_inmate(beast, tier)
    }

    pub fn hidden_of(&mut self, cell_idx: usize) -> Inmate {
        if let Some(h) = self.cells[cell_idx].hidden {
            return h;
        }
        let w = self.cell_weights(&self.cells[cell_idx]);
        let h = self.roll_inmate(&w);
        self.cells[cell_idx].hidden = Some(h);
        h
    }

    pub fn door_inmate(&mut self, cell_idx: usize) -> Inmate {
        let c = &self.cells[cell_idx];
        if let Some(h) = c.hidden {
            if h.tier > 0 || c.known > 0 || !self.has_tool(ToolId::Hook) {
                return h;
            }
        }
        let w = self.cell_weights(&self.cells[cell_idx]);
        self.roll_inmate(&w)
    }

    pub fn roll_event(&mut self) -> EventKind {
        let len = self.event_pool_len as usize;
        roll_event_from_pool(&mut self.rng, &self.event_pool[..len])
    }

    pub fn empty_cell(&mut self, cell_idx: usize) {
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
    }

    #[inline]
    pub fn sealed_cells(&self) -> ([usize; 16], usize) {
        let mut out = [0usize; 16];
        let mut len = 0;
        for c in &self.cells {
            if c.sealed {
                out[len] = c.n as usize;
                len += 1;
            }
        }
        (out, len)
    }

    #[inline]
    pub fn promotable_inmates(&self) -> ([u16; 16], usize) {
        let mut pool = [0u16; 16];
        let mut len = 0;
        for c in &self.cells {
            if !c.sealed {
                if let Some(i) = c.inmate {
                    if i.tier < RAINBOW {
                        pool[len] = i.id;
                        len += 1;
                    }
                }
            }
        }
        (pool, len)
    }

    #[inline]
    pub fn unheld_tools(&self, exclude: ToolMask) -> ([ToolId; 31], usize) {
        let mut pool = [ToolId::Revenge; 31];
        let mut len = 0;
        let blocked = self.tools_mask.0 | exclude.0;
        for t in ALL_TOOLS {
            if (blocked & (1u32 << (t as u32))) == 0 {
                pool[len] = t;
                len += 1;
            }
        }
        (pool, len)
    }

    pub fn meet_objective(&mut self) {
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
    }

    pub fn count_objective(&mut self, id: ObjectiveId) {
        if self.objective_id != id || self.milestone {
            return;
        }
        let need = id.need();
        if need == 0 {
            return;
        }
        self.objective_progress = (self.objective_progress + 1).min(need);
        if self.objective_progress >= need {
            self.meet_objective();
        }
    }

    pub fn after_growth(&mut self, inmate_id: u16) {
        let Some(cn) = self.cell_of(inmate_id) else {
            return;
        };
        let Some(inmate) = self.cells[cn as usize].inmate else {
            return;
        };
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
        let Some(cn) = self.cell_of(inmate_id) else {
            return;
        };
        let Some(mut inmate) = self.cells[cn as usize].inmate else {
            return;
        };
        inmate.tier = (inmate.tier + by).min(RAINBOW);
        if reroll_beast && inmate.tier < RAINBOW {
            let b_idx = pick_index(&mut self.rng, &self.params.beast_weights);
            inmate.beast = BEASTS[b_idx];
        }
        self.cells[cn as usize].inmate = Some(inmate);
        self.after_growth(inmate_id);
    }

    pub fn open_cell(
        &mut self,
        cell_idx: usize,
        is_patrol: bool,
        forced: Option<(Beast, u8)>,
    ) -> Option<EventKind> {
        if !self.cells[cell_idx].sealed {
            return None;
        }
        let event = self.cells[cell_idx].event;
        let inmate = if let Some((b, t)) = forced {
            self.make_inmate(b, t)
        } else {
            self.door_inmate(cell_idx)
        };
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
            if self.has_tool(ToolId::Dogtag)
                && self.cells[cell_idx].inmate.is_some_and(|i| i.id == inmate.id && i.tier < RAINBOW)
                && next_rand(&mut self.rng) < 0.25
            {
                self.promote(inmate.id, 1, false);
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
                && self.cells[cell_idx].inmate.is_some_and(|i| i.id == inmate.id)
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
        }
        if is_patrol { event } else { None }
    }

    #[inline]
    pub fn destination_limit(&self) -> usize {
        let base = if self.dest_base > 0 {
            self.dest_base as i32
        } else {
            self.params.destinations as i32
        };
        let penalty = (self.has_tool(ToolId::Forged) as i32)
            + (self.has_tool(ToolId::Whip) as i32)
            + (self.has_tool(ToolId::Stake) as i32);
        (base - penalty).max(1) as usize
    }

    pub fn fit_destinations(&mut self) {
        if self.phase == Phase::Patrol {
            let lim = self.destination_limit();
            if (self.dest_len as usize) > lim {
                self.dest_len = lim as u8;
            }
        }
    }

    pub fn roll_destinations(&mut self) {
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
    }

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
                .find(|& idx| self.prey[idx].lion == link.lion && self.prey[idx].prey == link.prey)
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
            if let Some(due) = self.cells[n].nest_due {
                if due <= self.day {
                    self.open_cell(n, false, Some((Beast::Rat, self.params.nest_tier)));
                }
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
                if !c.sealed {
                    if let Some(i) = c.inmate {
                        board[blen] = c.n as usize;
                        blen += 1;
                        if i.tier > top {
                            top = i.tier;
                        }
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
                if !c.sealed {
                    if let Some(i) = c.inmate {
                        if (i.tier as usize) < 6 {
                            counts[i.beast.idx()][i.tier as usize] += 1;
                        }
                        board[blen] = c.n as usize;
                        blen += 1;
                    }
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

        if let Some(ev) = event {
            if self.cells[to as usize].inmate.is_some() {
                self.phase = Phase::Event;
                self.active_event = Some((ev, to));
                self.count_objective(ObjectiveId::Incident);
                return true;
            }
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
                    if !c.sealed {
                        if let Some(i) = c.inmate {
                            counts[i.beast.idx()] += 1;
                        }
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
                if let Some(nxt) = next_ev {
                    if !self.event_pool_slice().contains(&nxt)
                        && (self.event_pool_len as usize) < self.event_pool.len()
                    {
                        self.event_pool[self.event_pool_len as usize] = nxt;
                        self.event_pool_len += 1;
                    }
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
                    if !c.sealed {
                        if let Some(i) = c.inmate {
                            if i.tier < low_tier {
                                low_tier = i.tier;
                            }
                        }
                    }
                }
                let mut lowest = [0u16; 16];
                let mut llen = 0;
                for c in &self.cells {
                    if !c.sealed {
                        if let Some(i) = c.inmate {
                            if i.tier == low_tier {
                                lowest[llen] = i.id;
                                llen += 1;
                            }
                        }
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
                    if c.n != n && !c.sealed {
                        if let Some(i) = c.inmate {
                            if i.beast == cur.beast {
                                same += 1;
                            }
                        }
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
        if !option.scratch && eater.beast == Beast::Mammoth && self.has_tool(ToolId::Maul) {
            if let Some(m) = meal {
                gain = (m.tier as i32 + 2 - eater.tier as i32).max(1) as u8;
            }
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

        if revenge {
            if let Some(ecn) = self.board_cell_of(eater.id) {
                let cur_eater = self.cells[ecn as usize].inmate.unwrap();
                let (sealed, slen) = self.sealed_cells();
                if slen > 0 {
                    let target = pick_one(&mut self.rng, &sealed[..slen]);
                    let (foods, flen) = self.eats_of(cur_eater.beast, Some(cur_eater.tier));
                    let food = pick_one(&mut self.rng, &foods[..flen]);
                    self.open_cell(target, false, Some((food, cur_eater.tier)));
                }
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
                if !c.sealed {
                    if let Some(i) = c.inmate {
                        if i.id != fresh.id {
                            if i.tier > fresh.tier {
                                above_cnt += 1;
                            } else if i.tier < fresh.tier {
                                below_cnt += 1;
                            }
                        }
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
                if !c.sealed {
                    if let Some(i) = c.inmate {
                        if self.params.ivory_includes_swapped || i.id != fresh.id {
                            pool[plen] = i.id;
                            plen += 1;
                        }
                    }
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
                    if !c.sealed {
                        if let Some(i) = c.inmate {
                            let (_, olen) = self.devour_options(c.n);
                            if olen > 0 {
                                ids[ilen] = i.id;
                                ilen += 1;
                            }
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
                    if !c.sealed {
                        if let Some(i) = c.inmate {
                            if i.tier < 3 {
                                low[llen] = i.id;
                                llen += 1;
                            }
                        }
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
                    if !c.sealed {
                        if let Some(i) = c.inmate {
                            let (_, olen) = self.devour_options(c.n);
                            if olen == 0 {
                                hungry[hlen] = i.id;
                                hlen += 1;
                            }
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
                        let has_meal = self.cells.iter().any(|c| {
                            !c.sealed && c.inmate.is_some_and(|o| self.can_eat(cand, o))
                        });
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
        if self.phase == Phase::Event {
            if let Some((_, ecn)) = self.active_event {
                if self.cells[ecn as usize].inmate.is_none() {
                    self.active_event = None;
                    self.phase = Phase::Manage;
                }
            }
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
        if let Some(contract) = self.current_contract() {
            if contract.deadline == self.day {
                if let Some(ch) = self.challenge.as_mut() {
                    ch.window_open = true;
                    ch.window_early = false;
                }
                return true;
            }
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
