//! Unit and integration tests for `parsers::luac`.

mod boundaries;
mod negative;
mod positive;

use crate::luac::opcode::{LuaOpcode, XDI_TO_LUA53_OPCODE};
pub(crate) use crate::luac::tables;
use crate::luac::types::XDI_MAGIC;

pub(crate) fn std_to_xdi_op(op: LuaOpcode) -> u8 {
    let pos = XDI_TO_LUA53_OPCODE
        .iter()
        .position(|&candidate| candidate == op)
        .expect("all opcodes present in bijection");
    u8::try_from(pos).expect("opcode index fits in u8")
}

pub(crate) fn encode_xdi_abc(op: LuaOpcode, a: u8, b: u16, c: u16) -> u32 {
    let xdi_op = u32::from(std_to_xdi_op(op));
    (xdi_op << 26) | (u32::from(b & 0x1FF) << 17) | (u32::from(c & 0x1FF) << 8) | u32::from(a)
}

pub(crate) fn encode_xdi_abx(op: LuaOpcode, a: u8, bx: u32) -> u32 {
    let xdi_op = u32::from(std_to_xdi_op(op));
    (xdi_op << 26) | ((bx & 0x3FFFF) << 8) | u32::from(a)
}

pub(crate) fn encode_xdi_ax(op: LuaOpcode, ax: u32) -> u32 {
    let xdi_op = u32::from(std_to_xdi_op(op));
    (xdi_op << 26) | (ax & 0x03FF_FFFF)
}

pub(crate) fn write_xdi_string(out: &mut Vec<u8>, s: Option<&[u8]>) {
    let Some(bytes) = s else {
        out.push(0x00);
        return;
    };
    let len = bytes.len();
    if len + 10 < 0xFF {
        out.push(u8::try_from(len + 10).expect("short string tag fits in u8"));
    } else {
        out.push(0xFF);
        let raw = u32::try_from(len + 10).expect("string len fits in u32");
        out.extend_from_slice(&raw.to_le_bytes());
    }
    out.extend_from_slice(bytes);
}

#[derive(Clone)]
pub(crate) enum TestConst {
    Float(f64),
    Int(i64),
    Str(Vec<u8>),
}

pub(crate) fn write_xdi_const(out: &mut Vec<u8>, c: &TestConst) {
    match c {
        TestConst::Float(f) => {
            out.push(0x03);
            out.extend_from_slice(&f.to_le_bytes());
        }
        TestConst::Int(i) => {
            out.push(0x13);
            out.extend_from_slice(&i.to_le_bytes());
        }
        TestConst::Str(s) => {
            let tag = if s.len() > 40 { 0x14 } else { 0x04 };
            out.push(tag);
            write_xdi_string(out, Some(s));
        }
    }
}

pub(crate) struct TestProto {
    pub(crate) source: Option<Vec<u8>>,
    pub(crate) num_params: u8,
    pub(crate) is_vararg: u8,
    pub(crate) max_stack_size: u8,
    pub(crate) code: Vec<u32>,
    pub(crate) consts: Vec<TestConst>,
    pub(crate) upvalues: Vec<(u8, u8)>,
    pub(crate) children: Vec<TestProto>,
    pub(crate) line_info: Vec<i32>,
    pub(crate) loc_vars: Vec<(Vec<u8>, u32, u32)>,
    pub(crate) upvalue_names: Vec<Vec<u8>>,
}

impl TestProto {
    pub(crate) fn new() -> Self {
        Self {
            source: Some(b"@test.lua".to_vec()),
            num_params: 0,
            is_vararg: 2,
            max_stack_size: 8,
            code: Vec::new(),
            consts: Vec::new(),
            upvalues: vec![(1, 0)],
            children: Vec::new(),
            line_info: Vec::new(),
            loc_vars: Vec::new(),
            upvalue_names: Vec::new(),
        }
    }
}

fn write_xdi_proto_head(out: &mut Vec<u8>, proto: &TestProto) {
    write_xdi_string(out, proto.source.as_deref());
    out.extend_from_slice(&0_u32.to_le_bytes()); // line_defined
    out.extend_from_slice(&0_u32.to_le_bytes()); // last_line_defined
    out.push(proto.num_params);
    out.push(proto.is_vararg);
    out.push(proto.max_stack_size);

    let size_code = u32::try_from(proto.code.len()).unwrap();
    out.extend_from_slice(&size_code.to_le_bytes());
    for &w in &proto.code {
        out.extend_from_slice(&w.to_le_bytes());
    }

    let size_k = u32::try_from(proto.consts.len()).unwrap();
    out.extend_from_slice(&size_k.to_le_bytes());
    for c in &proto.consts {
        write_xdi_const(out, c);
    }

    let size_up = u32::try_from(proto.upvalues.len()).unwrap();
    out.extend_from_slice(&size_up.to_le_bytes());
    for &(in_stack, idx) in &proto.upvalues {
        out.push(in_stack);
        out.push(idx);
    }

    let size_p = u32::try_from(proto.children.len()).unwrap();
    out.extend_from_slice(&size_p.to_le_bytes());
}

fn write_xdi_proto_tail(out: &mut Vec<u8>, proto: &TestProto) {
    let size_lines = u32::try_from(proto.line_info.len()).unwrap();
    out.extend_from_slice(&size_lines.to_le_bytes());
    for &line in &proto.line_info {
        out.extend_from_slice(&line.to_le_bytes());
    }

    let size_locs = u32::try_from(proto.loc_vars.len()).unwrap();
    out.extend_from_slice(&size_locs.to_le_bytes());
    for (name, start_pc, end_pc) in &proto.loc_vars {
        write_xdi_string(out, Some(name));
        out.extend_from_slice(&start_pc.to_le_bytes());
        out.extend_from_slice(&end_pc.to_le_bytes());
    }

    let size_up_names = u32::try_from(proto.upvalue_names.len()).unwrap();
    out.extend_from_slice(&size_up_names.to_le_bytes());
    for name in &proto.upvalue_names {
        write_xdi_string(out, Some(name));
    }
}

pub(crate) fn write_xdi_proto(out: &mut Vec<u8>, root: &TestProto) {
    const WRITE_STEPS_MAX: u32 = 1024;
    let mut stack = arrayvec::ArrayVec::<(&TestProto, u32, bool), 64>::new();
    stack.push((root, 0, false));
    for _ in 0..WRITE_STEPS_MAX {
        let Some((proto, child_idx, entered)) = stack.last_mut() else {
            break;
        };
        let p = *proto;
        if !*entered {
            write_xdi_proto_head(out, p);
            *entered = true;
            continue;
        }
        let idx_usize = crate::u32_to_usize(*child_idx);
        if idx_usize < p.children.len() {
            let child = &p.children[idx_usize];
            *child_idx += 1;
            stack.push((child, 0, false));
        } else {
            let _popped = stack.pop();
            write_xdi_proto_tail(out, p);
        }
    }
    assert!(stack.is_empty());
}

pub(crate) fn build_xdi_chunk(root: &TestProto) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&XDI_MAGIC);
    out.push(u8::try_from(root.upvalues.len()).unwrap());
    write_xdi_proto(&mut out, root);
    out
}

pub(crate) fn encrypt_xdi_in_place(buf: &mut [u8]) {
    for (i, byte) in buf.iter_mut().enumerate() {
        if i <= 1 {
            continue;
        }
        let i64_val = i as u64;
        let v = i64_val.wrapping_mul(0x2021_0507);
        let mask = match i % 3 {
            0 => ((i64_val.wrapping_add(v & 1).wrapping_add(v >> 28)) & 0xFF) as u8,
            1 => (((v >> 16) & 0xFF).wrapping_sub(i64_val) & 0xFF) as u8,
            _ => ((i64_val | (v >> 21)) & 0xFF) as u8,
        };
        *byte ^= mask;
    }
}
