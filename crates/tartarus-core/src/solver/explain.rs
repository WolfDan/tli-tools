use crate::engine::{
    APPETITE_ODDS, APPETITE_TRIES, CROW_CELLS, ChoiceId, EscortKind, EventKind, FREEDOM_CELLS,
    Inmate, OfficeChoice, PendingKind, SUPREME, SimState, ToolId, cell_label, describe_inmate,
    plural, tier_name,
};

use super::chain_dp::{Context, chain_odds};
use super::heuristics::{maiden_bonus, patrol_cell_why, swap_gain_here};
use super::params::{SimAction, SolverParams};
use super::search::{candidates, decide};
use super::tools::{reroll_odds, tool_gains};

#[derive(Clone, Debug)]
pub struct InstantPick {
    pub action: SimAction,
    pub label: String,
    pub why: String,
}

pub fn signed_fmt(v: f64) -> String {
    let sign = if v >= 0.0 { "+" } else { "\u{2212}" };
    format!("{sign}{:.2}", v.abs())
}

pub fn convoy_why(st: &SimState, cell_mask: u16, escort: EscortKind, p: &SolverParams) -> String {
    let Some(ch) = st.challenge.as_ref() else {
        return String::new();
    };
    let slot = ch.deliveries_len as usize;
    if slot >= 2 {
        return "The heavy contract ends the run, so it takes the most points your vouchers can escort.".to_string();
    }
    let keep = if slot == 0 { p.keep1 } else { p.keep2 };
    let mut kept_cells: Vec<(u8, Inmate)> = st
        .cells
        .iter()
        .filter_map(|c| {
            if !c.sealed && (cell_mask & (1u16 << c.n)) == 0 {
                c.inmate.map(|i| (c.n, i))
            } else {
                None
            }
        })
        .collect();
    kept_cells.sort_by_key(|a| std::cmp::Reverse(a.1.tier));
    let kept_names: Vec<String> = kept_cells
        .into_iter()
        .take(2)
        .map(|(n, i)| format!("{} {}", describe_inmate(i), cell_label(n)))
        .collect();
    let left = ch.vouchers.saturating_sub(escort.cost());
    let next = if slot == 0 {
        "the next contracts"
    } else {
        "the heavy contract"
    };
    let keep_text = if !kept_names.is_empty() {
        format!(" Your {} stay for {next}.", kept_names.join(" and "))
    } else {
        String::new()
    };
    let voucher_text = if left >= EscortKind::Heavy.cost() {
        format!(
            " It leaves {}, so the heavy contract can still go by heavy escort.",
            plural(left as usize, "voucher")
        )
    } else {
        format!(" It leaves {}.", plural(left as usize, "voucher"))
    };
    let lead = if keep >= 1.0 {
        "Meets the contract with the inmates worth least to you."
    } else if keep > 0.0 {
        "Meets the contract with the most points it can spare."
    } else {
        "Meets the contract for the most points."
    };
    format!("{lead}{keep_text}{voucher_text}")
}

pub fn reason(st: &SimState, action: &SimAction, params: Option<&'static SolverParams>) -> String {
    let p = params.unwrap_or_else(|| SolverParams::for_state(st));
    let mut ctx = Context::new(st, p);
    let here = &st.cells[st.position as usize];
    match *action {
        SimAction::Patrol { cell } => {
            let why = patrol_cell_why(st, &mut ctx, cell);
            if let Some(first) = why.chars().next() {
                return format!("{}{}.", first.to_uppercase(), &why[first.len_utf8()..]);
            }
            String::new()
        }
        SimAction::Devour { to } => {
            let odds = chain_odds(st, st.position, to);
            let over = if swap_gain_here(st, &mut ctx) > 0.0 {
                " It beats a Replace here, and a day has one or the other."
            } else {
                ""
            };
            format!(
                "The best first meal: picking its later meals well, its chain averages {:.1} bites, with {:.0}% to reach Rainbow.{over}",
                odds.bites,
                100.0 * odds.rainbow
            )
        }
        SimAction::Chain { to } => {
            let odds = chain_odds(st, st.position, to);
            format!(
                "The best next meal: going on with the best meal each time, the chain averages {:.1} more bites, with {:.0}% to reach Rainbow.",
                odds.bites,
                100.0 * odds.rainbow
            )
        }
        SimAction::StopChain => {
            "No meal left is worth eating: each would cost more than it gains, so press Cancel and keep them on the block.".to_string()
        }
        SimAction::Hunt { to } => {
            let Some(prey) = st.cells[to as usize].inmate else {
                return String::new();
            };
            let (_, s_len) = st.start_options();
            let over = if s_len > 0 || swap_gain_here(st, &mut ctx) > 0.0 {
                " That beats today\u{2019}s consume or Replace, and a day has one of them."
            } else {
                ""
            };
            format!(
                "A Prey mark: this Lion and the {} each rise a tier every morning for {}, as long as both stay on the block and keep their beasts.{over}",
                describe_inmate(prey),
                plural(st.params.prey_rounds as usize, "morning")
            )
        }
        SimAction::Office { choice } => {
            if choice == OfficeChoice::Bestow {
                "A random inmate rises a tier now, which beats a couple of new event doors."
                    .to_string()
            } else {
                "The block is thin, so new event doors are worth more than raising it.".to_string()
            }
        }
        SimAction::Swap => {
            let (_, s_len) = st.start_options();
            let over = if s_len > 0 {
                " That beats today\u{2019}s best consume, and a day has one or the other."
            } else {
                ""
            };
            let tier = here.inmate.map(|i| i.tier).unwrap_or(0);
            let (m_chance, _) = maiden_bonus(st, &ctx, tier);
            let odds = format!("{:.0}% of the time", 100.0 * m_chance);
            if here.ivory {
                let m_str = if m_chance > 0.0 {
                    format!(", and the Severed Iron Maiden adds 3 tiers to the new inmate {odds}")
                } else {
                    String::new()
                };
                return format!(
                    "A Tusk cell: replacing its inmate raises other inmates a tier{m_str}.{over}"
                );
            }
            if ctx.scapegoat_loss.is_some() {
                let chrom = if tier >= SUPREME { ", a Chromatic" } else { "" };
                return format!(
                    "The Scapegoat's Guillotine brings the new inmate in above {}{chrom}, worth more than the 2 random inmates it removes.{over}",
                    tier_name(tier)
                );
            }
            if m_chance > 0.0 {
                return format!(
                    "The Severed Iron Maiden adds 3 tiers to the new inmate {odds}, so a fresh roll beats this {} inmate on average.{over}",
                    tier_name(tier)
                );
            }
            if ctx.axe_doors > 0 {
                return format!(
                    "A {} inmate is worth less than a fresh roll on average, and a higher roll makes the Gaoler's Execution Axe open {}.{over}",
                    tier_name(tier),
                    plural(ctx.axe_doors, "door")
                );
            }
            format!(
                "A {} inmate is worth less than a fresh roll on average.{over}",
                tier_name(tier)
            )
        }
        SimAction::End => {
            let (s_opts, s_len) = st.start_options();
            if !st.can_devour_now() || s_len == 0 {
                return "Nothing left today is worth doing.".to_string();
            }
            let marked = here
                .inmate
                .is_some_and(|i| st.link_of(i.id).is_some())
                || s_opts[..s_len].iter().any(|o| {
                    st.cells[o.cell as usize]
                        .inmate
                        .is_some_and(|m| st.link_of(m.id).is_some())
                });
            if marked {
                return "Skip today\u{2019}s consume: it would end a prey mark, and the free tier each morning is worth more than one bite.".to_string();
            }
            "Every consume here would cost more than it gains today.".to_string()
        }
        SimAction::Event { accept } => {
            let Some((kind, cell_n)) = st.active_event else {
                return String::new();
            };
            match kind {
                EventKind::Chain => {
                    if accept {
                        "Your next walk opens every sealed door on the way.".to_string()
                    } else {
                        "More inmates would hurt this block today.".to_string()
                    }
                }
                EventKind::Trickster => {
                    if accept {
                        let who = st.cells[cell_n as usize].inmate.unwrap();
                        format!(
                            "The {} becomes a Supreme, one step from Rainbow.",
                            describe_inmate(who)
                        )
                    } else {
                        "It is already a Supreme; accepting would only change its beast."
                            .to_string()
                    }
                }
                EventKind::Appetite => format!(
                    "The lowest-rarity inmate gets {APPETITE_TRIES} tries at +1 tier, for {:.1} tiers on average.",
                    (APPETITE_TRIES as f64) * APPETITE_ODDS
                ),
                EventKind::Convict => {
                    "Another cell opens on a Legendary inmate, with no patrol needed.".to_string()
                }
                EventKind::Crow => format!(
                    "Free information: the rarity behind the next {CROW_CELLS} locked doors."
                ),
                EventKind::Freedom => {
                    format!("Free inmates: {FREEDOM_CELLS} more locked cells open.")
                }
                EventKind::Usurp1 => {
                    "A random inmate rises a tier, and Usurpation II joins the events.".to_string()
                }
                EventKind::Usurp2 => {
                    "One random inmate rises 2 tiers, and Usurpation III joins the events."
                        .to_string()
                }
                EventKind::Usurp3 => {
                    "Two random inmates rise 2 tiers each, and Usurpation IV joins the events."
                        .to_string()
                }
                EventKind::Usurp4 => "Every inmate on the block rises 2 tiers.".to_string(),
                EventKind::Ratway => {
                    "A free move to any cell but the office: the best door or inmate on the whole block, not just the next six cells.".to_string()
                }
                EventKind::Hunt => {
                    "Free information, and its card has only Confirm: every locked door holding a Normal or Magic shows its rarity, so the doors left dark hold Rare or better.".to_string()
                }
                EventKind::Portion => {
                    "Up to 3 random inmates on the block rise a tier.".to_string()
                }
                EventKind::Fortune => {
                    "This inmate rises a tier for each other inmate of the same beast.".to_string()
                }
                EventKind::Archenemy => {
                    "A locked cell opens with an inmate guaranteed to be a meal for this inmate."
                        .to_string()
                }
                EventKind::Transfer => {
                    "This inmate is replicated into a random locked cell.".to_string()
                }
                EventKind::Execution => {
                    if accept {
                        let exec = st.cells[cell_n as usize].inmate.unwrap();
                        let (_, vlen) = st.execution_victims(cell_n);
                        format!(
                            "The {} removes {} and rises as many tiers: it gains more than it removes.",
                            describe_inmate(exec),
                            plural(vlen, "inmate")
                        )
                    } else {
                        "It would remove more than it gains.".to_string()
                    }
                }
            }
        }
        SimAction::Choose(id) => {
            if st.pending_len == 0 {
                return String::new();
            }
            if st.pending[0].kind == PendingKind::Gift {
                return "Worth the most on this board right now.".to_string();
            }
            let ChoiceId::Tool(t_id) = id else {
                return String::new();
            };
            let mut tool_opts = [ToolId::Revenge; 4];
            let mut tlen = 0;
            for &c in st.pending[0].slice() {
                if let ChoiceId::Tool(t) = c {
                    tool_opts[tlen] = t;
                    tlen += 1;
                }
            }
            if ctx.p.tool_sets
                && let Some(gains) = tool_gains(st, &tool_opts[..tlen])
            {
                let add = gains[t_id.idx()];
                if !add.is_nan() {
                    let held_str = if st.tools_len == 1 {
                        format!("With the {}, it", st.tools_order[0].name())
                    } else if st.tools_len > 1 {
                        "With the instruments you hold, it".to_string()
                    } else {
                        "It".to_string()
                    };
                    let vs = if st.can_reroll() {
                        if let Some(odds) = reroll_odds(st, &tool_opts[..tlen]) {
                            format!(
                                " A re-roll's best would add about {} on average.",
                                signed_fmt(odds.next)
                            )
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };
                    return format!(
                        "{held_str} adds about {} Rainbows per run in simulated runs.{vs}",
                        signed_fmt(add)
                    );
                }
            }
            if let Some(rank) = ctx.p.tool_rank.iter().position(|&x| x == t_id) {
                return format!(
                    "Ranked #{} of {} tools in training.",
                    rank + 1,
                    ctx.p.tool_rank.len()
                );
            }
            String::new()
        }
        SimAction::Reroll => {
            let mut tool_opts = [ToolId::Revenge; 4];
            let mut tlen = 0;
            if st.pending_len > 0 {
                for &c in st.pending[0].slice() {
                    if let ChoiceId::Tool(t) = c {
                        tool_opts[tlen] = t;
                        tlen += 1;
                    }
                }
            }
            if let Some(odds) = reroll_odds(st, &tool_opts[..tlen]) {
                format!(
                    "The best instrument on offer adds about {} Rainbows per run; a new offer's best adds about {} on average.",
                    signed_fmt(odds.now),
                    signed_fmt(odds.next)
                )
            } else {
                "A new offer is likely to hold a better instrument.".to_string()
            }
        }
        SimAction::Deliver {
            cell_mask,
            escort,
            ..
        } => convoy_why(st, cell_mask, escort, ctx.p),
        SimAction::Skip => "Nothing on the block meets this contract.".to_string(),
        SimAction::Close => "Nothing meets the heavy contract yet.".to_string(),
        SimAction::Early => {
            if ctx.p.early_points < 0.0 {
                "The heavy convoy is as full as your escort allows, so waiting can\u{2019}t add points. Shipping now locks in the pass.".to_string()
            } else if ctx.p.early_points < 1e9 {
                format!(
                    "The heavy convoy scores at least {} points now. Shipping locks in the pass.",
                    ctx.p.early_points
                )
            } else {
                "The heavy contract can be met now. Shipping it ends the run.".to_string()
            }
        }
    }
}

pub fn instant(st: &SimState, params: Option<&'static SolverParams>) -> Option<InstantPick> {
    let p = params.unwrap_or_else(|| SolverParams::for_state(st));
    let mut rng = 1u32;
    let action = decide(st, Some(p), &mut rng)?;
    let cands = candidates(st);
    let label = cands
        .into_iter()
        .find(|c| c.action.same_action(&action))
        .map(|c| c.label)
        .unwrap_or_else(|| action.type_name().to_string());
    let why = reason(st, &action, Some(p));
    Some(InstantPick { action, label, why })
}
