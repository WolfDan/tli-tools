//! `FConfigAffix` expression-tree AST types and non-recursive `ArrayVec` work-stack decoder.

use crate::bin::error::{BinError, BinLimitKind, WireContext};
use crate::bin::tables::affix_ops::AffixOp3Code;
use crate::bin::types::AFFIX_STACK_DEPTH_MAX;
use crate::bin::wire::{RawTreeNodeMapEntry, WireCursor};
use crate::u32_to_usize;
use arrayvec::ArrayVec;

/// Operand kind inside a tag-3 expression node ([`AffixOp3Code`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AffixSubOpKind {
    /// 1-byte `is_formula` flag followed by either an `f64` literal or a sub-formula branch.
    FormulaOrF64,
    /// 1-byte `is_formula` flag followed by either an `i32` literal or a sub-formula branch.
    FormulaOrI32,
    /// 1-byte `is_formula` flag followed by either a `bool` literal or a sub-formula branch.
    FormulaOrBool,
    /// 1-byte `is_formula` flag followed by either a `u8` literal or a sub-formula branch.
    FormulaOrU8,
    /// `<count: u32>` followed by `count` `FormulaOrI32` operands.
    ArrayFormulaOrI32,
}

/// 12-byte lookup table entry inside a tag-4 [`AffixTreeNode::LookupTable`] node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AffixLookupEntry {
    /// Lookup key.
    pub key: i32,
    /// First mapped value.
    pub val_a: i32,
    /// Second mapped value.
    pub val_b: i32,
}

/// Flattened expression tree node stored in wire pre-order inside [`FConfigAffix`].
#[derive(Debug, Clone, PartialEq)]
pub enum AffixTreeNode {
    /// Integer node (`tag` `0`, `5`, `6`, or `7`).
    Int {
        /// Raw node tag (`0`, `5`, `6`, or `7`).
        tag: u8,
        /// 32-bit signed integer operand.
        value: i32,
    },
    /// 64-bit float literal node (`tag == 1`).
    Float(f64),
    /// 1-byte flag/enum node (`tag == 2`).
    Byte(u8),
    /// Tag-3 operator node (`tag == 3`), whose operands follow in [`FConfigAffix::operands`].
    Op3(AffixOp3Code),
    /// Tag-4 lookup table node (`tag == 4`).
    LookupTable(Box<[AffixLookupEntry]>),
    /// Tag-8 marker node (`tag == 8`).
    Tag8,
}

/// Flattened operand record produced by root formulas and [`AffixOp3Code`] sub-operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AffixOperand {
    /// Literal `f64` value (`FormulaOrF64` with `is_formula == 0`).
    LiteralF64(f64),
    /// Literal `i32` value (`FormulaOrI32` with `is_formula == 0`).
    LiteralI32(i32),
    /// Literal `bool` value (`FormulaOrBool` with `is_formula == 0`).
    LiteralBool(bool),
    /// Literal `u8` value (`FormulaOrU8` with `is_formula == 0`).
    LiteralU8(u8),
    /// Sub-formula branch consuming the next `node_count` nodes in [`FConfigAffix::nodes`].
    Formula {
        /// Number of child tree nodes in this sub-formula branch.
        node_count: u32,
    },
    /// `ArrayFormulaOrI32` header followed by `count` `LiteralI32` / `Formula` operands.
    ArrayI32 {
        /// Number of `FormulaOrI32` elements in this array operand.
        count: u32,
    },
}

/// Decoded `FConfigAffix` expression tree stored in flat contiguous arrays (zero recursive `Drop`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FConfigAffix {
    /// Pre-order expression tree nodes across all formula branches.
    pub nodes: Box<[AffixTreeNode]>,
    /// Pre-order operands across root formulas and `Op3` nodes.
    pub operands: Box<[AffixOperand]>,
    /// Number of root `FormulaOrF64` expressions.
    pub root_count: u32,
    /// Affix expression identifier.
    pub id: i32,
}

const AFFIX_STACK_MAX_USIZE: usize = u32_to_usize(AFFIX_STACK_DEPTH_MAX);

#[derive(Debug, Clone, Copy)]
enum AffixWorkFrame {
    Nodes { remaining: u32 },
    Op3 { code: AffixOp3Code, idx: u8 },
    ArrayI32 { remaining: u32 },
}

fn push_work_frame(
    stack: &mut ArrayVec<AffixWorkFrame, AFFIX_STACK_MAX_USIZE>,
    frame: AffixWorkFrame,
) -> Result<(), BinError> {
    stack.try_push(frame).map_err(|_| BinError::LimitExceeded {
        kind: BinLimitKind::AffixStackDepth,
        actual: u64::from(AFFIX_STACK_DEPTH_MAX) + 1,
        max: u64::from(AFFIX_STACK_DEPTH_MAX),
    })
}

fn step_sub_op(
    cur: &mut WireCursor<'_>,
    kind: AffixSubOpKind,
    stack: &mut ArrayVec<AffixWorkFrame, AFFIX_STACK_MAX_USIZE>,
    operands: &mut Vec<AffixOperand>,
) -> Result<(), BinError> {
    if matches!(kind, AffixSubOpKind::ArrayFormulaOrI32) {
        let count = cur.read_collection_count(WireContext::AffixExpression)?;
        operands.push(AffixOperand::ArrayI32 { count });
        return push_work_frame(stack, AffixWorkFrame::ArrayI32 { remaining: count });
    }
    let is_formula = cur.read_u8(WireContext::AffixExpression)?;
    if is_formula == 0 {
        let op = match kind {
            AffixSubOpKind::FormulaOrF64 => AffixOperand::LiteralF64(cur.read_f64()?),
            AffixSubOpKind::FormulaOrI32 => {
                AffixOperand::LiteralI32(cur.read_i32_ctx(WireContext::AffixExpression)?)
            }
            AffixSubOpKind::FormulaOrBool => {
                AffixOperand::LiteralBool(cur.read_u8(WireContext::AffixExpression)? != 0)
            }
            AffixSubOpKind::FormulaOrU8 | AffixSubOpKind::ArrayFormulaOrI32 => {
                AffixOperand::LiteralU8(cur.read_u8(WireContext::AffixExpression)?)
            }
        };
        operands.push(op);
        Ok(())
    } else {
        let node_count = cur.read_collection_count(WireContext::AffixExpression)?;
        operands.push(AffixOperand::Formula { node_count });
        push_work_frame(
            stack,
            AffixWorkFrame::Nodes {
                remaining: node_count,
            },
        )
    }
}

fn step_tree_node(
    cur: &mut WireCursor<'_>,
    stack: &mut ArrayVec<AffixWorkFrame, AFFIX_STACK_MAX_USIZE>,
    nodes: &mut Vec<AffixTreeNode>,
    total_nodes: &mut u32,
) -> Result<(), BinError> {
    let max_nodes = cur.limits().tree_nodes_count_max();
    if *total_nodes == max_nodes {
        return Err(BinError::LimitExceeded {
            kind: BinLimitKind::TreeNodesCount,
            actual: u64::from(*total_nodes) + 1,
            max: u64::from(max_nodes),
        });
    }
    *total_nodes += 1;
    let tag = cur.read_u8(WireContext::AffixExpression)?;
    match tag {
        0 | 5 | 6 | 7 => {
            let value = cur.read_i32_ctx(WireContext::AffixExpression)?;
            nodes.push(AffixTreeNode::Int { tag, value });
        }
        1 => nodes.push(AffixTreeNode::Float(cur.read_f64()?)),
        2 => nodes.push(AffixTreeNode::Byte(
            cur.read_u8(WireContext::AffixExpression)?,
        )),
        3 => {
            let raw_op = cur.read_u8(WireContext::AffixExpression)?;
            let code = AffixOp3Code::from_raw(raw_op)?;
            nodes.push(AffixTreeNode::Op3(code));
            push_work_frame(stack, AffixWorkFrame::Op3 { code, idx: 0 })?;
        }
        4 => {
            let _buckets = cur.read_u32(WireContext::AffixExpression)?;
            let count = cur.read_collection_count(WireContext::AffixExpression)?;
            let raw =
                cur.take_slice_elems::<RawTreeNodeMapEntry>(count, WireContext::AffixExpression)?;
            let mut entries = Vec::with_capacity(u32_to_usize(count));
            for e in raw {
                entries.push(AffixLookupEntry {
                    key: e.key.get(),
                    val_a: e.val_a.get(),
                    val_b: e.val_b.get(),
                });
            }
            nodes.push(AffixTreeNode::LookupTable(entries.into_boxed_slice()));
        }
        8 => nodes.push(AffixTreeNode::Tag8),
        other => return Err(BinError::InvalidTreeNodeTag { raw_tag: other }),
    }
    Ok(())
}

impl FConfigAffix {
    /// Decodes an `FConfigAffix` expression tree iteratively using a bounded `ArrayVec` stack.
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let id = cur.read_i32_ctx(WireContext::AffixExpression)?;
        let has_tree = cur.read_u8(WireContext::AffixExpression)?;
        if has_tree == 0 {
            return Ok(Self {
                nodes: Box::new([]),
                operands: Box::new([]),
                root_count: 0,
                id,
            });
        }
        let root_count = cur.read_collection_count(WireContext::AffixExpression)?;
        let mut nodes = Vec::new();
        let mut operands = Vec::with_capacity(u32_to_usize(root_count));
        let mut stack: ArrayVec<AffixWorkFrame, AFFIX_STACK_MAX_USIZE> = ArrayVec::new();
        let mut total_nodes: u32 = 0;

        for _ in 0..root_count {
            step_sub_op(cur, AffixSubOpKind::FormulaOrF64, &mut stack, &mut operands)?;
            while let Some(top) = stack.last_mut() {
                match *top {
                    AffixWorkFrame::Nodes { remaining } => {
                        if remaining == 0 {
                            stack.pop();
                        } else {
                            *top = AffixWorkFrame::Nodes {
                                remaining: remaining - 1,
                            };
                            step_tree_node(cur, &mut stack, &mut nodes, &mut total_nodes)?;
                        }
                    }
                    AffixWorkFrame::Op3 { code, idx } => {
                        let schema = code.schema();
                        if usize::from(idx) >= schema.len() {
                            stack.pop();
                        } else {
                            let kind = schema[usize::from(idx)];
                            *top = AffixWorkFrame::Op3 { code, idx: idx + 1 };
                            step_sub_op(cur, kind, &mut stack, &mut operands)?;
                        }
                    }
                    AffixWorkFrame::ArrayI32 { remaining } => {
                        if remaining == 0 {
                            stack.pop();
                        } else {
                            *top = AffixWorkFrame::ArrayI32 {
                                remaining: remaining - 1,
                            };
                            step_sub_op(
                                cur,
                                AffixSubOpKind::FormulaOrI32,
                                &mut stack,
                                &mut operands,
                            )?;
                        }
                    }
                }
            }
        }
        Ok(Self {
            nodes: nodes.into_boxed_slice(),
            operands: operands.into_boxed_slice(),
            root_count,
            id,
        })
    }

    /// Decodes a length-prefixed array of `FConfigAffix` from `cur`.
    #[inline(never)]
    pub fn read_array(cur: &mut WireCursor<'_>) -> Result<Box<[Self]>, BinError> {
        let count = cur.read_collection_count(WireContext::ArrayColumn)?;
        let mut out = Vec::with_capacity(u32_to_usize(count));
        for _ in 0..count {
            out.push(Self::read(cur)?);
        }
        Ok(out.into_boxed_slice())
    }
}
