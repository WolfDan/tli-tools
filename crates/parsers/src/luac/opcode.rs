//! Instruction bitfield layout and `\x01XDI` <-> PUC-Lua 5.3 opcode bijection.

use crate::u32_to_usize;

/// Standard 64-bit PUC-Lua 5.3 VM opcodes (`0..=46`).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LuaOpcode {
    Move = 0,
    LoadK = 1,
    LoadKx = 2,
    LoadBool = 3,
    LoadNil = 4,
    GetUpval = 5,
    GetTabUp = 6,
    GetTable = 7,
    SetTabUp = 8,
    SetUpval = 9,
    SetTable = 10,
    NewTable = 11,
    SelfOp = 12,
    Add = 13,
    Sub = 14,
    Mul = 15,
    Mod = 16,
    Pow = 17,
    Div = 18,
    IDiv = 19,
    BAnd = 20,
    BOr = 21,
    BXor = 22,
    Shl = 23,
    Shr = 24,
    Unm = 25,
    BNot = 26,
    Not = 27,
    Len = 28,
    Concat = 29,
    Jmp = 30,
    Eq = 31,
    Lt = 32,
    Le = 33,
    Test = 34,
    TestSet = 35,
    Call = 36,
    TailCall = 37,
    Return = 38,
    ForLoop = 39,
    ForPrep = 40,
    TForCall = 41,
    TForLoop = 42,
    SetList = 43,
    Closure = 44,
    VarArg = 45,
    ExtraArg = 46,
}

const STD_OPCODES: [LuaOpcode; 47] = [
    LuaOpcode::Move,
    LuaOpcode::LoadK,
    LuaOpcode::LoadKx,
    LuaOpcode::LoadBool,
    LuaOpcode::LoadNil,
    LuaOpcode::GetUpval,
    LuaOpcode::GetTabUp,
    LuaOpcode::GetTable,
    LuaOpcode::SetTabUp,
    LuaOpcode::SetUpval,
    LuaOpcode::SetTable,
    LuaOpcode::NewTable,
    LuaOpcode::SelfOp,
    LuaOpcode::Add,
    LuaOpcode::Sub,
    LuaOpcode::Mul,
    LuaOpcode::Mod,
    LuaOpcode::Pow,
    LuaOpcode::Div,
    LuaOpcode::IDiv,
    LuaOpcode::BAnd,
    LuaOpcode::BOr,
    LuaOpcode::BXor,
    LuaOpcode::Shl,
    LuaOpcode::Shr,
    LuaOpcode::Unm,
    LuaOpcode::BNot,
    LuaOpcode::Not,
    LuaOpcode::Len,
    LuaOpcode::Concat,
    LuaOpcode::Jmp,
    LuaOpcode::Eq,
    LuaOpcode::Lt,
    LuaOpcode::Le,
    LuaOpcode::Test,
    LuaOpcode::TestSet,
    LuaOpcode::Call,
    LuaOpcode::TailCall,
    LuaOpcode::Return,
    LuaOpcode::ForLoop,
    LuaOpcode::ForPrep,
    LuaOpcode::TForCall,
    LuaOpcode::TForLoop,
    LuaOpcode::SetList,
    LuaOpcode::Closure,
    LuaOpcode::VarArg,
    LuaOpcode::ExtraArg,
];

/// Bijection mapping Xindong `\x01XDI` opcode (`0..=46`) to standard PUC-Lua 5.3 `LuaOpcode`.
pub(crate) const XDI_TO_LUA53_OPCODE: [LuaOpcode; 47] = [
    LuaOpcode::LoadNil,  // 0x00 -> 4
    LuaOpcode::Mul,      // 0x01 -> 15
    LuaOpcode::SetTable, // 0x02 -> 10
    LuaOpcode::Le,       // 0x03 -> 33
    LuaOpcode::NewTable, // 0x04 -> 11
    LuaOpcode::LoadKx,   // 0x05 -> 2
    LuaOpcode::LoadK,    // 0x06 -> 1
    LuaOpcode::Pow,      // 0x07 -> 17
    LuaOpcode::GetTable, // 0x08 -> 7
    LuaOpcode::TestSet,  // 0x09 -> 35
    LuaOpcode::Lt,       // 0x0A -> 32
    LuaOpcode::Eq,       // 0x0B -> 31
    LuaOpcode::Move,     // 0x0C -> 0
    LuaOpcode::VarArg,   // 0x0D -> 45
    LuaOpcode::GetUpval, // 0x0E -> 5
    LuaOpcode::LoadBool, // 0x0F -> 3
    LuaOpcode::IDiv,     // 0x10 -> 19
    LuaOpcode::Div,      // 0x11 -> 18
    LuaOpcode::GetTabUp, // 0x12 -> 6
    LuaOpcode::SetTabUp, // 0x13 -> 8
    LuaOpcode::BOr,      // 0x14 -> 21
    LuaOpcode::Return,   // 0x15 -> 38
    LuaOpcode::Add,      // 0x16 -> 13
    LuaOpcode::Concat,   // 0x17 -> 29
    LuaOpcode::Call,     // 0x18 -> 36
    LuaOpcode::TForLoop, // 0x19 -> 42
    LuaOpcode::Closure,  // 0x1A -> 44
    LuaOpcode::ForPrep,  // 0x1B -> 40
    LuaOpcode::SetList,  // 0x1C -> 43
    LuaOpcode::TailCall, // 0x1D -> 37
    LuaOpcode::ForLoop,  // 0x1E -> 39
    LuaOpcode::BAnd,     // 0x1F -> 20
    LuaOpcode::SetUpval, // 0x20 -> 9
    LuaOpcode::BXor,     // 0x21 -> 22
    LuaOpcode::Jmp,      // 0x22 -> 30
    LuaOpcode::Mod,      // 0x23 -> 16
    LuaOpcode::Not,      // 0x24 -> 27
    LuaOpcode::SelfOp,   // 0x25 -> 12
    LuaOpcode::BNot,     // 0x26 -> 26
    LuaOpcode::Unm,      // 0x27 -> 25
    LuaOpcode::Shr,      // 0x28 -> 24
    LuaOpcode::TForCall, // 0x29 -> 41
    LuaOpcode::Test,     // 0x2A -> 34
    LuaOpcode::Len,      // 0x2B -> 28
    LuaOpcode::Sub,      // 0x2C -> 14
    LuaOpcode::Shl,      // 0x2D -> 23
    LuaOpcode::ExtraArg, // 0x2E -> 46
];

impl LuaOpcode {
    #[inline]
    #[must_use]
    pub(crate) const fn as_u32(self) -> u32 {
        self as u32
    }
}

// Compile-time proof that `STD_OPCODES` and `XDI_TO_LUA53_OPCODE` form a bijection on `0..=46`.
const _: () = {
    let mut seen = [false; 47];
    let mut i = 0;
    while i < 47 {
        assert!(u32_to_usize(STD_OPCODES[i].as_u32()) == i);
        let std_idx = u32_to_usize(XDI_TO_LUA53_OPCODE[i].as_u32());
        assert!(!seen[std_idx]);
        seen[std_idx] = true;
        i += 1;
    }
};

/// Canonical 32-bit PUC-Lua 5.3 instruction word (`OP` in bits `0..5`).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LuaInstruction(u32);
const _: () = assert!(core::mem::size_of::<LuaInstruction>() == 4);

impl LuaInstruction {
    /// Transpiles a 32-bit `\x01XDI` instruction word into a canonical Lua 5.3 instruction.
    #[inline]
    pub(crate) const fn from_xdi_word(w: u32) -> Result<Self, u8> {
        let raw_op_u32 = (w >> 26) & 0x3F;
        if raw_op_u32 >= 47 {
            return Err(raw_op_u32 as u8);
        }
        let std_op = XDI_TO_LUA53_OPCODE[u32_to_usize(raw_op_u32)].as_u32();
        let std_w = ((w & 0x03FF_FFFF) << 6) | std_op;
        Ok(Self(std_w))
    }

    /// Validates a 32-bit canonical Lua 5.3 instruction word.
    #[inline]
    pub(crate) const fn from_lua53_word(w: u32) -> Result<Self, u8> {
        let raw_op = (w & 0x3F) as u8;
        if raw_op >= 47 {
            return Err(raw_op);
        }
        Ok(Self(w))
    }

    #[inline]
    #[must_use]
    pub(crate) const fn opcode(self) -> LuaOpcode {
        let idx = self.0 & 0x3F;
        assert!(idx < 47);
        STD_OPCODES[u32_to_usize(idx)]
    }

    #[inline]
    #[must_use]
    pub(crate) const fn a(self) -> u8 {
        ((self.0 >> 6) & 0xFF) as u8
    }

    #[inline]
    #[must_use]
    pub(crate) const fn c(self) -> u16 {
        ((self.0 >> 14) & 0x1FF) as u16
    }

    #[inline]
    #[must_use]
    pub(crate) const fn b(self) -> u16 {
        ((self.0 >> 23) & 0x1FF) as u16
    }

    #[inline]
    #[must_use]
    pub(crate) const fn bx(self) -> u32 {
        (self.0 >> 14) & 0x3FFFF
    }

    #[inline]
    #[must_use]
    pub(crate) const fn ax(self) -> u32 {
        (self.0 >> 6) & 0x03FF_FFFF
    }
}
