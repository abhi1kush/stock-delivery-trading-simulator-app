//! Turning the ADX confirm trigger or the RSI cap off changes engine behaviour.
use simulator_core::execute;

const YAML: &str = include_str!("../../../policy/example_policy.yaml");
const CUPID: &str = include_str!("../../../public/data/CUPID.csv");
const TRENT: &str = include_str!("../../../public/data/TRENT.csv");

fn run(csv: &str, period: &str, adx: bool, rsi: bool) -> simulator_core::RunOutput {
    let y = YAML
        .replace("buy_on_adx_confirm: true", &format!("buy_on_adx_confirm: {adx}"))
        .replace("require_rsi_not_overbought: true", &format!("require_rsi_not_overbought: {rsi}"));
    execute(csv, &format!("{y}\ncandles:\n  period: {period}\n  scale_lengths: true\n")).unwrap()
}

fn summary(o: &simulator_core::RunOutput) -> (usize, f64) {
    (o.stats.num_trades, (o.stats.total_return_pct * 100.0).round() / 100.0)
}

#[test]
fn defaults_unchanged() {
    assert_eq!(summary(&run(CUPID, "1D", true, true)), (3, 40.81));
}

#[test]
fn adx_off_removes_the_adx_confirm_trigger() {
    // CUPID 1D: the third buy came from ADX crossing 20 on 2026-09-18; without it
    // the engine waits for the EMA cross on 2026-09-29 instead.
    let on = run(CUPID, "1D", true, true);
    let off = run(CUPID, "1D", false, true);
    assert!(on.trades.iter().any(|t| t.entry_explanation.contains("ADX crossed up")));
    assert!(off.trades.iter().all(|t| !t.entry_explanation.contains("ADX crossed up")));
    assert_eq!(summary(&off), (3, 40.23));
    assert!(off.trades[0].entry_explanation.contains("Off: ADX confirm"));
    // Trent 1D: two of its three buys were ADX-confirm buys.
    assert_eq!(summary(&run(TRENT, "1D", true, true)), (3, 42.06));
    assert_eq!(summary(&run(TRENT, "1D", false, true)), (1, 13.5));
}

#[test]
fn rsi_off_removes_the_overbought_cap() {
    // Trent 1W: the only weekly EMA cross had RSI 73 and was blocked by the cap.
    assert_eq!(summary(&run(TRENT, "1W", true, true)), (0, 0.0));
    let off = run(TRENT, "1W", true, false);
    assert_eq!(summary(&off), (1, 15.48));
    let e = &off.trades[0].entry_explanation;
    assert!(!e.contains("RSI 7"), "{e}");
    assert!(e.contains("Off: RSI cap"), "{e}");
    // CUPID 1D gains an ADX buy on 2026-06-08 that the cap used to block.
    assert_eq!(summary(&run(CUPID, "1D", true, false)).0, 4);
}
