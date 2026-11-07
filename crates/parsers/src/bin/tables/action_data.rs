//! Auto-generated `ActionData` tagged union (`0` and `1000..=1042`) extracted from client.
use crate::bin::affix::FConfigAffix;
use crate::bin::error::{BinError, WireContext};
use crate::bin::tables::substructs::{FConfigAffixI32I32, FConfigI32F64};
use crate::bin::wire::WireCursor;
/// Maximum `ActionData` index offset from `1000` (`0..=42`).
pub const ACTION_DATA_INDEX_MAX: i32 = 42;
/// Payload for `ActionData` tag `1001`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1001 {
    pub f0: Box<[FConfigAffix]>,
}
impl ActionData1001 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = FConfigAffix::read_array(cur)?;
        Ok(Self { f0 })
    }
}
/// Payload for `ActionData` tag `1002`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1002 {
    pub f1: Box<[FConfigI32F64]>,
    pub f2: Box<[i32]>,
    pub f0: i32,
}
impl ActionData1002 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_i32()?;
        let f1 = FConfigI32F64::read_array(cur)?;
        let f2 = cur.read_array_i32()?;
        Ok(Self { f1, f2, f0 })
    }
}
/// Payload for `ActionData` tag `1003`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1003 {
    pub f0: Box<[FConfigI32F64]>,
    pub f1: Box<[i32]>,
    pub f2: Box<[i32]>,
}
impl ActionData1003 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = FConfigI32F64::read_array(cur)?;
        let f1 = cur.read_array_i32()?;
        let f2 = cur.read_array_i32()?;
        Ok(Self { f0, f1, f2 })
    }
}
/// Payload for `ActionData` tag `1004`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1004 {
    pub f0: f64,
}
impl ActionData1004 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_f64()?;
        Ok(Self { f0 })
    }
}
/// Payload for `ActionData` tag `1005`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ActionData1005 {
    pub f0: i32,
    pub f1: i32,
}
impl ActionData1005 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_i32()?;
        let f1 = cur.read_i32()?;
        Ok(Self { f0, f1 })
    }
}
/// Payload for `ActionData` tag `1006`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1006 {
    pub f1: Box<[FConfigI32F64]>,
    pub f0: i32,
}
impl ActionData1006 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_i32()?;
        let f1 = FConfigI32F64::read_array(cur)?;
        Ok(Self { f1, f0 })
    }
}
/// Payload for `ActionData` tag `1007`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1007 {
    pub f0: f64,
    pub f1: f64,
}
impl ActionData1007 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_f64()?;
        let f1 = cur.read_f64()?;
        Ok(Self { f0, f1 })
    }
}
/// Payload for `ActionData` tag `1008`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1008 {
    pub f0: Box<[FConfigI32F64]>,
    pub f1: Box<[FConfigI32F64]>,
    pub f2: Box<[i32]>,
}
impl ActionData1008 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = FConfigI32F64::read_array(cur)?;
        let f1 = FConfigI32F64::read_array(cur)?;
        let f2 = cur.read_array_i32()?;
        Ok(Self { f0, f1, f2 })
    }
}
/// Payload for `ActionData` tag `1009`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1009 {
    pub f0: f64,
    pub f1: f64,
    pub f2: f64,
    pub f3: f64,
    pub f4: f64,
}
impl ActionData1009 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_f64()?;
        let f1 = cur.read_f64()?;
        let f2 = cur.read_f64()?;
        let f3 = cur.read_f64()?;
        let f4 = cur.read_f64()?;
        Ok(Self { f0, f1, f2, f3, f4 })
    }
}
/// Payload for `ActionData` tag `1010`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1010 {
    pub f0: f64,
    pub f1: f64,
    pub f2: f64,
    pub f3: f64,
}
impl ActionData1010 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_f64()?;
        let f1 = cur.read_f64()?;
        let f2 = cur.read_f64()?;
        let f3 = cur.read_f64()?;
        Ok(Self { f0, f1, f2, f3 })
    }
}
/// Payload for `ActionData` tag `1011`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1011 {
    pub f0: f64,
}
impl ActionData1011 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_f64()?;
        Ok(Self { f0 })
    }
}
/// Payload for `ActionData` tag `1012`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1012 {
    pub f0: f64,
}
impl ActionData1012 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_f64()?;
        Ok(Self { f0 })
    }
}
/// Payload for `ActionData` tag `1013`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ActionData1013 {
    pub f0: Box<str>,
}
impl ActionData1013 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_str()?;
        Ok(Self { f0 })
    }
}
/// Payload for `ActionData` tag `1015`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1015 {
    pub f0: f64,
    pub f1: Box<[i32]>,
}
impl ActionData1015 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_f64()?;
        let f1 = cur.read_array_i32()?;
        Ok(Self { f0, f1 })
    }
}
/// Payload for `ActionData` tag `1016`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ActionData1016 {
    pub f2: Box<[i32]>,
    pub f3: Box<[i32]>,
    pub f0: i32,
    pub f1: i32,
}
impl ActionData1016 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_i32()?;
        let f1 = cur.read_i32()?;
        let f2 = cur.read_array_i32()?;
        let f3 = cur.read_array_i32()?;
        Ok(Self { f2, f3, f0, f1 })
    }
}
/// Payload for `ActionData` tag `1017`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ActionData1017 {
    pub f0: Box<[i32]>,
}
impl ActionData1017 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_array_i32()?;
        Ok(Self { f0 })
    }
}
/// Payload for `ActionData` tag `1018`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1018 {
    pub f0: f64,
    pub f1: f64,
    pub f2: f64,
}
impl ActionData1018 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_f64()?;
        let f1 = cur.read_f64()?;
        let f2 = cur.read_f64()?;
        Ok(Self { f0, f1, f2 })
    }
}
/// Payload for `ActionData` tag `1019`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1019 {
    pub f1: f64,
    pub f3: Box<[i32]>,
    pub f0: i32,
    pub f2: i32,
}
impl ActionData1019 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_i32()?;
        let f1 = cur.read_f64()?;
        let f2 = cur.read_i32()?;
        let f3 = cur.read_array_i32()?;
        Ok(Self { f1, f3, f0, f2 })
    }
}
/// Payload for `ActionData` tag `1021`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1021 {
    pub f0: f64,
    pub f1: f64,
}
impl ActionData1021 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_f64()?;
        let f1 = cur.read_f64()?;
        Ok(Self { f0, f1 })
    }
}
/// Payload for `ActionData` tag `1023`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ActionData1023 {
    pub f0: i32,
}
impl ActionData1023 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_i32()?;
        Ok(Self { f0 })
    }
}
/// Payload for `ActionData` tag `1026`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1026 {
    pub f0: Box<[FConfigI32F64]>,
    pub f1: Box<[FConfigAffix]>,
}
impl ActionData1026 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = FConfigI32F64::read_array(cur)?;
        let f1 = FConfigAffix::read_array(cur)?;
        Ok(Self { f0, f1 })
    }
}
/// Payload for `ActionData` tag `1028`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1028 {
    pub f0: Box<[FConfigAffixI32I32]>,
}
impl ActionData1028 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = FConfigAffixI32I32::read_array(cur)?;
        Ok(Self { f0 })
    }
}
/// Payload for `ActionData` tag `1029`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1029 {
    pub f0: Box<[FConfigAffix]>,
}
impl ActionData1029 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = FConfigAffix::read_array(cur)?;
        Ok(Self { f0 })
    }
}
/// Payload for `ActionData` tag `1030`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1030 {
    pub f0: Box<[FConfigAffix]>,
    pub f1: Box<[FConfigAffix]>,
    pub f2: Box<[FConfigAffix]>,
}
impl ActionData1030 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = FConfigAffix::read_array(cur)?;
        let f1 = FConfigAffix::read_array(cur)?;
        let f2 = FConfigAffix::read_array(cur)?;
        Ok(Self { f0, f1, f2 })
    }
}
/// Payload for `ActionData` tag `1032`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ActionData1032 {
    pub f0: i32,
    pub f1: i32,
    pub f2: i32,
}
impl ActionData1032 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_i32()?;
        let f1 = cur.read_i32()?;
        let f2 = cur.read_i32()?;
        Ok(Self { f0, f1, f2 })
    }
}
/// Payload for `ActionData` tag `1034`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1034 {
    pub f0: Box<[FConfigAffix]>,
    pub f1: f64,
    pub f2: f64,
    pub f3: f64,
}
impl ActionData1034 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = FConfigAffix::read_array(cur)?;
        let f1 = cur.read_f64()?;
        let f2 = cur.read_f64()?;
        let f3 = cur.read_f64()?;
        Ok(Self { f0, f1, f2, f3 })
    }
}
/// Payload for `ActionData` tag `1035`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1035 {
    pub f8: Box<[FConfigAffix]>,
    pub f0: i32,
    pub f1: i32,
    pub f2: i32,
    pub f3: i32,
    pub f4: i32,
    pub f5: i32,
    pub f6: i32,
    pub f7: i32,
}
impl ActionData1035 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_i32()?;
        let f1 = cur.read_i32()?;
        let f2 = cur.read_i32()?;
        let f3 = cur.read_i32()?;
        let f4 = cur.read_i32()?;
        let f5 = cur.read_i32()?;
        let f6 = cur.read_i32()?;
        let f7 = cur.read_i32()?;
        let f8 = FConfigAffix::read_array(cur)?;
        Ok(Self {
            f8,
            f0,
            f1,
            f2,
            f3,
            f4,
            f5,
            f6,
            f7,
        })
    }
}
/// Payload for `ActionData` tag `1036`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1036 {
    pub f0: Box<[FConfigAffix]>,
}
impl ActionData1036 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = FConfigAffix::read_array(cur)?;
        Ok(Self { f0 })
    }
}
/// Payload for `ActionData` tag `1038`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1038 {
    pub f0: Box<[FConfigAffix]>,
}
impl ActionData1038 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = FConfigAffix::read_array(cur)?;
        Ok(Self { f0 })
    }
}
/// Payload for `ActionData` tag `1039`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1039 {
    pub f0: Box<[FConfigAffix]>,
}
impl ActionData1039 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = FConfigAffix::read_array(cur)?;
        Ok(Self { f0 })
    }
}
/// Payload for `ActionData` tag `1040`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1040 {
    pub f0: Box<[FConfigAffix]>,
}
impl ActionData1040 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = FConfigAffix::read_array(cur)?;
        Ok(Self { f0 })
    }
}
/// Payload for `ActionData` tag `1041`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1041 {
    pub f1: f64,
    pub f2: f64,
    pub f0: i32,
}
impl ActionData1041 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_i32()?;
        let f1 = cur.read_f64()?;
        let f2 = cur.read_f64()?;
        Ok(Self { f1, f2, f0 })
    }
}
/// Payload for `ActionData` tag `1042`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionData1042 {
    pub f1: f64,
    pub f3: Box<[FConfigAffix]>,
    pub f0: i32,
    pub f2: i32,
}
impl ActionData1042 {
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_i32()?;
        let f1 = cur.read_f64()?;
        let f2 = cur.read_i32()?;
        let f3 = FConfigAffix::read_array(cur)?;
        Ok(Self { f1, f3, f0, f2 })
    }
}
/// Polymorphic gameplay action payload (`tag == 0` or `1000..=1000 + ACTION_DATA_INDEX_MAX`).
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ActionData {
    #[default]
    None,
    Tag1000,
    Tag1001(ActionData1001),
    Tag1002(ActionData1002),
    Tag1003(ActionData1003),
    Tag1004(ActionData1004),
    Tag1005(ActionData1005),
    Tag1006(ActionData1006),
    Tag1007(ActionData1007),
    Tag1008(ActionData1008),
    Tag1009(ActionData1009),
    Tag1010(ActionData1010),
    Tag1011(ActionData1011),
    Tag1012(ActionData1012),
    Tag1013(ActionData1013),
    Tag1014,
    Tag1015(ActionData1015),
    Tag1016(ActionData1016),
    Tag1017(ActionData1017),
    Tag1018(ActionData1018),
    Tag1019(ActionData1019),
    Tag1020,
    Tag1021(ActionData1021),
    Tag1022,
    Tag1023(ActionData1023),
    Tag1024,
    Tag1025,
    Tag1026(ActionData1026),
    Tag1027,
    Tag1028(ActionData1028),
    Tag1029(ActionData1029),
    Tag1030(ActionData1030),
    Tag1031,
    Tag1032(ActionData1032),
    Tag1033,
    Tag1034(ActionData1034),
    Tag1035(ActionData1035),
    Tag1036(ActionData1036),
    Tag1037,
    Tag1038(ActionData1038),
    Tag1039(ActionData1039),
    Tag1040(ActionData1040),
    Tag1041(ActionData1041),
    Tag1042(ActionData1042),
}
#[inline(never)]
fn read_action_chunk_0(
    tag: i32,
    cur: &mut WireCursor<'_>,
) -> Option<Result<ActionData, BinError>> {
    let res = match tag {
        1000 => Ok(ActionData::Tag1000),
        1001 => ActionData1001::read(cur).map(ActionData::Tag1001),
        1002 => ActionData1002::read(cur).map(ActionData::Tag1002),
        1003 => ActionData1003::read(cur).map(ActionData::Tag1003),
        1004 => ActionData1004::read(cur).map(ActionData::Tag1004),
        1005 => ActionData1005::read(cur).map(ActionData::Tag1005),
        1006 => ActionData1006::read(cur).map(ActionData::Tag1006),
        1007 => ActionData1007::read(cur).map(ActionData::Tag1007),
        1008 => ActionData1008::read(cur).map(ActionData::Tag1008),
        1009 => ActionData1009::read(cur).map(ActionData::Tag1009),
        1010 => ActionData1010::read(cur).map(ActionData::Tag1010),
        1011 => ActionData1011::read(cur).map(ActionData::Tag1011),
        1012 => ActionData1012::read(cur).map(ActionData::Tag1012),
        1013 => ActionData1013::read(cur).map(ActionData::Tag1013),
        1014 => Ok(ActionData::Tag1014),
        1015 => ActionData1015::read(cur).map(ActionData::Tag1015),
        _ => return None,
    };
    Some(res)
}
#[inline(never)]
fn read_action_chunk_1(
    tag: i32,
    cur: &mut WireCursor<'_>,
) -> Option<Result<ActionData, BinError>> {
    let res = match tag {
        1016 => ActionData1016::read(cur).map(ActionData::Tag1016),
        1017 => ActionData1017::read(cur).map(ActionData::Tag1017),
        1018 => ActionData1018::read(cur).map(ActionData::Tag1018),
        1019 => ActionData1019::read(cur).map(ActionData::Tag1019),
        1020 => Ok(ActionData::Tag1020),
        1021 => ActionData1021::read(cur).map(ActionData::Tag1021),
        1022 => Ok(ActionData::Tag1022),
        1023 => ActionData1023::read(cur).map(ActionData::Tag1023),
        1024 => Ok(ActionData::Tag1024),
        1025 => Ok(ActionData::Tag1025),
        1026 => ActionData1026::read(cur).map(ActionData::Tag1026),
        1027 => Ok(ActionData::Tag1027),
        1028 => ActionData1028::read(cur).map(ActionData::Tag1028),
        1029 => ActionData1029::read(cur).map(ActionData::Tag1029),
        1030 => ActionData1030::read(cur).map(ActionData::Tag1030),
        1031 => Ok(ActionData::Tag1031),
        _ => return None,
    };
    Some(res)
}
#[inline(never)]
fn read_action_chunk_2(
    tag: i32,
    cur: &mut WireCursor<'_>,
) -> Option<Result<ActionData, BinError>> {
    let res = match tag {
        1032 => ActionData1032::read(cur).map(ActionData::Tag1032),
        1033 => Ok(ActionData::Tag1033),
        1034 => ActionData1034::read(cur).map(ActionData::Tag1034),
        1035 => ActionData1035::read(cur).map(ActionData::Tag1035),
        1036 => ActionData1036::read(cur).map(ActionData::Tag1036),
        1037 => Ok(ActionData::Tag1037),
        1038 => ActionData1038::read(cur).map(ActionData::Tag1038),
        1039 => ActionData1039::read(cur).map(ActionData::Tag1039),
        1040 => ActionData1040::read(cur).map(ActionData::Tag1040),
        1041 => ActionData1041::read(cur).map(ActionData::Tag1041),
        1042 => ActionData1042::read(cur).map(ActionData::Tag1042),
        _ => return None,
    };
    Some(res)
}
impl ActionData {
    /// Decodes a tagged `ActionData` union from `cur`.
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let tag = cur.read_i32_ctx(WireContext::ActionDataPayload)?;
        if tag == 0 {
            return Ok(Self::None);
        }
        if let Some(res) = read_action_chunk_0(tag, cur) {
            return res;
        }
        if let Some(res) = read_action_chunk_1(tag, cur) {
            return res;
        }
        if let Some(res) = read_action_chunk_2(tag, cur) {
            return res;
        }
        Err(BinError::InvalidActionDataTag {
            raw_tag: tag,
        })
    }
}
