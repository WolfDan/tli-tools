use rayon::prelude::*;
use tartarus_core::{
    Beast, BoardModel, BoardStep, CellKind, CellModel, Context, DEFAULT_SOLVER_PARAMS, EventKind,
    GameMode, INITIAL_EVENT_POOL, Inmate, ObjectiveId, Phase, PreyLink, SUPREME, SimAction,
    SimState, StartEventId, ToolId, apply_action, chain_odds, decide, link_value, playout,
    reroll_odds, solve_parallel, tool_gains,
};

#[test]
fn test_sim_state_size_and_copy() {
    let m = BoardModel::example();
    let (st, problems) = m.build_state(12345);
    assert!(problems.is_empty(), "Problems: {:?}", problems);
    let st_copy = st;
    assert_eq!(st.day, st_copy.day);
    assert_eq!(st.position, 4);
    assert_eq!(st.destinations_slice(), &[5, 6, 8]);
}

#[test]
fn test_example_board_evaluation() {
    let m = BoardModel::example();
    let snap = m.evaluate(9600, 200, 42);
    assert!(snap.problems.is_empty());
    assert!(!snap.move_text.is_empty());
    assert_eq!(snap.options.len(), 3);
}

#[test]
fn test_tool_gains_and_reroll() {
    let mut st = SimState::new_blank(GameMode::Drill, true, 20);
    let opts = [ToolId::Crown, ToolId::Ironbox, ToolId::Forged];
    let gains = tool_gains(&st, &opts).expect("gains should exist");
    assert!(gains[ToolId::Crown.idx()] > gains[ToolId::Ironbox.idx()]);
    st.add_tool(ToolId::Rope);
    let gains2 = tool_gains(&st, &opts).expect("gains with held tool");
    assert!(!gains2[ToolId::Crown.idx()].is_nan());
    let ro =
        reroll_odds(&st, &[ToolId::Ironbox, ToolId::Forged, ToolId::Axe]).expect("reroll odds");
    assert!(ro.next > ro.now);
}

#[test]
fn test_s16_events_and_usurp2_and_weights() {
    assert_eq!(INITIAL_EVENT_POOL.len(), 14);
    assert_eq!(EventKind::Transfer as usize, 16);
    assert_eq!(EventKind::Convict.weight(), 40);
    assert_eq!(EventKind::Execution.weight(), 20);
    assert_eq!(EventKind::Chain.weight(), 20);
    assert_eq!(EventKind::Portion.weight(), 30);
    assert_eq!(EventKind::Fortune.weight(), 20);
    assert_eq!(EventKind::Archenemy.weight(), 30);
    assert_eq!(EventKind::Transfer.weight(), 20);
    assert_eq!(EventKind::Usurp1.weight(), 10);
    assert_eq!(EventKind::parse("closed"), Some(EventKind::Trickster));
    assert_eq!(EventKind::parse("efficiency"), Some(EventKind::Execution));
    assert_eq!(EventKind::parse("whispers"), Some(EventKind::Crow));
    assert_eq!(EventKind::parse("treasure"), Some(EventKind::Hunt));
    assert_eq!(ObjectiveId::parse("devour"), Some(ObjectiveId::Consume));

    // Test Usurp2 promotes 1 inmate by +2 tiers and unlocks Usurp3
    let mut st = SimState::new_blank(GameMode::Drill, false, 20);
    st.cells[1].sealed = false;
    st.cells[1].inmate = Some(Inmate {
        id: 1,
        beast: Beast::Lion,
        tier: 1,
    });
    st.cells[2].sealed = false;
    st.cells[2].inmate = Some(Inmate {
        id: 2,
        beast: Beast::Wolf,
        tier: 1,
    });
    st.phase = Phase::Event;
    st.position = 1;
    st.active_event = Some((EventKind::Usurp2, 1));
    assert!(apply_action(&mut st, &SimAction::Event { accept: true }));
    let t1 = st.cells[1].inmate.unwrap().tier;
    let t2 = st.cells[2].inmate.unwrap().tier;
    assert_eq!(t1 + t2, 4, "Exactly 1 inmate should rise by +2 tiers");
    assert!(st.event_pool_slice().contains(&EventKind::Usurp3));

    // Test Portion, Fortune, Archenemy
    let mut st2 = SimState::new_blank(GameMode::Drill, false, 20);
    st2.cells[3].sealed = false;
    st2.cells[3].inmate = Some(Inmate {
        id: 10,
        beast: Beast::Mammoth,
        tier: 1,
    });
    st2.cells[4].sealed = false;
    st2.cells[4].inmate = Some(Inmate {
        id: 11,
        beast: Beast::Mammoth,
        tier: 2,
    });
    st2.cells[5].sealed = false;
    st2.cells[5].inmate = Some(Inmate {
        id: 12,
        beast: Beast::Mammoth,
        tier: 0,
    });
    st2.position = 3;
    st2.phase = Phase::Event;
    st2.active_event = Some((EventKind::Fortune, 3));
    assert!(apply_action(&mut st2, &SimAction::Event { accept: true }));
    assert_eq!(st2.cells[3].inmate.unwrap().tier, 3);
    assert_eq!(st2.cells[4].inmate.unwrap().tier, 2);
}

#[test]
fn test_guillotine_solver_and_chain_odds() {
    let mut st = SimState::new_blank(GameMode::Drill, false, 20);
    st.add_tool(ToolId::Guillotine);
    st.position = 1;
    st.phase = Phase::Manage;
    st.cells[1].sealed = false;
    st.cells[1].inmate = Some(Inmate {
        id: 1,
        beast: Beast::Wolf,
        tier: 2,
    });
    // Lower-tier Rat in cell 2 (eaten via Guillotine), +1 tier Rat in cell 3
    // (chained after Wolf rises to tier 3)
    st.cells[2].sealed = false;
    st.cells[2].inmate = Some(Inmate {
        id: 2,
        beast: Beast::Rat,
        tier: 0,
    });
    st.cells[3].sealed = false;
    st.cells[3].inmate = Some(Inmate {
        id: 3,
        beast: Beast::Rat,
        tier: 2,
    });
    let ctx = Context::new(&st, &DEFAULT_SOLVER_PARAMS);
    assert!(ctx.guillotine);
    assert_eq!(ctx.meal_min(2, Beast::Wolf.idx() as u8), 0);
    assert_eq!(
        ctx.meals_at(2, Beast::Wolf.idx() as u8, 0).slice(),
        &[Beast::Rat as u8]
    );

    let odds = chain_odds(&st, 1, 2);
    assert!(
        (odds.bites - 1.25).abs() < 1e-6,
        "Guillotine wolf should eat tier-0 Rat + 25% chance to stay Wolf and chain into tier-2 Rat"
    );
    st.start_event = StartEventId::CageFight;
    let odds_cf = chain_odds(&st, 1, 2);
    assert!(
        (odds_cf.bites - 2.0).abs() < 1e-6,
        "With CageFight + Guillotine, Wolf stays Wolf and deterministically chains into tier-2 Rat"
    );
}

#[test]
fn test_horizon_link_value_and_early_stopping() {
    let mut st = SimState::new_blank(GameMode::Drill, false, 20);
    st.day = 20;
    st.days_max = 20;
    st.cells[1].sealed = false;
    st.cells[1].inmate = Some(Inmate {
        id: 1,
        beast: Beast::Lion,
        tier: 3,
    });
    st.cells[2].sealed = false;
    st.cells[2].inmate = Some(Inmate {
        id: 2,
        beast: Beast::Wolf,
        tier: 3,
    });
    let link = PreyLink {
        lion: 1,
        prey: 2,
        prey_beast: Beast::Wolf,
        rounds: 3,
        next_day: 21,
    };
    let ctx = Context::new(&st, &DEFAULT_SOLVER_PARAMS);
    assert_eq!(
        link_value(&st, &ctx, link),
        0.0,
        "PreyLink on Day 20 has 0 remaining mornings"
    );

    // Test early stopping when 2 top candidates are tied (cells 6 and 12 both have
    // Supreme Wolves eating Legendary Rats) and 1 candidate is clearly worse (cell 10
    // is empty) on Day 20
    let mut m = BoardModel::blank();
    m.day = 20;
    m.warden = 4;
    m.step = BoardStep::Patrol;
    m.dest_mask = (1u16 << 6) | (1u16 << 10) | (1u16 << 12);
    m.cells[10].kind = CellKind::Empty;
    for &n in &[6usize, 12usize] {
        m.cells[n] = CellModel {
            kind: CellKind::Inmate,
            beast: Beast::Wolf,
            tier: SUPREME,
            ..CellModel::sealed()
        };
    }
    m.cells[1] = CellModel {
        kind: CellKind::Inmate,
        beast: Beast::Rat,
        tier: SUPREME,
        ..CellModel::sealed()
    };
    let (st_tie, _) = m.build_state(77);
    let t0 = std::time::Instant::now();
    let summary = solve_parallel(&st_tie, 9600, 85, 77, true);
    let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;
    assert!(summary.done);
    assert!(
        summary.rounds <= 256,
        "Expected early stop in <= 256 rollouts on tied-top + inferior-third board, \
         got {} rounds in {:.2}ms",
        summary.rounds,
        elapsed_ms
    );
}

#[test]
fn test_s16_drill_rainbow_ev_and_solver_speed() {
    let runs = 400usize;
    let t0 = std::time::Instant::now();
    let rainbows_total: u64 = (0..runs)
        .into_par_iter()
        .map(|r| {
            let seed = 1000u32.wrapping_add((r as u32).wrapping_mul(0x9e3779b1));
            let mut st = SimState::new_drill_run(seed, &[ToolId::Crown, ToolId::Rope]);
            st.params.rewards = true;
            let mut rng = seed ^ 0x5bd1e995;
            playout(&mut st, None, &mut rng);
            st.vault_len as u64
        })
        .sum();
    let dt_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let ev = (rainbows_total as f64) / (runs as f64);
    println!(
        "S16 Drill 400 runs EV (Crown+Rope): {:.3} Rainbows/run in {:.2}ms ({:.0} runs/sec)",
        ev,
        dt_ms,
        (runs as f64) / (dt_ms / 1000.0)
    );
    assert!(
        ev > 2.0,
        "Expected > 2.0 Rainbows/run with Crown+Rope, got {ev}"
    );
}

#[test]
fn test_transfer_event_accepts_on_developed_board_and_foodchain_alias() {
    assert_eq!(
        StartEventId::parse("foodchain"),
        Some(StartEventId::CageFight)
    );

    let mut st = SimState::new_blank(GameMode::Drill, true, 20);
    for (cn, tier) in [
        (1usize, SUPREME),
        (2usize, 3),
        (3usize, SUPREME),
        (4usize, 3),
    ] {
        st.cells[cn].sealed = false;
        st.cells[cn].inmate = Some(Inmate {
            id: cn as u16,
            beast: Beast::Lion,
            tier,
        });
    }
    st.position = 1;
    st.phase = Phase::Event;
    st.active_event = Some((EventKind::Transfer, 1));
    let mut rng = 42u32;
    let act = decide(&st, None, &mut rng);
    assert_eq!(
        act,
        Some(SimAction::Event { accept: true }),
        "Transfer of Convicts replicates the current inmate into a sealed cell \
         and must always be accepted"
    );
    assert!(apply_action(&mut st, &SimAction::Event { accept: true }));
    let open_count = st.cells.iter().filter(|c| !c.sealed).count();
    assert_eq!(
        open_count, 6,
        "Office + 4 open cells + 1 replicated sealed cell = 6 open cells"
    );
}

#[test]
fn test_s16_random_run_rainbow_ev_and_solve_parallel_benchmark() {
    let runs = 1000usize;
    let t0 = std::time::Instant::now();
    let rainbows_total: u64 = (0..runs)
        .into_par_iter()
        .map(|r| {
            let seed = 7777u32.wrapping_add((r as u32).wrapping_mul(0x9e3779b1));
            let mut st = SimState::new_random_run(seed);
            let mut rng = seed ^ 0x5bd1e995;
            playout(&mut st, None, &mut rng);
            st.vault_len as u64
        })
        .sum();
    let dt_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let ev = (rainbows_total as f64) / (runs as f64);
    println!(
        "S16 Random Run (SimState::new_random_run) 1000 runs EV: \
         {:.3} Rainbows/run in {:.2}ms ({:.0} runs/sec)",
        ev,
        dt_ms,
        (runs as f64) / (dt_ms / 1000.0)
    );
    assert!(
        ev > 3.5,
        "Expected > 3.5 Rainbows/run across random 20-day S16 runs, got {ev}"
    );

    let m = BoardModel::example();
    let (st_ex, _) = m.build_state(12345);
    let t_solve = std::time::Instant::now();
    let summary = solve_parallel(&st_ex, 2400, 500, 12345, true);
    let solve_ms = t_solve.elapsed().as_secs_f64() * 1000.0;
    let playouts_total = summary.rounds * summary.options.len().max(1);
    println!(
        "solve_parallel benchmark: {} rounds ({} total playouts across {} candidates) \
         in {:.2}ms ({:.0} playouts/sec), best = {}",
        summary.rounds,
        playouts_total,
        summary.options.len(),
        solve_ms,
        (playouts_total as f64) / (solve_ms / 1000.0),
        summary
            .options
            .first()
            .map(|r| r.label.as_str())
            .unwrap_or("none")
    );
    assert!(summary.done);
}
