//! Lock the port to the upstream CLI result on bundled CUPID + example policy.
//!
//! Upstream `stock_simulator` (same engine, release build) printed:
//! ending equity 140814.80, return 40.81%, 3 trades, win rate 66.67%,
//! max drawdown 10.87%, avg hold 15.3 days.

use simulator_core::indicators::IndicatorRow;
use simulator_core::{execute, load_prices_csv, parse_policy_text, run_backtest};

const CSV: &str = include_str!("../../../public/data/CUPID.csv");
const YAML: &str = include_str!("../../../policy/example_policy.yaml");

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn round1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

#[test]
fn public_policy_file_matches_canonical() {
    let public_yaml = include_str!("../../../public/policy/example_policy.yaml");
    assert_eq!(YAML, public_yaml);
}

#[test]
fn cupid_example_policy_matches_upstream() {
    let out = execute(CSV, YAML).expect("backtest");
    let s = &out.stats;
    assert_eq!(s.num_bars, 246);
    assert_eq!(s.first_date, "2025-10-06");
    assert_eq!(s.last_date, "2026-10-05");
    assert_eq!(s.num_trades, 3);
    assert_eq!(s.winning_trades, 2);
    assert_eq!(s.losing_trades, 1);
    assert_eq!(round2(s.starting_capital), 100_000.00);
    assert_eq!(round2(s.ending_equity), 140_814.80);
    assert_eq!(round2(s.total_pnl), 40_814.80);
    assert_eq!(round2(s.total_return_pct), 40.81);
    assert_eq!(round2(s.win_rate_pct), 66.67);
    assert_eq!(round2(s.max_drawdown_pct), 10.87);
    assert_eq!(round1(s.avg_hold_days), 15.3);

    let expected = [
        (
            "2026-03-11",
            "2026-03-13",
            1035_i64,
            "stop_loss",
            -10_867.50,
            2_i64,
        ),
        (
            "2026-04-02",
            "2026-05-07",
            990,
            "trailing_stop",
            41_570.10,
            35,
        ),
        (
            "2026-09-21",
            "2026-09-30",
            466,
            "trailing_stop",
            10_112.20,
            9,
        ),
    ];
    assert_eq!(out.trades.len(), expected.len());
    for (trade, exp) in out.trades.iter().zip(expected) {
        assert_eq!(trade.entry_date, exp.0);
        assert_eq!(trade.exit_date, exp.1);
        assert_eq!(trade.shares, exp.2);
        assert_eq!(trade.side, "long");
        assert_eq!(trade.exit_reason, exp.3);
        assert_eq!(round2(trade.pnl), exp.4);
        assert_eq!(trade.hold_days, exp.5);
    }

    assert!(out.policy.require_price_above_sma_trend);
    assert_eq!(out.policy.sma_trend, Some(100));
    assert_eq!(out.policy.ema_fast, Some(12));
    assert_eq!(out.policy.ema_slow, Some(26));
    assert_eq!(out.policy.stop_loss_pct, Some(8.0));
    assert_eq!(out.policy.trailing_stop_pct, Some(10.0));
    assert_eq!(out.policy.atr_stop_mult, None);
    assert_eq!(out.policy.trailing_atr_mult, None);
    assert_eq!(out.policy.fill_at, "next_open");
    assert_eq!(out.equity.len(), 246);
    assert_eq!(out.prices.len(), 246);
    assert_eq!(round2(out.equity.last().unwrap().equity), 140_814.80);
}

#[test]
fn json_policy_matches_yaml_policy() {
    let policy = parse_policy_text(YAML).unwrap();
    let json = serde_json::to_string(&policy).unwrap();
    assert!(json.starts_with('{'));
    let from_yaml = execute(CSV, YAML).unwrap();
    let from_json = execute(CSV, &json).unwrap();
    assert_eq!(
        round2(from_yaml.stats.ending_equity),
        round2(from_json.stats.ending_equity)
    );
    assert_eq!(from_yaml.stats.num_trades, from_json.stats.num_trades);
    for (a, b) in from_yaml.trades.iter().zip(from_json.trades.iter()) {
        assert_eq!(a.entry_date, b.entry_date);
        assert_eq!(a.exit_date, b.exit_date);
        assert_eq!(a.shares, b.shares);
        assert_eq!(round2(a.pnl), round2(b.pnl));
        assert_eq!(a.exit_reason, b.exit_reason);
    }
}

#[test]
fn indicators_depend_only_on_the_prefix() {
    let bars = load_prices_csv(CSV).unwrap();
    let policy = parse_policy_text(YAML).unwrap();
    let full = simulator_core::indicators::compute(&bars, &policy);
    for k in [40_usize, 100, 180, bars.len()] {
        let prefix = simulator_core::indicators::compute(&bars[..k], &policy);
        for i in 0..k {
            assert_eq!(
                row_key(&prefix[i]),
                row_key(&full[i]),
                "indicator mismatch at i={i} with prefix {k}"
            );
        }
    }
}

fn row_key(
    row: &IndicatorRow,
) -> (
    Option<u64>,
    Option<u64>,
    Option<u64>,
    Option<u64>,
    bool,
    bool,
) {
    (
        row.sma_trend.map(|v| v.to_bits()),
        row.ema_fast.map(|v| v.to_bits()),
        row.rsi.map(|v| v.to_bits()),
        row.adx.map(|v| v.to_bits()),
        row.ema_cross_up,
        row.adx_cross_up,
    )
}

#[test]
fn bars_after_last_exit_do_not_change_closed_trades() {
    let bars = load_prices_csv(CSV).unwrap();
    let policy = parse_policy_text(YAML).unwrap();
    let cut = bars
        .iter()
        .position(|b| b.date.to_string() == "2026-09-30")
        .unwrap();
    let full = run_backtest(bars.clone(), &policy);
    let prefix = run_backtest(bars[..=cut].to_vec(), &policy);
    assert_eq!(full.trades.len(), prefix.trades.len());
    for (a, b) in full.trades.iter().zip(prefix.trades.iter()) {
        assert_eq!(a.entry_date, b.entry_date);
        assert_eq!(a.exit_date, b.exit_date);
        assert_eq!(a.shares, b.shares);
        assert_eq!(a.pnl.to_bits(), b.pnl.to_bits());
        assert_eq!(a.exit_reason, b.exit_reason);
    }
}

#[test]
fn mutating_a_future_bar_does_not_change_earlier_equity() {
    let bars = load_prices_csv(CSV).unwrap();
    let policy = parse_policy_text(YAML).unwrap();
    // Last bar: a new buy cannot fill (no next open), and the book is flat,
    // so earlier equity and closed trades must be identical.
    let idx = bars.len() - 1;
    assert_eq!(bars[idx].date.to_string(), "2026-10-05");
    let mut mutated = bars.clone();
    mutated[idx].close += 25.0;
    mutated[idx].high = mutated[idx].high.max(mutated[idx].close);
    mutated[idx].low = mutated[idx]
        .low
        .min(mutated[idx].open.min(mutated[idx].close));

    let original = run_backtest(bars, &policy);
    let changed = run_backtest(mutated, &policy);
    assert_eq!(original.trades.len(), changed.trades.len());
    for (a, b) in original.trades.iter().zip(changed.trades.iter()) {
        assert_eq!(a.entry_date, b.entry_date);
        assert_eq!(a.exit_price.to_bits(), b.exit_price.to_bits());
        assert_eq!(a.pnl.to_bits(), b.pnl.to_bits());
    }
    for i in 0..idx {
        assert_eq!(
            original.equity_curve[i].equity.to_bits(),
            changed.equity_curve[i].equity.to_bits(),
            "equity changed at {i} before the mutated bar"
        );
        assert_eq!(
            original.indicators[i].ema_fast.map(|v| v.to_bits()),
            changed.indicators[i].ema_fast.map(|v| v.to_bits())
        );
        assert_eq!(
            original.indicators[i].atr.map(|v| v.to_bits()),
            changed.indicators[i].atr.map(|v| v.to_bits())
        );
    }
}

#[test]
fn rejects_shorting_and_bad_ohlc() {
    let err = parse_policy_text("risk:\n  allow_short: true\n").unwrap_err();
    assert!(err.to_string().contains("short"));

    let bad = "date,open,high,low,close,volume\n2024-01-02,10,9,8,11,100\n";
    let err = load_prices_csv(bad).unwrap_err();
    assert!(err.to_string().contains("high"));
}
